use std::fmt;

use crate::ast::*;
// use crate::utils::*;

#[derive(Debug, Clone, PartialEq)]
enum ContextElement {
    Variable(String),
    Existential(String),
    Solved(String, Type),
    Marker(String),
    TypedVariable(String, Type),
}

impl fmt::Display for ContextElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ContextElement::Variable(var) => write!(f, "{}", var),
            ContextElement::Existential(ex) => write!(f, "{}^", ex),
            ContextElement::Solved(a, ty) => write!(f, "{}^: {}", a, ty),
            ContextElement::Marker(a) => write!(f, "<|{}", a),
            ContextElement::TypedVariable(x, ty) => write!(f, "{}: {}", x, ty),
        }
    }
}

/// As the context needs to be ordered, it is implemented as a simple Vector.
#[derive(Debug, Clone, PartialEq)]
struct Context {
    elements: Vec<ContextElement>,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[").unwrap();
        &self.elements.iter().fold(true, |first, ele| {
            if !first {
                write!(f, ", ").unwrap()
            };
            write!(f, "{}", ele).unwrap();
            false
        });
        write!(f, "]")
    }
}

/// Context operations derive from "Hole notation" described in 3.1 and the fact that the context is ordered.
impl Context {
    fn initial() -> Self {
        Context {
            elements: Vec::new(),
        }
    }

    fn add(&self, element: ContextElement) -> Self {
        let mut eles = self.elements.clone();
        eles.push(element);
        Context { elements: eles }
    }

    fn split_at(&self, element: ContextElement) -> (Context, Context) {
        if let Some(index) = self.elements.iter().position(|ele| ele == &element) {
            let (lhs, rhs) = self.elements.split_at(index);
            let left_context = Context {
                elements: lhs.to_vec(),
            };
            let right_context = Context {
                elements: rhs.to_vec(),
            };

            return (left_context, right_context);
        }
        panic!();
    }

    fn insert_in_place(&self, element: ContextElement, inserts: Vec<ContextElement>) -> Self {
        if let Some(index) = self.elements.iter().position(|ele| ele == &element) {
            let mut eles = self.elements.clone();
            let _ = eles.splice(index..=index, inserts).count();
            return Context { elements: eles };
        }
        panic!();
    }

    fn drop(&self, element: ContextElement) -> Self {
        if let Some(index) = self.elements.iter().position(|ele| ele == &element) {
            let mut eles = self.elements.clone();
            eles.split_off(index);
            return Context { elements: eles };
        }
        panic!();
    }

    fn get_solved(&self, alpha: &str) -> Option<&Type> {
        for ele in &self.elements {
            if let ContextElement::Solved(alpha1, tau) = ele {
                if alpha == alpha1 {
                    return Some(tau);
                }
            }
        }
        None
    }

    fn has_existential(&self, alpha: &str) -> bool {
        self.elements
            .iter()
            .any(|ele| ele == &ContextElement::Existential(alpha.to_string()))
    }

    fn has_variable(&self, alpha: &str) -> bool {
        self.elements
            .iter()
            .any(|ele| ele == &ContextElement::Variable(alpha.to_string()))
    }

    fn get_annotation(&self, x: &str) -> Option<&Type> {
        for ele in &self.elements {
            if let ContextElement::TypedVariable(var, type_) = ele {
                if var == x {
                    return Some(type_);
                }
            }
        }
        None
    }
}

/// The state is used to generate new existentials.
/// (In the paper mostly notated as α^ α1^ or β^)
/// It is passed around mutably everywhere
#[derive(Clone, Debug)]
struct State {
    existentials: usize,
}

impl State {
    fn initial() -> State {
        State { existentials: 0 }
    }

    fn fresh_existential(&mut self) -> String {
        let result = format!("t{}", self.existentials);
        self.existentials += 1;
        result
    }
}

