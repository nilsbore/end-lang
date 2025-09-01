use std::fmt;

use crate::ast::*;
use crate::utils::Span;
use std::iter::zip;
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

fn print_helper(fun: &str, c1: String, c2: String, context: &Context) {
    print!(
        "{:<15} {:<85}| {:<25} {:<88}",
        fun,
        c1,
        c2,
        format!("{}", context)
    );
}

fn print_rule(rule: &str) {
    println!("{:>20}", rule);
}

/// Figure 11.
fn checks_against(state: &mut State, context: &Context, expr: &Expr, type_: &Type) -> Context {
    print_helper(
        "check",
        format!("{:?}", expr),
        format!("{}", type_),
        context,
    );
    assert!(is_well_formed(context, type_));
    match (expr, type_) {
        //1I
        (Expr::Literal(lit), Type::Literal(lit_ty)) => {
            print_rule("1I");
            assert!(literal_checks_against(lit, lit_ty));
            context.clone()
        }
        //->I
        // (Expr::Abstraction(x, e), Type::Function(a, b)) => {
        //     print_rule("->I");
        //     let typed_var = ContextElement::TypedVariable(x.clone(), *a.clone());
        //     let gamma = context.add(typed_var.clone());
        //     checks_against(state, &gamma, e, b).drop(typed_var)
        // }
        //forallI
        (_, Type::Quantification(alpha, a)) => {
            print_rule("∀I");
            let var = ContextElement::Variable(alpha.clone());
            let gamma = context.add(var.clone());
            checks_against(state, &gamma, expr, a).drop(var)
        }
        //xI
        (Expr::Tuple(elements), Type::Tuple(tys)) => {
            print_rule("xI");
            assert!(elements.len() == tys.len());
            let mut gamma = context.clone();
            for (element, ty) in zip(elements, tys) {
                gamma = checks_against(state, &gamma, &element.0, &ty.0);
            }
            gamma
        }
        (Expr::Object(items), Type::Struct(tys)) => {
            assert!(items.len() == tys.len());
            let mut sorted_tys: Vec<_> = tys.iter().map(|a| a.clone()).collect();
            sorted_tys.sort_by(|a, b| a.name.cmp(&b.name));
            let mut sorted_items: Vec<_> = items.iter().map(|a| a.clone()).collect();
            sorted_items.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));
            print_rule("xI");
            let mut gamma = context.clone();
            for (item, ty) in zip(sorted_items, sorted_tys) {
                assert!(item.0 .0 == ty.name);
                gamma = checks_against(state, &gamma, &item.1 .0, &ty.ty.0);
            }
            gamma
        }
        //^I - Intersection type checking
        (_, Type::Intersection(a, b)) => {
            print_rule("^I");
            // To check against A^B, we need to check against both A and B
            let gamma = checks_against(state, context, expr, a);
            checks_against(state, &gamma, expr, b)
        }
        //Sub
        (_, _) => {
            print_rule("Sub");
            // context.clone()
            let (a, theta) = synthesizes_to(state, context, expr);
            subtype(
                state,
                &theta,
                &apply_context(a, &theta),
                &apply_context(type_.clone(), &theta),
            )
        }
    }
}

/// This corresponds to the FV call in Figure 9 Rule <:InstantiateL and <:InstantiateR
/// It checks if a existential variable already occurs in a type to be able to find and panic on cycles
///
/// Alas, I could not find a definition of the FV function and had to copy the implementation of
/// https://github.com/ollef/Bidirectional and https://github.com/atennapel/bidirectional.js
fn occurs_in(alpha: &str, a: &Type) -> bool {
    match a {
        Type::Literal(_) => false,
        Type::Ident(var) => alpha == var.path[0].0,
        Type::Func(t1, t2) => occurs_in(alpha, &t1.0) || occurs_in(alpha, &t2.0),
        Type::Quantification(beta, t) => {
            if alpha == beta {
                return true;
            } else {
                return occurs_in(alpha, t);
            }
        }
        Type::Existential(var) => alpha == var,
        Type::Tuple(items) => items.iter().fold(false, |a, b| a || occurs_in(alpha, &b.0)), //occurs_in(alpha, a) || occurs_in(alpha, b),
        Type::Struct(items) => items
            .iter()
            .fold(false, |a, b| a || occurs_in(alpha, &b.ty.0)),
        Type::Enum(variants) => variants.iter().fold(false, |a, b| {
            a || b.values.iter().fold(false, |_c, d| occurs_in(alpha, &d.0))
        }),
        Type::Intersection(a, b) => occurs_in(alpha, &a) || occurs_in(alpha, &b),
        Type::Error => false,
        Type::Unknown => false,
        Type::UniverseType => false,
    }
}