fn literal_checks_against(literal: &Literal, type_: &LiteralType) -> bool {
    match (literal, type_) {
        (Literal::Bool(_), LiteralType::Bool) => true,
        (Literal::Nat(_), LiteralType::Nat) => true,
        (Literal::Int(_), LiteralType::Int) => true,
        (Literal::Real(_), LiteralType::Real) => true,
        (Literal::Char(_), LiteralType::Char) => true,
        (Literal::Str(_), LiteralType::Str) => true,
        _ => false,
    }
}

fn literal_synthesizes_to(literal: &Literal) -> LiteralType {
    match literal {
        Literal::Bool(_) => LiteralType::Bool,
        Literal::Nat(_) => LiteralType::Nat,
        Literal::Int(_) => LiteralType::Int,
        Literal::Real(_) => LiteralType::Real,
        Literal::Char(_) => LiteralType::Char,
        Literal::Str(_) => LiteralType::Str,
    }
}

/// Figure 7
fn is_well_formed(context: &Context, type_: &Type) -> bool {
    match type_ {
        Type::Literal(_) => true,
        Type::UniverseType => true,
        Type::Error => true,
        Type::Unknown => true,
        // TODO: make this work with proper idents
        Type::Ident(ref ident) => context.has_variable(&ident.ident.0),
        Type::Func(args, rtn) => {
            // args.iter()
            //     .fold(true, |first, a| first && is_well_formed(context, &a.0))
            //     && is_well_formed(context, &rtn.0)
            is_well_formed(context, &args.0) && is_well_formed(context, &rtn.0)
        }
        Type::Tuple(elements) => elements
            .iter()
            .fold(true, |first, a| first && is_well_formed(context, &a.0)),
        Type::Enum(variants) => variants.iter().fold(true, |first, a| {
            first
                && a.values
                    .iter()
                    .fold(true, |second, b| second && is_well_formed(context, &b.0))
        }),
        Type::Struct(items) => items
            .iter()
            .fold(true, |first, a| first && is_well_formed(context, &a.ty.0)),
        Type::Quantification(alpha, a) => {
            is_well_formed(&context.add(ContextElement::Variable(alpha.clone())), a)
        }
        Type::Existential(var) => context.has_existential(var) || context.get_solved(var).is_some(),
        Type::Intersection(a, b) => is_well_formed(context, a) && is_well_formed(context, b),
    }
}

/// Figure 8
fn apply_context(a: Type, context: &Context) -> Type {
    match a {
        Type::Literal(_) => a,
        Type::Ident(_) => a,
        Type::UniverseType => a,
        Type::Error => a,
        Type::Unknown => a,
        Type::Intersection(b, c) => Type::Intersection(
            Box::new(apply_context(*b, context)),
            Box::new(apply_context(*c, context)),
        ),
        Type::Existential(ref alpha) => {
            if let Some(tau) = context.get_solved(alpha) {
                apply_context(tau.clone(), context)
            } else {
                a
            }
        }
        Type::Quantification(alpha, a) => {
            Type::Quantification(alpha, Box::new(apply_context(*a, context)))
        }
        Type::Func(args, rtn) => Type::Func(
            // args.into_iter()
            //     .map(|a| (apply_context(a.0, context), a.1))
            //     .collect(),
            // Box::new((apply_context(rtn.0, context), rtn.1)),
            Box::new((apply_context(args.0, context), args.1)),
            Box::new((apply_context(rtn.0, context), rtn.1)),
        ),
        Type::Tuple(elements) => Type::Tuple(
            elements
                .into_iter()
                .map(|a| (apply_context(a.0, context), a.1))
                .collect(),
        ),
        Type::Struct(items) => Type::Struct(
            items
                .into_iter()
                .map(|a| Item {
                    name: a.name,
                    ty: (apply_context(a.ty.0, context), a.ty.1),
                })
                .collect(),
        ),
        Type::Enum(variants) => Type::Enum(
            variants
                .into_iter()
                .map(|a| EnumVariant {
                    name: a.name,
                    values: a
                        .values
                        .into_iter()
                        .map(|b| (apply_context(b.0, context), b.1))
                        .collect(),
                })
                .collect(),
        ),
    }
}