/// Figure 10
fn instantiate_l(state: &mut State, context: &Context, alpha: &str, b: &Type) -> Context {
    match b {
        Type::Existential(beta) if context.has_existential(beta) => {
            // InstLReach: both are existentials, pick the leftmost one
            if let Some(alpha_pos) = context.elements.iter().position(|e| 
                matches!(e, ContextElement::Existential(name) if name == alpha)
            ) {
                if let Some(beta_pos) = context.elements.iter().position(|e|
                    matches!(e, ContextElement::Existential(name) if name == beta)
                ) {
                    if alpha_pos <= beta_pos {
                        // α^ comes before β^, so α^ := β^
                        context.insert_in_place(
                            ContextElement::Existential(alpha.to_string()),
                            vec![ContextElement::Solved(alpha.to_string(), Type::Existential(beta.clone()))]
                        )
                    } else {
                        // β^ comes before α^, so β^ := α^
                        context.insert_in_place(
                            ContextElement::Existential(beta.clone()),
                            vec![ContextElement::Solved(beta.clone(), Type::Existential(alpha.to_string()))]
                        )
                    }
                } else {
                    panic!("Beta existential not found in context")
                }
            } else {
                panic!("Alpha existential not found in context")
            }
        }
        Type::Func(a, c) => {
            // InstLArr: α^ :≤ A → C becomes α^1 → α^2 :≤ A → C
            let alpha1 = state.fresh_existential();
            let alpha2 = state.fresh_existential();
            let func_type = Type::Func(
                Box::new((Type::Existential(alpha1.clone()), Span::default())),
                Box::new((Type::Existential(alpha2.clone()), Span::default()))
            );
            let gamma = context.insert_in_place(
                ContextElement::Existential(alpha.to_string()),
                vec![
                    ContextElement::Existential(alpha2.clone()),
                    ContextElement::Existential(alpha1.clone()),
                    ContextElement::Solved(alpha.to_string(), func_type)
                ]
            );
            let theta = instantiate_r(state, &gamma, &a.0, &alpha1);
            subtype(state, &theta, &Type::Existential(alpha2), &c.0)
        }
        Type::Quantification(beta, c) => {
            // InstLAllR: α^ :≤ ∀β.C becomes α^ :≤ [β^/β]C
            let beta_hat = state.fresh_existential();
            let gamma = context.add(ContextElement::Existential(beta_hat.clone()));
            let substituted_c = substitution(c, beta, &Type::Existential(beta_hat));
            instantiate_l(state, &gamma, alpha, &substituted_c)
        }
        _ => {
            // InstLSolve: solve α^ := B if α^ ∉ FV(B)
            if !occurs_in(alpha, b) {
                context.insert_in_place(
                    ContextElement::Existential(alpha.to_string()),
                    vec![ContextElement::Solved(alpha.to_string(), b.clone())]
                )
            } else {
                panic!("Occurs check failed in instantiate_l")
            }
        }
    }
}

/// Figure 10
fn instantiate_r(state: &mut State, context: &Context, a: &Type, alpha: &str) -> Context {
    match a {
        Type::Existential(beta) if context.has_existential(beta) => {
            // InstRReach: both are existentials, pick the leftmost one
            if let Some(alpha_pos) = context.elements.iter().position(|e| 
                matches!(e, ContextElement::Existential(name) if name == alpha)
            ) {
                if let Some(beta_pos) = context.elements.iter().position(|e|
                    matches!(e, ContextElement::Existential(name) if name == beta)
                ) {
                    if beta_pos <= alpha_pos {
                        // β^ comes before α^, so β^ := α^
                        context.insert_in_place(
                            ContextElement::Existential(beta.clone()),
                            vec![ContextElement::Solved(beta.clone(), Type::Existential(alpha.to_string()))]
                        )
                    } else {
                        // α^ comes before β^, so α^ := β^
                        context.insert_in_place(
                            ContextElement::Existential(alpha.to_string()),
                            vec![ContextElement::Solved(alpha.to_string(), Type::Existential(beta.clone()))]
                        )
                    }
                } else {
                    panic!("Beta existential not found in context")
                }
            } else {
                panic!("Alpha existential not found in context")
            }
        }
        Type::Func(a_arg, a_ret) => {
            // InstRArr: A → C :≤ α^ becomes A → C :≤ α^1 → α^2
            let alpha1 = state.fresh_existential();
            let alpha2 = state.fresh_existential();
            let func_type = Type::Func(
                Box::new((Type::Existential(alpha1.clone()), Span::default())),
                Box::new((Type::Existential(alpha2.clone()), Span::default()))
            );
            let gamma = context.insert_in_place(
                ContextElement::Existential(alpha.to_string()),
                vec![
                    ContextElement::Existential(alpha2.clone()),
                    ContextElement::Existential(alpha1.clone()),
                    ContextElement::Solved(alpha.to_string(), func_type)
                ]
            );
            let theta = instantiate_l(state, &gamma, &alpha1, &a_arg.0);
            subtype(state, &theta, &a_ret.0, &Type::Existential(alpha2))
        }
        Type::Quantification(beta, c) => {
            // InstRAllL: ∀β.C :≤ α^ becomes [β/β]C :≤ α^ where β is fresh
            let gamma = context.add(ContextElement::Variable(beta.clone()));
            let result = instantiate_r(state, &gamma, c, alpha);
            result.drop(ContextElement::Variable(beta.clone()))
        }
        _ => {
            // InstRSolve: solve α^ := A if α^ ∉ FV(A)
            if !occurs_in(alpha, a) {
                context.insert_in_place(
                    ContextElement::Existential(alpha.to_string()),
                    vec![ContextElement::Solved(alpha.to_string(), a.clone())]
                )
            } else {
                panic!("Occurs check failed in instantiate_r")
            }
        }
    }
}

/// Figure 9
fn subtype(state: &mut State, context: &Context, a: &Type, b: &Type) -> Context {
    print_helper("subtype", format!("{}", a), format!("{}", b), context);
    assert!(is_well_formed(context, a));
    assert!(is_well_formed(context, b));
    match (a, b) {
        //<:Unit
        (Type::Literal(lit_a), Type::Literal(lit_b)) => {
            print_rule("<:Unit");
            assert_eq!(lit_a, lit_b);
            context.clone()
        }
        //<:Var
        (Type::Ident(alpha1), Type::Ident(alpha2)) => {
            print_rule("<:Var");
            if is_well_formed(context, a) && alpha1 == alpha2 {
                return context.clone();
            } else {
                panic!();
            }
        }
        //<:Exvar
        (Type::Existential(exist1), Type::Existential(exist2)) if exist1 == exist2 => {
            print_rule("<:Exvar");
            if is_well_formed(context, a) {
                return context.clone();
            } else {
                panic!();
            }
        }
        //<:->
        (Type::Func(a1, a2), Type::Func(b1, b2)) => {
            print_rule("<:->");
            let theta = subtype(state, context, &a1.0, &b1.0);
            return subtype(
                state,
                &theta,
                &apply_context(a2.0.clone(), &theta),
                &apply_context(b2.0.clone(), &theta),
            );
        }
        (Type::Tuple(a1), Type::Tuple(a2)) => {
            print_rule("SubProduct");
            let mut gamma = context.clone();
            for (v1, v2) in zip(a1.iter(), a2.iter()) {
                gamma = subtype(state, &gamma, &v1.0, &v2.0);
            }
            gamma
        }
        //<:forallL
        (Type::Quantification(alpha, a), _) => {
            print_rule("<:∀L");
            let r1 = state.fresh_existential();
            let gamma = context
                .add(ContextElement::Marker(r1.clone()))
                .add(ContextElement::Existential(r1.clone()));
            let substituted_a = substitution(a, alpha, &Type::Existential(r1.clone()));
            let delta = subtype(state, &gamma, &substituted_a, b);
            return delta.drop(ContextElement::Marker(r1.clone()));
        }
        //<:forallR
        (_, Type::Quantification(alpha, b)) => {
            print_rule("<:∀R");
            let theta = context.add(ContextElement::Variable(alpha.clone()));
            let delta = subtype(state, &theta, a, b);
            return delta.drop(ContextElement::Variable(alpha.clone()));
        }
        //<:InstatiateL
        (Type::Existential(alpha), _) => {
            print_rule("<:InstantiateL");
            if !occurs_in(alpha, b) {
                instantiate_l(state, context, alpha, b)
            } else {
                panic!("Circular!");
            }
        }
        //<:InstantiateR
        (_, Type::Existential(alpha)) => {
            print_rule("<:InstantiateR");
            if !occurs_in(alpha, a) {
                instantiate_r(state, context, a, alpha)
            } else {
                panic!("Circular!");
            }
        }
        //<:^ - Intersection subtyping
        (Type::Intersection(a, b), c) => {
            print_rule("<:^");
            // A^B <: C if A <: C OR B <: C (try both)
            // Try A <: C first
            let result_a = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                subtype(state, context, a, c)
            }));
            if result_a.is_ok() {
                return result_a.unwrap();
            }
            // If A <: C fails, try B <: C
            subtype(state, context, b, c)
        }
        // Additional subtyping rules for built-in types
        (Type::Literal(LiteralType::Nat), Type::Ident(ident)) => {
            let type_name = &ident.path[0].0;
            if matches!(type_name.as_str(), "i32" | "u32" | "i64" | "u64" | "usize") {
                print_rule("<:NatToInt");
                context.clone()
            } else {
                panic!("Cannot subtype Nat to {}", type_name);
            }
        }
        (Type::Literal(LiteralType::Bool), Type::Ident(ident)) => {
            let type_name = &ident.path[0].0;
            if type_name == "bool" {
                print_rule("<:BoolToBool");
                context.clone()
            } else {
                panic!("Cannot subtype Bool to {}", type_name);
            }
        }
        (Type::Literal(LiteralType::Str), Type::Ident(ident)) => {
            let type_name = &ident.path[0].0;
            if matches!(type_name.as_str(), "str" | "String") {
                print_rule("<:StrToString");
                context.clone()
            } else {
                panic!("Cannot subtype Str to {}", type_name);
            }
        }
        _ => {
            panic!("Couldn't subtype {} <: {}", a, b);
        }
    }
}

///Figure 11
fn synthesizes_to(state: &mut State, context: &Context, expr: &Expr) -> (Type, Context) {
    print_helper("synth", format!("{:?}", expr), "".into(), context);
    match expr {
        //1I=>
        Expr::Literal(lit) => {
            print_rule("1I=>");
            (Type::Literal(literal_synthesizes_to(lit)), context.clone())
        }
        //Var
        Expr::QualifiedIdent(x) => {
            print_rule("Var");
            if let Some(annotation) = context.get_annotation(&x.path[0].0) {
                return (annotation.clone(), context.clone());
            };
            panic!();
        }
        //Anno
        Expr::Annotation(e, annotation) => {
            print_rule("Anno");
            if is_well_formed(context, annotation) {
                let delta = checks_against(state, context, e, annotation);
                return (annotation.clone(), delta);
            }
            panic!();
        }
        //->I=>
        // Expr::Abstraction(x, e) => {
        //     print_rule("->I=>");
        //     let alpha = state.fresh_existential();
        //     let beta = state.fresh_existential();
        //     let gamma = context
        //         .add(ContextElement::Existential(alpha.clone()))
        //         .add(ContextElement::Existential(beta.clone()))
        //         .add(ContextElement::TypedVariable(
        //             x.clone(),
        //             Type::Existential(alpha.clone()),
        //         ));
        //     let delta = checks_against(state, &gamma, e, &Type::Existential(beta.clone())).drop(
        //         ContextElement::TypedVariable(x.clone(), Type::Existential(alpha.clone())),
        //     );
        //     return (
        //         Type::Function(
        //             Box::new(Type::Existential(alpha.clone())),
        //             Box::new(Type::Existential(beta.clone())),
        //         ),
        //         delta,
        //     );
        // }
        Expr::Tuple(elements) => {
            print_rule("SynthProduct");
            let mut gamma = context.clone();
            let mut tuple_types = Vec::new();
            for element in elements {
                let (element_type, new_gamma) = synthesizes_to(state, &gamma, &element.0);
                tuple_types.push((element_type, element.1.clone()));
                gamma = new_gamma;
            }
            (Type::Tuple(tuple_types), gamma)
        }
        // TODO: Implement Let expressions when they're added to the AST
        // Expr::Let(var, expr, body) => {
        //     print_rule("Let");
        //     let (t0, gamma) = synthesizes_to(state, context, expr);
        //     let theta = gamma.add(ContextElement::TypedVariable(var.clone(), t0.clone()));
        //     let (t1, delta) = synthesizes_to(state, &theta, body);
        //     return (
        //         t1,
        //         delta.insert_in_place(ContextElement::TypedVariable(var.clone(), t0), vec![]),
        //     );
        // }

        //->E
        Expr::Call(e1, args) => {
            print_rule("->E");
            let (func_type, theta) = synthesizes_to(state, context, &e1.0);
            
            // For now, handle simple single-argument calls
            // TODO: Implement full multi-argument and named argument support
            if args.len() == 1 {
                let arg = &args[0].1; // Get the expression, ignore the optional name
                return application_synthesizes_to(state, &theta, &apply_context(func_type, &theta), &arg.0);
            } else if args.is_empty() {
                // No arguments - this might be a nullary function call
                return (func_type, theta);
            } else {
                // Multiple arguments - need to handle currying or tuple arguments
                // For now, just panic with a helpful message
                panic!("Multi-argument function calls not yet implemented");
            }
        }
        // Missing patterns - add basic implementations
        Expr::Error => {
            print_rule("Error");
            (Type::Error, context.clone())
        }
        Expr::Object(_items) => {
            print_rule("Object");
            // TODO: Implement object type synthesis
            (Type::Unknown, context.clone())
        }
        Expr::Binary(left, op, right) => {
            print_rule("Binary");
            // Synthesize types for both operands
            let (left_type, gamma) = synthesizes_to(state, context, &left.0);
            let (right_type, delta) = synthesizes_to(state, &gamma, &right.0);
            
            // Determine result type based on operation and operand types
            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    // Arithmetic operations: require same numeric types
                    let result_type = match (&left_type, &right_type) {
                        (Type::Literal(LiteralType::Nat), Type::Literal(LiteralType::Nat)) => Type::Literal(LiteralType::Nat),
                        (Type::Ident(l), Type::Ident(r)) if l.path[0].0 == r.path[0].0 => {
                            let type_name = &l.path[0].0;
                            if matches!(type_name.as_str(), "i32" | "u32" | "i64" | "u64" | "f32" | "f64") {
                                left_type.clone()
                            } else {
                                Type::Unknown
                            }
                        }
                        _ => Type::Unknown
                    };
                    (result_type, delta)
                }
                BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Less | BinaryOp::LessEq | 
                BinaryOp::Greater | BinaryOp::GreaterEq => {
                    // Comparison operations: return bool
                    (Type::Ident(QualifiedType {
                        path: vec![("bool".to_string(), Span::default())],
                        type_args: vec![],
                    }), delta)
                }
                _ => {
                    // Other operations - not yet implemented
                    (Type::Unknown, delta)
                }
            }
        }
        Expr::Member(_expr, _field) => {
            print_rule("Member");
            // TODO: Implement member access type synthesis
            (Type::Unknown, context.clone())
        }
        // Add support for synthesizing intersection types from structural literals
        // This will be needed when we have object literals that should get nominal+structural types
    }
}

/// Similar to the FV function from subtyping I couldn't find a definition of substitution in the paper
/// Thus I tried to copy the implementation of
/// https://github.com/ollef/Bidirectional and https://github.com/atennapel/bidirectional.js
///
/// Substitution is written in the paper as [α^/α]A which means, α is replaced with α^ in all occurrences in A
fn substitution(a: &Type, alpha: &str, b: &Type) -> Type {
    match a {
        Type::Literal(_) => a.clone(),
        Type::Ident(var) => {
            if var.path[0].0 == alpha {
                b.clone()
            } else {
                a.clone()
            }
        }
        Type::Quantification(var, type_) => {
            if var == alpha {
                Type::Quantification(var.clone(), Box::new(b.clone()))
            } else {
                Type::Quantification(var.clone(), Box::new(substitution(type_, alpha, b)))
            }
        }
        Type::Existential(var) => {
            if var == alpha {
                b.clone()
            } else {
                a.clone()
            }
        }
        Type::Tuple(items) => Type::Tuple(
            items
                .iter()
                .map(|a| (substitution(&a.0, alpha, b), a.1.clone()))
                .collect(),
        ),
        Type::Struct(items) => Type::Struct(
            items
                .iter()
                .map(|a| Item {
                    name: a.name.clone(),
                    ty: (substitution(&a.ty.0, alpha, b), a.ty.1.clone()),
                })
                .collect(),
        ),
        Type::Enum(items) => Type::Enum(
            items
                .iter()
                .map(|a| EnumVariant {
                    name: a.name.clone(),
                    values: a
                        .values
                        .iter()
                        .map(|c| (substitution(&c.0, alpha, b), c.1.clone()))
                        .collect(),
                })
                .collect(),
        ),
        Type::Func(t1, t2) => Type::Func(
            Box::new((substitution(&t1.0, alpha, b), t1.1.clone())),
            Box::new((substitution(&t2.0, alpha, b), t2.1.clone())),
        ),
        Type::Intersection(t1, t2) => Type::Intersection(
            Box::new(substitution(&t1, alpha, b)),
            Box::new(substitution(&t2, alpha, b)),
        ),
        Type::UniverseType => Type::UniverseType,
        Type::Error => Type::Error,
        Type::Unknown => Type::Unknown,
    }
}

//Figure 11
fn application_synthesizes_to(
    state: &mut State,
    context: &Context,
    type_: &Type,
    expr: &Expr,
) -> (Type, Context) {
    print_helper(
        "app_synth",
        format!("{:?}", expr),
        format!("{}", type_),
        context,
    );
    match type_ {
        //alphaApp
        Type::Existential(alpha) => {
            print_rule("α^App");
            let alpha1 = state.fresh_existential();
            let alpha2 = state.fresh_existential();
            let gamma = context.insert_in_place(
                ContextElement::Existential(alpha.to_string()),
                vec![
                    ContextElement::Existential(alpha2.clone()),
                    ContextElement::Existential(alpha1.clone()),
                    ContextElement::Solved(
                        alpha.clone(),
                        Type::Func(
                            Box::new((Type::Existential(alpha1.clone()), Span::default())),
                            Box::new((Type::Existential(alpha2.clone()), Span::default())),
                        ),
                    ),
                ],
            );
            let delta = checks_against(state, &gamma, expr, &Type::Existential(alpha1.clone()));
            return (Type::Existential(alpha2.clone()), delta);
        }
        //ForallApp
        Type::Quantification(alpha, a) => {
            print_rule("∀App");
            let alpha1 = state.fresh_existential();
            let gamma = context.add(ContextElement::Existential(alpha1.clone()));
            let substituted_a = substitution(a, alpha, &Type::Existential(alpha1));
            return application_synthesizes_to(state, &gamma, &substituted_a, expr);
        }
        //App
        Type::Func(a, c) => {
            print_rule("->App");
            let delta = checks_against(state, context, expr, &a.0);
            return (c.0.clone(), delta);
        }
        _ => panic!(),
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
        // Handle built-in types and user-defined types
        Type::Ident(ref ident) => {
            let type_name = &ident.path[0].0;
            let is_builtin = matches!(type_name.as_str(), 
                "i32" | "u32" | "i64" | "u64" | "f32" | "f64" | 
                "bool" | "char" | "str" | "String" | "usize" | "isize");
            
            (is_builtin || context.has_variable(type_name))
                && ident
                    .type_args
                    .iter()
                    .fold(true, |a, b| a && is_well_formed(context, &b.0))
        }
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
        Type::Ident(ident) => Type::Ident(QualifiedType {
            path: ident.path,
            type_args: ident
                .type_args
                .into_iter()
                .map(|a| (apply_context(a.0, context), a.1))
                .collect(),
        }),
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

/// Public interface for type checking a complete program
pub fn check_program(program: &Program) -> Result<(), String> {
    let state = State::initial();
    let mut context = Context::initial();
    
    // First pass: collect type definitions and function signatures
    for def in &program.defs {
        match def {
            Def::Type(type_def) => {
                // Add type definition to context with intersection type
                let type_name = &type_def.name.0;
                let structural_type = &type_def.ty.0;
                
                // Create intersection type: TypeName^StructuralType
                let nominal_type = Type::Ident(QualifiedType {
                    path: vec![(type_name.clone(), type_def.name.1.clone())],
                    type_args: vec![],
                });
                
                // For structural types (like structs), create intersection
                let intersection_type = match structural_type {
                    Type::Struct(_) | Type::Enum(_) => {
                        Type::Intersection(
                            Box::new(nominal_type.clone()),
                            Box::new(structural_type.clone())
                        )
                    }
                    _ => {
                        // For non-structural types, just use the nominal type
                        nominal_type
                    }
                };
                
                context = context.add(ContextElement::TypedVariable(
                    type_name.clone(), 
                    intersection_type
                ));
            }
            Def::Func(func) => {
                // Add function signature to context
                let func_name = &func.ident;
                let func_type = Type::Func(
                    Box::new((Type::Unknown, Span::default())), // Will be properly inferred later
                    Box::new((func.rtn.0.clone(), func.rtn.1.clone())),
                );
                context = context.add(ContextElement::TypedVariable(
                    func_name.clone(),
                    func_type
                ));
            }
            Def::Const(const_def) => {
                // Add constant to context
                let const_name = &const_def.name.0;
                context = context.add(ContextElement::TypedVariable(
                    const_name.clone(),
                    const_def.ty.0.clone()
                ));
            }
            _ => {
                // For now, skip other definition types
                println!("Skipping definition type: {:?}", def);
            }
        }
    }
    
    // Second pass: type check function bodies and other expressions
    for def in &program.defs {
        match def {
            Def::Func(func) => {
                // Type check function body with parameters in context
                match &*func.body {
                    AssignExpr::Expr(expr) => {
                        // Create context with generic parameters and function parameters  
                        let mut func_context = context.clone();
                        
                        // Add generic type parameters if any
                        if let Some(generics) = &func.generics {
                            for generic_param in &generics.args {
                                func_context = func_context.add(ContextElement::Variable(
                                    generic_param.name.clone()
                                ));
                            }
                        }
                        
                        // Add function parameters
                        for param in &func.args {
                            func_context = func_context.add(ContextElement::TypedVariable(
                                param.name.clone(),
                                param.ty.0.clone()
                            ));
                        }
                        
                        // Try type checking with parameter context
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            synthesizes_to(&mut state.clone(), &func_context, &expr.0)
                        })) {
                            Ok((inferred_type, _)) => {
                                println!("Function {} has type: {}", func.ident, inferred_type);
                            }
                            Err(_) => {
                                return Err(format!("Type checking failed for function {}", func.ident));
                            }
                        }
                    }
                    _ => {
                        println!("Complex function body not yet supported for {}", func.ident);
                    }
                }
            }
            Def::Const(const_def) => {
                // Type check constant expression
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    checks_against(&mut state.clone(), &context, &const_def.value.0, &const_def.ty.0)
                })) {
                    Ok(_) => {
                        println!("Constant {} type checks successfully", const_def.name.0);
                    }
                    Err(_) => {
                        return Err(format!("Type checking failed for constant {}", const_def.name.0));
                    }
                }
            }
            _ => {
                // Skip other definitions for now
            }
        }
    }
    
    Ok(())
}
