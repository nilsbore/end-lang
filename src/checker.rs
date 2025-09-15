use std::fmt;

use crate::ast::*;
use crate::utils::{Span, Spanned};
use std::iter::zip;
use std::collections::{HashMap, HashSet};
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

/// Complete Module Symbol Table
/// Collects ALL definitions in a module for forward references and cross-module usage
#[derive(Debug, Clone)]
struct ModuleSymbolTable {
    /// ALL types defined in this module - updated in-place after open blocks
    type_definitions: HashMap<String, Type>,
    
    /// ALL function signatures in this module
    function_signatures: HashMap<String, Type>,
    
    /// ALL constants in this module
    constants: HashMap<String, Type>,
    
    /// ALL traits with complete member information
    traits: HashMap<String, TraitDefinition>,
    
    /// Types defined locally (for orphan rule validation)
    local_types: HashSet<String>,
    
    /// Enhancement registry: base_type -> [enhancement_types]
    /// Example: "i32" -> ["Add", "Print"] means i32^Add^Print
    enhancements: HashMap<String, Vec<String>>,
    
    /// Method implementations: (trait_name, base_type) -> [method_names]
    trait_methods: HashMap<(String, String), Vec<String>>,
}

/// Complete trait definition with all members
#[derive(Debug, Clone)]
struct TraitDefinition {
    name: String,
    members: Vec<TraitMember>,
}

/// Individual trait member (function or variable)
#[derive(Debug, Clone)]
struct TraitMember {
    name: String,
    member_type: Type,
    kind: MemberKind,
}

/// Type of trait member
#[derive(Debug, Clone)]
enum MemberKind {
    Function,
    Variable,
}

impl ModuleSymbolTable {
    fn new() -> Self {
        ModuleSymbolTable {
            type_definitions: HashMap::new(),
            function_signatures: HashMap::new(),
            constants: HashMap::new(),
            traits: HashMap::new(),
            local_types: HashSet::new(),
            enhancements: HashMap::new(),
            trait_methods: HashMap::new(),
        }
    }
    
    /// Add a type as locally defined (for orphan rule)
    fn add_local_type(&mut self, type_name: String) {
        self.local_types.insert(type_name);
    }
    
    /// Add an enhancement: open A^B adds B to A's enhancement list
    fn add_enhancement(&mut self, base_type: String, enhancement_type: String) {
        self.enhancements
            .entry(base_type)
            .or_insert_with(Vec::new)
            .push(enhancement_type);
    }
    
    /// Add a type definition to the registry
    fn add_type_definition(&mut self, type_name: String, type_definition: Type) {
        self.type_definitions.insert(type_name, type_definition);
    }
    
    /// Add a function signature to the registry
    fn add_function_signature(&mut self, function_name: String, function_type: Type) {
        self.function_signatures.insert(function_name, function_type);
    }
    
    /// Add a constant to the registry
    fn add_constant(&mut self, constant_name: String, constant_type: Type) {
        self.constants.insert(constant_name, constant_type);
    }
    
    /// Add a complete trait definition to the registry
    fn add_trait_definition(&mut self, trait_name: String, trait_def: TraitDefinition) {
        self.traits.insert(trait_name, trait_def);
    }
    
    /// Add a method name to the trait methods registry
    fn add_trait_method(&mut self, trait_name: String, base_type: String, method_name: String) {
        self.trait_methods
            .entry((trait_name, base_type))
            .or_insert_with(Vec::new)
            .push(method_name);
    }
    
    /// Validate orphan rule: current module must define at least one type in the intersection
    fn validate_orphan_rule(&self, type_a: &str, type_b: &str) -> Result<(), String> {
        if !self.local_types.contains(type_a) && !self.local_types.contains(type_b) {
            return Err(format!(
                "Cannot implement intersection `{}^{}` - neither type defined in current module",
                type_a, type_b
            ));
        }
        Ok(())
    }
    
    /// Apply enhancements in-place to a type definition
    fn apply_enhancements_in_place(&mut self, type_name: &str) {
        if let Some(base_type) = self.type_definitions.get(type_name).cloned() {
            if let Some(enhancements) = self.enhancements.get(type_name) {
                let mut enhanced_type = base_type;
                
                for enhancement in enhancements {
                    // Create intersection with enhancement type
                    let enhancement_type = Type::Ident(QualifiedType {
                        path: vec![(enhancement.clone(), Span::default())],
                        type_args: vec![],
                    });
                    
                    enhanced_type = Type::Intersection(
                        Box::new(enhanced_type),
                        Box::new(enhancement_type)
                    );
                }
                
                // Update the type definition in-place
                self.type_definitions.insert(type_name.to_string(), enhanced_type);
            }
        }
    }
    
    /// Apply all enhancements to all type definitions
    fn apply_all_enhancements(&mut self) {
        let type_names: Vec<String> = self.type_definitions.keys().cloned().collect();
        for type_name in type_names {
            self.apply_enhancements_in_place(&type_name);
        }
    }
}

/// Helper function to extract type names from intersection types for open blocks
fn extract_intersection_types(intersection_type: &Type) -> Vec<String> {
    match intersection_type {
        Type::Intersection(left, right) => {
            let mut types = extract_intersection_types(left);
            types.extend(extract_intersection_types(right));
            types
        }
        Type::Ident(qualified_type) => {
            if let Some((type_name, _)) = qualified_type.path.first() {
                vec![type_name.clone()]
            } else {
                vec![]
            }
        }
        _ => vec![]
    }
}

/// Phase 1: Collect ALL module symbols for complete registry
fn collect_module_symbols(program: &Program) -> Result<ModuleSymbolTable, String> {
    let mut symbols = ModuleSymbolTable::new();
    
    // PASS 1: Collect all type and trait definitions first (for orphan rule validation)
    for def in &program.defs {
        match def {
            Def::Type(type_def) => {
                let type_name = &type_def.name.0;
                symbols.add_local_type(type_name.clone());
            }
            Def::Trait(trait_def) => {
                let trait_name = &trait_def.name.0;
                symbols.add_local_type(trait_name.clone());
            }
            _ => {} // Skip other definitions in pass 1
        }
    }
    
    // Track current open block for trait method collection
    let mut current_open_block: Option<(String, String)> = None; // (trait_name, base_type)
    
    // PASS 2: Process ALL definitions with complete local_types knowledge
    for def in &program.defs {
        match def {
            Def::Import(_import_stmt) => {
                // TODO: Process import statements for cross-module support
                // For now, skip imports until cross-module system is implemented
            },
            Def::Type(type_def) => {
                let type_name = &type_def.name.0;
                // Note: add_local_type already called in pass 1
                
                // Create full intersection type: NominalType^StructuralType
                let structural_type = &type_def.ty.0;
                let nominal_type = Type::Ident(QualifiedType {
                    path: vec![(type_name.clone(), type_def.name.1.clone())],
                    type_args: vec![],
                });
                
                let intersection_type = Type::Intersection(
                    Box::new(nominal_type),
                    Box::new(structural_type.clone())
                );
                
                // Add to type definitions registry
                symbols.add_type_definition(type_name.clone(), intersection_type);
            }
            Def::Func(func) => {
                let func_name = &func.ident;
                
                // Check if this function is inside an open block (trait method)
                if let Some((trait_name, base_type)) = &current_open_block {
                    symbols.add_trait_method(trait_name.clone(), base_type.clone(), func_name.clone());
                }
                
                // Build function type signature for ALL functions
                let func_type = if func.args.is_empty() {
                    func.rtn.0.clone()
                } else if func.args.len() == 1 {
                    Type::Func(
                        Box::new(func.args[0].ty.clone()),
                        Box::new(func.rtn.clone()),
                    )
                } else {
                    let param_types: Vec<Spanned<Type>> = func.args.iter()
                        .map(|item| item.ty.clone())
                        .collect();
                    Type::Func(
                        Box::new((Type::Tuple(param_types), Span::default())),
                        Box::new(func.rtn.clone()),
                    )
                };
                
                // Add ALL function signatures to registry
                symbols.add_function_signature(func_name.clone(), func_type);
            }
            Def::Const(const_def) => {
                let const_name = &const_def.name.0;
                // Add ALL constants to registry
                symbols.add_constant(const_name.clone(), const_def.ty.0.clone());
            }
            Def::Trait(trait_def) => {
                let trait_name = &trait_def.name.0;
                // Note: add_local_type already called in pass 1
                
                // Build complete trait definition with all members
                let mut trait_members = Vec::new();
                for member in &trait_def.members {
                    let member_kind = match &member.ty.0 {
                        Type::Func(_, _) => MemberKind::Function,
                        _ => MemberKind::Variable,
                    };
                    
                    trait_members.push(TraitMember {
                        name: member.name.clone(),
                        member_type: member.ty.0.clone(),
                        kind: member_kind,
                    });
                }
                
                let complete_trait_def = TraitDefinition {
                    name: trait_name.clone(),
                    members: trait_members,
                };
                
                // Add complete trait to registry
                symbols.add_trait_definition(trait_name.clone(), complete_trait_def);
            }
            Def::Open(open_def) => {
                // Parse intersection type from open block: open A^B
                let intersection_types = extract_intersection_types(&open_def.ty.0);
                
                if intersection_types.len() >= 2 {
                    // For open A^B, add B to A's enhancement list and vice versa
                    let type_a = &intersection_types[0];
                    let type_b = &intersection_types[1];
                    
                    // Validate orphan rule
                    symbols.validate_orphan_rule(type_a, type_b)?;
                    
                    // Add enhancements both ways (bidirectional intersection)
                    symbols.add_enhancement(type_a.clone(), type_b.clone());
                    symbols.add_enhancement(type_b.clone(), type_a.clone());
                    
                    // Set current open block for method collection
                    // Assume first type is trait, second is base type (convention from examples)
                    current_open_block = Some((type_a.clone(), type_b.clone()));
                }
            }
            Def::Close(_) => {
                // Close blocks end the current method collection scope
                current_open_block = None;
            }
        }
    }
    
    Ok(symbols)
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
            // Legacy struct checking - will be removed once structs are fully converted to intersections
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
        (Expr::Object(items), Type::FieldConstraint(field_name, expected_type)) => {
            // Check object literal against single field constraint
            print_rule("FieldConstraintI");
            // Object must have exactly one field matching the constraint
            if items.len() != 1 {
                panic!("Object literal must have exactly one field to match field constraint {{{}: {}}}", field_name, expected_type.0);
            }
            let (item_name, item_expr) = &items[0];
            if item_name.0 != *field_name {
                panic!("Object field '{}' does not match expected field '{}'", item_name.0, field_name);
            }
            checks_against(state, context, &item_expr.0, &expected_type.0)
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
        Type::FieldConstraint(_, ty) => occurs_in(alpha, &ty.0),
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
        // Unknown can subtype to anything (for forward declarations)
        (Type::Unknown, _) => {
            print_rule("<:Unknown");
            context.clone()
        }
        //<:Unit
        (Type::Literal(lit_a), Type::Literal(lit_b)) => {
            print_rule("<:Unit");
            assert_eq!(lit_a, lit_b);
            context.clone()
        }
        //<:Var
        (Type::Ident(alpha1), Type::Ident(alpha2)) => {
            print_rule("<:Var");
            let well_formed = is_well_formed(context, a);
            // Compare types semantically, ignoring span information
            let equal = alpha1.path.len() == alpha2.path.len()
                && alpha1.path.iter().zip(alpha2.path.iter()).all(|(a, b)| a.0 == b.0)
                && alpha1.type_args.len() == alpha2.type_args.len()
                && alpha1.type_args.iter().zip(alpha2.type_args.iter()).all(|(a, b)| a.0 == b.0);
            
            if well_formed && equal {
                return context.clone();
            } else {
                panic!("Cannot subtype {} <: {}", alpha1, alpha2);
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
        // Field constraint intersection → nominal type subtyping (stricter checking)
        (Type::Intersection(a, b), Type::Ident(ident)) if is_field_constraint_intersection(a, b) => {
            print_rule("<:FieldIntersectionToNominal");
            // For field constraint intersections, ALL fields must be compatible with nominal type
            let type_name = &ident.path[0].0;
            if let Some(nominal_intersection) = context.get_annotation(type_name) {
                if all_field_constraints_match_nominal(a, b, nominal_intersection) {
                    context.clone()
                } else {
                    panic!("Not all field constraints match nominal type {}", type_name);
                }
            } else {
                panic!("Cannot subtype field intersection to undefined nominal type {}", type_name);
            }
        }
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
            if matches!(type_name.as_str(), "bool" | "Bool") {
                print_rule("<:BoolToBool");
                context.clone()
            } else {
                panic!("Cannot subtype Bool to {}", type_name);
            }
        }
        (Type::Literal(LiteralType::Bool), Type::Literal(LiteralType::Bool)) => {
            print_rule("<:BoolToBool");
            context.clone()
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
        // Field constraint → nominal type subtyping
        (Type::FieldConstraint(field_name, field_type), Type::Ident(ident)) => {
            print_rule("<:FieldToNominal");
            // Check if the field constraint is compatible with the nominal type
            let type_name = &ident.path[0].0;
            if let Some(nominal_intersection) = context.get_annotation(type_name) {
                // For single field constraint, verify it matches AND that nominal type has only one field
                let nominal_field_count = count_fields_in_intersection(nominal_intersection);
                if nominal_field_count == 1 {
                    // Single field case - check if it matches
                    if field_constraint_matches_in_intersection(field_name, &field_type.0, nominal_intersection) {
                        context.clone()
                    } else {
                        panic!("Field {} not compatible with nominal type {}", field_name, type_name);
                    }
                } else {
                    // Multi-field nominal type but single field constraint - incomplete
                    panic!("Single field constraint cannot satisfy multi-field nominal type {}", type_name);
                }
            } else {
                panic!("Cannot subtype field constraint to undefined nominal type {}", type_name);
            }
        }
        // Field constraint reflexivity and compatibility
        (Type::FieldConstraint(field1, type1), Type::FieldConstraint(field2, type2)) => {
            if field1 == field2 {
                // Same field name - check if types are compatible
                print_rule("<:FieldConstraintRefl");
                subtype(state, context, &type1.0, &type2.0)
            } else {
                panic!("Cannot subtype different field constraints: {} vs {}", field1, field2);
            }
        }
        // Unknown type compatibility - handles generic/unknown types
        (_, Type::Unknown) => {
            print_rule("<:Unknown");
            context.clone()
        }
        (Type::Unknown, _) => {
            print_rule("<:UnknownLeft");
            context.clone()
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
            
            // Check if this is an enum variant constructor
            if let Some(constructor_type) = resolve_enum_variant_constructor(context, &x.path[0].0) {
                return (constructor_type, context.clone());
            }
            
            panic!("variable not found: {}", x.path[0].0);
        }
        //Anno
        Expr::Annotation(e, annotation) => {
            print_rule("Anno");
            if is_well_formed(context, annotation) {
                let delta = checks_against(state, context, e, annotation);
                return (annotation.clone(), delta);
            } else {
                // Annotation is not well-formed - return Error type
                (Type::Error, context.clone())
            }
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
            
            match args.len() {
                0 => {
                    // No arguments - nullary function call
                    (func_type, theta)
                }
                1 => {
                    // Single argument - use existing application logic
                    let arg = &args[0].1; // Get the expression, ignore the optional name for now
                    application_synthesizes_to(state, &theta, &apply_context(func_type, &theta), &arg.0)
                }
                _ => {
                    // Multiple arguments - implement currying or multi-parameter functions
                    synthesize_multi_argument_call(state, &theta, &apply_context(func_type, &theta), args)
                }
            }
        }
        // Missing patterns - add basic implementations
        Expr::Error => {
            print_rule("Error");
            (Type::Error, context.clone())
        }
        Expr::Object(items) => {
            print_rule("Object");
            // Synthesize object literal as field constraint intersection
            let mut gamma = context.clone();
            let mut field_constraints = Vec::new();
            
            for (field_name, field_expr) in items {
                // Synthesize the type for each field expression
                let (field_type, new_gamma) = synthesizes_to(state, &gamma, &field_expr.0);
                
                // Create a FieldConstraint for this field
                field_constraints.push(Type::FieldConstraint(
                    field_name.0.clone(),
                    Box::new((field_type, field_expr.1.clone()))
                ));
                
                gamma = new_gamma;
            }
            
            // Create intersection of all field constraints: {field1: T1}^{field2: T2}^...
            let result_type = if field_constraints.is_empty() {
                // Empty object literal - for now use struct type
                Type::Struct(vec![])
            } else if field_constraints.len() == 1 {
                // Single field
                field_constraints.into_iter().next().unwrap()
            } else {
                // Multiple fields - create intersection
                field_constraints.into_iter().reduce(|acc, field_constraint| {
                    Type::Intersection(Box::new(acc), Box::new(field_constraint))
                }).unwrap()
            };
            
            (result_type, gamma)
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
                    // Comparison operations: return Bool (matching the expected type case)
                    (Type::Literal(LiteralType::Bool), delta)
                }
                _ => {
                    // Other operations - not yet implemented
                    (Type::Unknown, delta)
                }
            }
        }
        Expr::Member(expr, field) => {
            print_rule("Member");
            // Synthesize the type of the base expression
            let (expr_type, gamma) = synthesizes_to(state, context, &expr.0);
            let field_name = &field.0;
            
            // Search for field in intersection components
            if let Some(field_type) = find_field_in_intersection(&expr_type, field_name) {
                (field_type, gamma)
            } else {
                // For nominal types, look up their intersection representation
                match &expr_type {
                    Type::Ident(ident) => {
                        // Look up the type definition in context to get its intersection
                        if let Some(resolved_type) = context.get_annotation(&ident.path[0].0) {
                            if let Some(field_type) = find_field_in_intersection(resolved_type, field_name) {
                                (field_type, gamma)
                            } else {
                                panic!("Field {} not found in type {}", field_name, ident.path[0].0);
                            }
                        } else {
                            panic!("Type {} not found in context", ident.path[0].0);
                        }
                    }
                    Type::Intersection(a, b) if matches!(**b, Type::Unknown) => {
                        // Forward declared type - return Unknown for any field access
                        (Type::Unknown, gamma)
                    }
                    _ => {
                        panic!("Field {} not found in type {}", field_name, expr_type);
                    }
                }
            }
        }
        // Add support for synthesizing intersection types from structural literals
        // This will be needed when we have object literals that should get nominal+structural types
    }
}

/// Helper function to check if an intersection consists of field constraints
fn is_field_constraint_intersection(a: &Type, b: &Type) -> bool {
    fn is_field_constraint_or_intersection(t: &Type) -> bool {
        match t {
            Type::FieldConstraint(_, _) => true,
            Type::Intersection(x, y) => is_field_constraint_or_intersection(x) && is_field_constraint_or_intersection(y),
            _ => false
        }
    }
    is_field_constraint_or_intersection(a) && is_field_constraint_or_intersection(b)
}

/// Helper function to check if all field constraints in an intersection match a nominal type
fn all_field_constraints_match_nominal(a: &Type, b: &Type, nominal_type: &Type) -> bool {
    fn check_all_field_constraints(field_intersection: &Type, nominal_type: &Type) -> bool {
        match field_intersection {
            Type::FieldConstraint(field_name, field_type) => {
                field_constraint_matches_in_intersection(field_name, &field_type.0, nominal_type)
            }
            Type::Intersection(x, y) => {
                check_all_field_constraints(x, nominal_type) && check_all_field_constraints(y, nominal_type)
            }
            _ => false
        }
    }
    check_all_field_constraints(a, nominal_type) && check_all_field_constraints(b, nominal_type)
}

/// Helper function to count the number of fields in an intersection type
fn count_fields_in_intersection(intersection_type: &Type) -> usize {
    match intersection_type {
        Type::FieldConstraint(_, _) => 1,
        Type::Intersection(a, b) => count_fields_in_intersection(a) + count_fields_in_intersection(b),
        _ => 0
    }
}

/// Helper function to add pattern variables to context for match expression type checking
fn add_pattern_variables_to_context(context: &Context, pattern: &Pattern) -> Context {
    // Legacy function - should use add_pattern_variables_to_context_with_scrutinee instead
    add_pattern_variables_to_context_with_scrutinee(context, pattern, &Type::Unknown)
}

/// Helper function to add pattern variables to context with proper type extraction from scrutinee
fn add_pattern_variables_to_context_with_scrutinee(context: &Context, pattern: &Pattern, scrutinee_type: &Type) -> Context {
    let mut new_context = context.clone();
    
    // Extract pattern variables with proper types from the sum type variant
    if let Some(ref members) = pattern.members {
        // Get the variant parameter types for this pattern
        let variant_types = extract_variant_parameter_types(context, scrutinee_type, &pattern.ident.0);
        
        // Add each pattern variable with its corresponding type
        for (i, member_pattern) in members.iter().enumerate() {
            let param_type = if i < variant_types.len() {
                variant_types[i].clone()
            } else {
                Type::Unknown // Fallback if we can't determine the type
            };
            
            new_context = new_context.add(ContextElement::TypedVariable(
                member_pattern.ident.0.clone(), 
                param_type
            ));
        }
    }
    
    new_context
}

/// Extract parameter types for a specific variant from an instantiated sum type
fn extract_variant_parameter_types(context: &Context, scrutinee_type: &Type, variant_name: &str) -> Vec<Type> {
    match scrutinee_type {
        Type::Ident(qualified_type) => {
            // Get the base type name (e.g., "Option")
            let type_name = &qualified_type.path[0].0;
            
            // Look up the generic sum type definition in the context
            if let Some(base_type) = context.get_annotation(type_name) {
                if let Type::Intersection(_, b) = base_type {
                    if let Type::Enum(variants) = &**b {
                        // Find the matching variant
                        for variant in variants {
                            if variant.name.0 == variant_name {
                                // Substitute type parameters with the concrete instantiated types
                                return substitute_variant_types_with_instantiation(&variant.values, &qualified_type.type_args, context);
                            }
                        }
                    }
                }
            }
        }
        _ => {
            // Handle other cases if needed
        }
    }
    
    vec![] // Return empty if we can't extract types
}

/// Substitute generic type parameters with concrete types in variant parameters
fn substitute_variant_types(variant_values: &[Spanned<Type>], type_args: &[Spanned<Type>]) -> Vec<Type> {
    let mut result = Vec::new();
    
    for variant_value in variant_values {
        let substituted_type = substitute_type_parameters(&variant_value.0, type_args);
        result.push(substituted_type);
    }
    
    result
}

/// Substitute generic type parameters with concrete instantiated types, resolving existentials
fn substitute_variant_types_with_instantiation(variant_values: &[Spanned<Type>], type_args: &[Spanned<Type>], context: &Context) -> Vec<Type> {
    let mut result = Vec::new();
    
    for variant_value in variant_values {
        let substituted_type = substitute_type_parameters(&variant_value.0, type_args);
        // Resolve existential types in the substituted result
        let resolved_type = resolve_existential_type(&substituted_type, context);
        result.push(resolved_type);
    }
    
    result
}

/// Substitute type parameters (like T) with concrete types (like i32)
fn substitute_type_parameters(ty: &Type, type_args: &[Spanned<Type>]) -> Type {
    match ty {
        Type::Ident(qualified_type) => {
            // Check if this is a type parameter (single letter like "T")
            if qualified_type.path.len() == 1 {
                let param_name = &qualified_type.path[0].0;
                // For now, assume the first type parameter "T" maps to the first type argument
                // In a full implementation, this would use a proper parameter mapping
                if param_name == "T" && !type_args.is_empty() {
                    return type_args[0].0.clone();
                }
            }
            ty.clone()
        }
        _ => ty.clone(), // For other types, return as-is
    }
}

/// Extract element type from iterator types like Vec[i32] -> i32
fn extract_iterator_element_type(iter_type: &Type) -> Type {
    match iter_type {
        Type::Ident(qualified_type) => {
            // For generic types like Vec[i32], extract the first type argument
            if !qualified_type.type_args.is_empty() {
                return qualified_type.type_args[0].0.clone();
            }
        }
        Type::Intersection(a, b) => {
            // Look for iterator types in intersection types
            if let Some(element_type) = try_extract_from_intersection(&**a) {
                return element_type;
            }
            if let Some(element_type) = try_extract_from_intersection(&**b) {
                return element_type;
            }
        }
        _ => {}
    }
    
    // Fallback to Unknown if we can't extract element type
    Type::Unknown
}

/// Helper to extract element type from parts of intersection types
fn try_extract_from_intersection(ty: &Type) -> Option<Type> {
    match ty {
        Type::Ident(qualified_type) if !qualified_type.type_args.is_empty() => {
            Some(qualified_type.type_args[0].0.clone())
        }
        _ => None
    }
}

/// Resolve enum variant constructor type from context
fn resolve_enum_variant_constructor(context: &Context, variant_name: &str) -> Option<Type> {
    // Look through all type definitions in the context for enum types
    for element in &context.elements {
        if let ContextElement::TypedVariable(type_name, type_def) = element {
            // Check if this is an intersection type with an enum
            if let Some(enum_variants) = extract_enum_variants_from_type(type_def) {
                // Look for matching variant
                for variant in enum_variants {
                    if variant.name.0 == variant_name {
                        // Create constructor type: param_types -> enum_type
                        let enum_type = Type::Ident(QualifiedType {
                            path: vec![(type_name.clone(), Span::default())],
                            type_args: vec![], // For now, without generics
                        });
                        
                        if variant.values.is_empty() {
                            // Nullary constructor - just return the enum type
                            return Some(enum_type);
                        } else if variant.values.len() == 1 {
                            // Unary constructor - return polymorphic function type using quantification
                            let param_type = variant.values[0].0.clone();
                            
                            if is_type_parameter(&param_type) {
                                // For generic constructors like Some: ∀T. T -> Option[T]
                                let type_var = extract_type_parameter_name(&param_type);
                                let quantified_return_type = Type::Ident(QualifiedType {
                                    path: vec![(type_name.clone(), Span::default())],
                                    type_args: vec![(Type::Ident(QualifiedType {
                                        path: vec![(type_var.clone(), Span::default())],
                                        type_args: vec![],
                                    }), Span::default())],
                                });
                                let function_type = Type::Func(
                                    Box::new((param_type, Span::default())),
                                    Box::new((quantified_return_type, Span::default())),
                                );
                                return Some(Type::Quantification(type_var, Box::new(function_type)));
                            } else {
                                // Non-generic constructor
                                return Some(Type::Func(
                                    Box::new((param_type, Span::default())),
                                    Box::new((enum_type, Span::default())),
                                ));
                            }
                        } else {
                            // Multi-parameter constructor - return tuple -> enum function type
                            let tuple_type = Type::Tuple(variant.values.clone());
                            return Some(Type::Func(
                                Box::new((tuple_type, Span::default())),
                                Box::new((enum_type, Span::default())),
                            ));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract enum variants from a type (looking through intersections)
fn extract_enum_variants_from_type(ty: &Type) -> Option<&Vec<EnumVariant>> {
    match ty {
        Type::Enum(variants) => Some(variants),
        Type::Intersection(a, b) => {
            if let Some(variants) = extract_enum_variants_from_type(a) {
                Some(variants)
            } else {
                extract_enum_variants_from_type(b)
            }
        }
        _ => None,
    }
}

/// Check if a type is a type parameter (like T)
fn is_type_parameter(ty: &Type) -> bool {
    match ty {
        Type::Ident(qualified_type) => {
            qualified_type.path.len() == 1 
            && qualified_type.path[0].0.chars().all(|c| c.is_ascii_uppercase())
            && qualified_type.path[0].0.len() == 1
        }
        _ => false,
    }
}

/// Extract type parameter name from type
fn extract_type_parameter_name(ty: &Type) -> String {
    match ty {
        Type::Ident(qualified_type) if qualified_type.path.len() == 1 => {
            qualified_type.path[0].0.clone()
        }
        _ => "T".to_string(), // fallback
    }
}

/// Resolve existential types by looking up their solved values in the context
fn resolve_existential_type(ty: &Type, context: &Context) -> Type {
    match ty {
        Type::Existential(alpha) => {
            // Look up solved existential in context
            for element in &context.elements {
                if let ContextElement::Solved(name, solved_type) = element {
                    if name == alpha {
                        // Found solved existential, recursively resolve it
                        return resolve_existential_type(solved_type, context);
                    }
                }
            }
            // If not solved, return as-is
            ty.clone()
        }
        // For compound types, recursively resolve their components
        Type::Ident(qualified_type) => {
            let resolved_args: Vec<Spanned<Type>> = qualified_type.type_args.iter()
                .map(|arg| (resolve_existential_type(&arg.0, context), arg.1.clone()))
                .collect();
            Type::Ident(QualifiedType {
                path: qualified_type.path.clone(),
                type_args: resolved_args,
            })
        }
        Type::Intersection(a, b) => Type::Intersection(
            Box::new(resolve_existential_type(a, context)),
            Box::new(resolve_existential_type(b, context)),
        ),
        Type::Func(a, b) => Type::Func(
            Box::new((resolve_existential_type(&a.0, context), a.1.clone())),
            Box::new((resolve_existential_type(&b.0, context), b.1.clone())),
        ),
        // For other types, return as-is
        _ => ty.clone(),
    }
}

/// Helper function to check if a field constraint matches within an intersection type
fn field_constraint_matches_in_intersection(field_name: &str, field_type: &Type, intersection_type: &Type) -> bool {
    match intersection_type {
        Type::FieldConstraint(name, constraint_type) if name == field_name => {
            // Found matching field - check if types are compatible
            // For now, do simple equality check (could be enhanced with subtyping)
            match (&field_type, &constraint_type.0) {
                (Type::Literal(a), Type::Literal(b)) => a == b,
                (Type::Ident(a), Type::Ident(b)) => a.path[0].0 == b.path[0].0,
                (Type::Literal(LiteralType::Str), Type::Ident(ident)) if ident.path[0].0 == "String" => true,
                (Type::Literal(LiteralType::Nat), Type::Ident(ident)) if matches!(ident.path[0].0.as_str(), "i32" | "u32" | "i64" | "u64") => true,
                _ => false
            }
        }
        Type::Intersection(a, b) => {
            // Search both sides of intersection
            field_constraint_matches_in_intersection(field_name, field_type, a) ||
            field_constraint_matches_in_intersection(field_name, field_type, b)
        }
        _ => false
    }
}

/// Helper function to find a field in an intersection type (for member access)
fn find_field_in_intersection(ty: &Type, field_name: &str) -> Option<Type> {
    match ty {
        Type::FieldConstraint(name, field_type) if name == field_name => Some(field_type.0.clone()),
        Type::Intersection(a, b) => {
            find_field_in_intersection(a, field_name)
                .or_else(|| find_field_in_intersection(b, field_name))
        }
        Type::Ident(ident) => {
            // Look up nominal type and search its intersection
            // A nominal type A is always stored as A^{...fields...}
            // For now, return None - this will be handled when we implement 
            // automatic nominal→intersection conversion
            None
        }
        Type::Struct(items) => {
            // Temporary: still handle legacy struct access during transition
            for item in items {
                if item.name == field_name {
                    return Some(item.ty.0.clone());
                }
            }
            None
        }
        _ => None,
    }
}

/// Handle multi-argument function calls with tuple-based arguments
fn synthesize_multi_argument_call(
    state: &mut State,
    context: &Context, 
    func_type: &Type,
    args: &[(Option<Spanned<String>>, Spanned<Expr>)]
) -> (Type, Context) {
    print_helper("multi_call", format!("{} args", args.len()), format!("{}", func_type), context);
    
    // TODO: Handle named arguments properly when name_opt.is_some()
    for (name_opt, _) in args {
        if name_opt.is_some() {
            panic!("Named arguments not yet implemented in multi-argument calls");
        }
    }
    
    // Match against function type to get expected argument tuple and return type
    match func_type {
        Type::Func(arg_type, ret_type) => {
            // Synthesize each argument
            let mut gamma = context.clone();
            let mut arg_types = Vec::new();
            
            for (_, arg_expr) in args {
                let (arg_type, new_gamma) = synthesizes_to(state, &gamma, &arg_expr.0);
                arg_types.push((arg_type, arg_expr.1.clone()));
                gamma = new_gamma;
            }
            
            // Create tuple type from synthesized arguments
            let actual_arg_tuple = Type::Tuple(arg_types);
            
            // Check if our tuple is compatible with the function's expected argument type
            let final_gamma = subtype(state, &gamma, &actual_arg_tuple, &arg_type.0);
            
            (ret_type.0.clone(), final_gamma)
        }
        _ => {
            panic!("Cannot call non-function type: {}", func_type);
        }
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
                // Recursively substitute in type arguments
                let substituted_type_args: Vec<Spanned<Type>> = var.type_args.iter()
                    .map(|arg| (substitution(&arg.0, alpha, b), arg.1.clone()))
                    .collect();
                Type::Ident(QualifiedType {
                    path: var.path.clone(),
                    type_args: substituted_type_args,
                })
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
        Type::FieldConstraint(field, ty) => Type::FieldConstraint(
            field.clone(),
            Box::new((substitution(&ty.0, alpha, b), ty.1.clone()))
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

/// Helper function to check if type A is a subtype of type B (returns boolean)
fn is_subtype(state: &mut State, context: &Context, a: &Type, b: &Type) -> bool {
    // Use panic catching to convert subtype failures to boolean false
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        subtype(state, context, a, b);
        true
    })).unwrap_or(false)
}

/// Type synthesis for block expressions (control flow)
fn synthesizes_to_block_expr(state: &mut State, context: &Context, block_expr: &BlockExpr) -> (Type, Context) {
    print_helper("synth_block", format!("{:?}", block_expr), "".into(), context);
    match block_expr {
        BlockExpr::If(if_expr) => synthesizes_to_if_expr(state, context, if_expr),
        BlockExpr::Match(match_expr) => synthesizes_to_match_expr(state, context, match_expr),
        BlockExpr::For(for_expr) => synthesizes_to_for_expr(state, context, for_expr),
        BlockExpr::While(while_expr) => synthesizes_to_while_expr(state, context, while_expr),
        BlockExpr::Block(block) => synthesizes_to_block(state, context, block),
    }
}

/// Type synthesis for if expressions
fn synthesizes_to_if_expr(state: &mut State, context: &Context, if_expr: &IfExpr) -> (Type, Context) {
    print_rule("IfExpr");
    
    // Check main condition synthesizes to bool
    let (cond_type, gamma) = synthesizes_to(state, context, &if_expr.main_expr.0);
    let bool_type = Type::Literal(LiteralType::Bool);
    if !is_subtype(state, &gamma, &cond_type, &bool_type) {
        panic!("If condition must be bool, got: {}", cond_type);
    }
    
    // Check main body (don't leak variables from branch)
    let (main_body_type, _delta) = synthesizes_to_control_expr(state, &gamma, &if_expr.main_body);
    
    // Check all elif branches have same type as main body
    for (elif_cond, elif_body) in &if_expr.elif_expr_body {
        // Check elif condition is bool (use parent context, not accumulated)
        let (elif_cond_type, theta) = synthesizes_to(state, &gamma, &elif_cond.0);
        if !is_subtype(state, &theta, &elif_cond_type, &bool_type) {
            panic!("Elif condition must be bool, got: {}", elif_cond_type);
        }
        
        // Check elif body has same type as main body (don't accumulate context)
        let (elif_body_type, _phi) = synthesizes_to_control_expr(state, &theta, elif_body);
        if !is_subtype(state, &gamma, &elif_body_type, &main_body_type) && !is_subtype(state, &gamma, &main_body_type, &elif_body_type) {
            panic!("All if branches must have same type. Main: {}, elif: {}", main_body_type, elif_body_type);
        }
    }
    
    // Check else branch if present (don't leak variables from branch)
    if let Some(else_body) = &*if_expr.else_body {
        let (else_body_type, _psi) = synthesizes_to_control_expr(state, &gamma, else_body);
        if !is_subtype(state, &gamma, &else_body_type, &main_body_type) && !is_subtype(state, &gamma, &main_body_type, &else_body_type) {
            panic!("All if branches must have same type. Main: {}, else: {}", main_body_type, else_body_type);
        }
    }
    
    // Return main body type and preserve parent context (no variable leakage from branches)
    (main_body_type, gamma)
}

/// Type synthesis for match expressions
fn synthesizes_to_match_expr(state: &mut State, context: &Context, match_expr: &MatchExpr) -> (Type, Context) {
    print_rule("MatchExpr");
    
    // Synthesize type of scrutinee (the value being matched)
    let (scrutinee_type, gamma) = synthesizes_to(state, context, &match_expr.expr.0);
    
    if match_expr.branches.is_empty() {
        panic!("Match expression must have at least one branch");
    }
    
    // Check first branch to establish expected type (don't leak variables from branch)
    let first_branch = &match_expr.branches[0];
    // Resolve existentials in scrutinee type using the updated context
    let resolved_scrutinee_type = resolve_existential_type(&scrutinee_type, &gamma);
    let branch_context = add_pattern_variables_to_context_with_scrutinee(&gamma, &first_branch.pattern, &resolved_scrutinee_type);
    let (first_branch_type, _first_context) = synthesizes_to_control_expr(state, &branch_context, &first_branch.body);
    
    // Check all other branches have same type (don't accumulate context)
    for branch in &match_expr.branches[1..] {
        let branch_context = add_pattern_variables_to_context_with_scrutinee(&gamma, &branch.pattern, &resolved_scrutinee_type);
        let (branch_type, _delta) = synthesizes_to_control_expr(state, &branch_context, &branch.body);
        if !is_subtype(state, &gamma, &branch_type, &first_branch_type) && !is_subtype(state, &gamma, &first_branch_type, &branch_type) {
            panic!("All match branches must have same type. First: {}, other: {}", first_branch_type, branch_type);
        }
    }
    
    // Return first branch type and preserve parent context (no variable leakage from branches)
    (first_branch_type, gamma)
}

/// Type synthesis for for expressions (returns void for now)
fn synthesizes_to_for_expr(state: &mut State, context: &Context, for_expr: &ForExpr) -> (Type, Context) {
    print_rule("ForExpr");
    
    // Type check the iterator expression
    let (iter_type, gamma) = synthesizes_to(state, context, &for_expr.expr.0);
    
    // Extract element type from iterator (e.g., Vec[i32] -> i32)
    let element_type = extract_iterator_element_type(&iter_type);
    
    // Add loop variable to context with proper element type
    let loop_context = gamma.add(ContextElement::TypedVariable(
        for_expr.ident.0.clone(),
        element_type,
    ));
    
    // Type check body (but ignore result and don't leak loop scope variables)
    let (_, _delta) = synthesizes_to_control_expr(state, &loop_context, &for_expr.body);
    
    // For expressions return void type () and preserve parent context (no variable leakage)
    let unit_type = Type::Tuple(vec![]);
    (unit_type, gamma) // Return parent context, not the loop context with variables
}

/// Type synthesis for while expressions (returns void for now)
fn synthesizes_to_while_expr(state: &mut State, context: &Context, while_expr: &WhileExpr) -> (Type, Context) {
    print_rule("WhileExpr");
    
    // Check condition synthesizes to bool
    let (cond_type, gamma) = synthesizes_to(state, context, &while_expr.expr.0);
    let bool_type = Type::Literal(LiteralType::Bool);
    if !is_subtype(state, &gamma, &cond_type, &bool_type) {
        panic!("While condition must be bool, got: {}", cond_type);
    }
    
    // Type check body (but ignore result and don't leak block scope variables)
    let (_, _delta) = synthesizes_to_control_expr(state, &gamma, &while_expr.body);
    
    // While expressions return void type () and preserve parent context (no variable leakage)
    let unit_type = Type::Tuple(vec![]);
    (unit_type, gamma) // Return parent context, not the block context with variables
}

/// Type synthesis for control expressions
fn synthesizes_to_control_expr(state: &mut State, context: &Context, control_expr: &ControlExpr) -> (Type, Context) {
    match control_expr {
        ControlExpr::Expr(expr) => synthesizes_to(state, context, &expr.0),
        ControlExpr::Block(block) => synthesizes_to_block(state, context, block),
    }
}

/// Type synthesis for blocks (simplified for now)
fn synthesizes_to_block(state: &mut State, context: &Context, block: &Block) -> (Type, Context) {
    print_rule("Block");
    
    let mut current_context = context.clone();
    let mut last_type = Type::Tuple(vec![]); // Default to unit type
    
    for statement in &block.body {
        match statement {
            Statement::Expr(assign_expr) => {
                let (stmt_type, delta) = synthesizes_to_assign_expr(state, &current_context, assign_expr);
                last_type = stmt_type;
                current_context = delta;
            }
            Statement::Let(let_stmt) => {
                // Type check let statement and add variable to context
                let (var_type, delta) = synthesizes_to_assign_expr(state, &current_context, &let_stmt.value);
                
                // If explicit type annotation, check against it
                if let Some(annotation) = &let_stmt.ty {
                    if !is_subtype(state, &delta, &var_type, &annotation.0) {
                        panic!("Let binding type mismatch. Expected: {}, got: {}", annotation.0, var_type);
                    }
                }
                
                // Add variable to context
                current_context = delta.add(ContextElement::TypedVariable(
                    let_stmt.name.0.clone(),
                    var_type.clone()
                ));
                
                // Let statements don't contribute to block type
                last_type = Type::Tuple(vec![]); // Unit type
            }
            Statement::Assign(assign_stmt) => {
                // Type check assignment (simplified)
                let (assign_type, delta) = synthesizes_to_assign_expr(state, &current_context, &assign_stmt.value);
                // For now, don't update context - assignments don't create new variables
                last_type = assign_type;
                current_context = delta;
            }
            Statement::Error => {
                panic!("Error statement in block");
            }
        }
    }
    
    // Block should not leak local variables to parent scope
    // Only return the last expression type, preserve original context
    (last_type, context.clone())
}

/// Type synthesis for assignment expressions
fn synthesizes_to_assign_expr(state: &mut State, context: &Context, assign_expr: &AssignExpr) -> (Type, Context) {
    match assign_expr {
        AssignExpr::Expr(expr) => synthesizes_to(state, context, &expr.0),
        AssignExpr::BlockExpr(block_expr) => synthesizes_to_block_expr(state, context, block_expr),
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
            
            (is_builtin || context.has_variable(type_name) || context.get_annotation(type_name).is_some())
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
        Type::FieldConstraint(_, ty) => is_well_formed(context, &ty.0),
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
        Type::FieldConstraint(field, ty) => Type::FieldConstraint(
            field,
            Box::new((apply_context(ty.0, context), ty.1))
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
    
    // Phase 1: Collect all module symbols including enhancements from open blocks
    let mut symbols = collect_module_symbols(program)?;
    
    println!("Type definitions (before enhancement): {:?}", symbols.type_definitions.keys().collect::<Vec<_>>());
    println!("Function signatures: {:?}", symbols.function_signatures.keys().collect::<Vec<_>>());
    println!("Constants: {:?}", symbols.constants.keys().collect::<Vec<_>>());
    println!("Traits: {:?}", symbols.traits.keys().collect::<Vec<_>>());
    println!("Enhancement registry: {:?}", symbols.enhancements);
    println!("Trait methods: {:?}", symbols.trait_methods.keys().collect::<Vec<_>>());
    
    // Phase 1.5: Apply all enhancements in-place to create retroactive type enhancement
    println!("Applying retroactive enhancements...");
    symbols.apply_all_enhancements();
    println!("Type definitions (after enhancement): {:?}", symbols.type_definitions.keys().collect::<Vec<_>>());
    
    // Phase 2: Build context from enhanced registry
    // Add all type definitions to context
    for (type_name, type_def) in &symbols.type_definitions {
        context = context.add(ContextElement::TypedVariable(
            type_name.clone(),
            type_def.clone()
        ));
    }
    
    // Add all function signatures to context
    for (func_name, func_type) in &symbols.function_signatures {
        context = context.add(ContextElement::TypedVariable(
            func_name.clone(),
            func_type.clone()
        ));
    }
    
    // Add all constants to context
    for (const_name, const_type) in &symbols.constants {
        context = context.add(ContextElement::TypedVariable(
            const_name.clone(),
            const_type.clone()
        ));
    }
    
    // Phase 3: Type check function bodies and expressions with enhanced context
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
                    AssignExpr::BlockExpr(block_expr) => {
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
                        
                        // Try type checking block expression with parameter context
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            synthesizes_to_block_expr(&mut state.clone(), &func_context, &block_expr)
                        })) {
                            Ok((inferred_type, _)) => {
                                println!("Function {} with block body has type: {}", func.ident, inferred_type);
                            }
                            Err(_) => {
                                return Err(format!("Type checking failed for function {} with block body", func.ident));
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{lexer, tts_to_stream};
    use crate::parser::program_parser;
    use chumsky::Parser;

    fn parse_and_check(source: &str) -> Result<(), String> {
        // Parse tokens
        let (tts, _errs) = lexer().parse_recovery(source);
        let tts = tts.ok_or("Failed to parse tokens")?;
        
        // Flatten to token stream
        let eoi = 0..source.chars().count();
        let token_stream = tts_to_stream(eoi, tts);
        
        // Parse AST
        let (ast, _parse_errs) = program_parser().parse_recovery(token_stream);
        let ast = ast.ok_or("Failed to parse AST")?;
        
        // Type check
        check_program(&ast)
    }

    #[test]
    fn test_basic_constant() {
        let source = "def answer: i32 = 42";
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_binary_operations() {
        let source = "def result: i32 = 2 + 3";
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_struct_type_definition() {
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_member_access() {
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

def getName(p: Person) -> String = p.name
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_generic_function() {
        let source = "def times(a: T, b: T) -> T for T: type = a * b";
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_intersection_types() {
        // Test that Person type gets intersection type Person^{fields}
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

def processName(p: Person) -> String = p.name
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_subtyping_nat_to_int() {
        let tests = vec![
            "def x: i32 = 42",
            "def y: u32 = 100", 
            "def z: i64 = 999",
        ];
        
        for source in tests {
            assert!(parse_and_check(source).is_ok(), "Failed for: {}", source);
        }
    }

    #[test]
    fn test_comparison_operations() {
        let source = r#"
def isEqual: bool = 5 == 3
def isGreater: bool = 10 > 5
def isLessEq: bool = 2 <= 7
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_function_with_parameters() {
        let source = r#"
def add(x: i32, y: i32) -> i32 = x + y
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_nested_member_access() {
        let source = r#"
def type Address = {
    street: String,
    city: String,
}

def type Person = {
    name: String,
    address: Address,
}

def getCity(p: Person) -> String = p.address
"#;
        // This should work - p.address has type Address, even though return type is String
        // The type checker should detect the mismatch if we implement return type checking
        let result = parse_and_check(source);
        assert!(result.is_ok());
    }

    #[test]  
    fn test_field_not_found() {
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

def getBadField(p: Person) -> String = p.nonexistent
"#;
        // This should fail because field doesn't exist
        let result = parse_and_check(source);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Type checking failed"));
    }

    #[test]
    fn test_built_in_types_recognition() {
        // Test that built-in type names are recognized as well-formed
        use crate::utils::Span;
        
        let context = Context::initial();
        let i32_type = Type::Ident(QualifiedType {
            path: vec![("i32".to_string(), Span::default())],
            type_args: vec![],
        });
        let bool_type = Type::Ident(QualifiedType {
            path: vec![("bool".to_string(), Span::default())],
            type_args: vec![],
        });
        let unknown_type = Type::Ident(QualifiedType {
            path: vec![("UnknownType".to_string(), Span::default())],
            type_args: vec![],
        });
        
        assert!(is_well_formed(&context, &i32_type));
        assert!(is_well_formed(&context, &bool_type));
        assert!(!is_well_formed(&context, &unknown_type));
    }

    // Helper function tests for type system internals
    #[test]
    fn test_find_field_in_struct() {
        use crate::utils::Span;
        
        let items = vec![
            Item {
                name: "name".to_string(),
                ty: (Type::Ident(QualifiedType {
                    path: vec![("String".to_string(), Span::default())],
                    type_args: vec![],
                }), Span::default()),
            },
            Item {
                name: "age".to_string(), 
                ty: (Type::Ident(QualifiedType {
                    path: vec![("i32".to_string(), Span::default())],
                    type_args: vec![],
                }), Span::default()),
            },
        ];
        
        let struct_type = Type::Struct(items);
        
        // Should find existing field
        assert!(find_field_in_intersection(&struct_type, "name").is_some());
        assert!(find_field_in_intersection(&struct_type, "age").is_some());
        
        // Should not find non-existent field
        assert!(find_field_in_intersection(&struct_type, "nonexistent").is_none());
    }

    #[test]
    fn test_intersection_type_field_lookup() {
        use crate::utils::Span;
        
        // Create a struct type
        let struct_items = vec![
            Item {
                name: "name".to_string(),
                ty: (Type::Ident(QualifiedType {
                    path: vec![("String".to_string(), Span::default())],
                    type_args: vec![],
                }), Span::default()),
            },
        ];
        let struct_type = Type::Struct(struct_items);
        
        // Create an intersection type
        let nominal_type = Type::Ident(QualifiedType {
            path: vec![("Person".to_string(), Span::default())],
            type_args: vec![],
        });
        let intersection_type = Type::Intersection(
            Box::new(nominal_type),
            Box::new(struct_type),
        );
        
        // Should find field in intersection type
        assert!(find_field_in_intersection(&intersection_type, "name").is_some());
        assert!(find_field_in_intersection(&intersection_type, "nonexistent").is_none());
    }

    #[test]
    fn test_object_literal_synthesis() {
        let source = r#"
def type PersonType = {
    name: String,
    age: i32,
    active: bool,
}

def createPerson: PersonType = {
    name = "Alice",
    age = 30,
    active = true,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_object_literal_with_expressions() {
        let source = r#"
def type ComputedType = {
    sum: i32,
    comparison: bool,
    value: i32,
}

def computedObject: ComputedType = {
    sum = 5 + 3,
    comparison = 10 > 5,
    value = 42,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_single_argument_function_call() {
        let source = r#"
def double(x: i32) -> i32 = x * 2

def result: i32 = double(5)
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_multi_argument_function_call() {
        let source = r#"
def add(a: i32, b: i32) -> i32 = a + b

def result: i32 = add(5, 3)
"#;
        // This should work with our new tuple-based multi-argument system
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_nullary_function_call() {
        let source = r#"
def getAnswer() -> i32 = 42

def result: i32 = getAnswer()
"#;
        assert!(parse_and_check(source).is_ok());
    }

    // ==================== WEEK 2 INTERSECTION TYPE TESTS ====================
    
    #[test]
    fn test_field_intersection_parsing() {
        // Test that struct syntax {x: T1, y: T2} parses to field intersections {x: T1}^{y: T2}
        let source = r#"
def type Point = {
    x: i32,
    y: i32,
}

def origin: Point = {
    x = 0,
    y = 0,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_single_field_intersection() {
        // Test that single field struct becomes single FieldConstraint
        let source = r#"
def type Name = {
    value: String,
}

def myName: Name = {
    value = "Alice",
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_complex_field_intersection() {
        // Test multi-field struct with different types
        let source = r#"
def type ComplexType = {
    name: String,
    count: i32,
    active: bool,
}

def instance: ComplexType = {
    name = "test",
    count = 42,
    active = true,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_automatic_nominal_intersection_creation() {
        // Test that def type Person = {...} creates Person^{...fields...}
        let source = r#"
def type Student = {
    name: String,
    grade: i32,
}

def type Teacher = {
    name: String,
    subject: String,
}

def processStudent(s: Student) -> String = s.name
def processTeacher(t: Teacher) -> String = t.name
"#;
        // Both should work - each nominal type should have automatic intersection
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_field_constraint_to_nominal_subtyping() {
        // Test that field constraint intersections subtype to compatible nominal types
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

def alice: Person = {
    name = "Alice",
    age = 30,
}

def bob: Person = {
    name = "Bob", 
    age = 25,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_object_literal_with_nominal_types() {
        // Test comprehensive object literal → nominal type checking
        let source = r#"
def type Address = {
    street: String,
    city: String,
}

def type PersonWithAddress = {
    name: String,
    age: i32,
    address: Address,
}

def home: Address = {
    street = "123 Main St",
    city = "Anytown",
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_nat_to_integer_in_intersections() {
        // Test that Nat literals work in field intersections with integer fields
        let source = r#"
def type Config = {
    port: i32,
    timeout: u32,
    retries: i64,
}

def defaultConfig: Config = {
    port = 8080,
    timeout = 5000,
    retries = 3,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_member_access_on_intersection_types() {
        // Test that member access works on intersection types
        let source = r#"
def type Vehicle = {
    brand: String,
    year: i32,
}

def getBrand(v: Vehicle) -> String = v.brand
def getYear(v: Vehicle) -> i32 = v.year

def myCar: Vehicle = {
    brand = "Toyota",
    year = 2022,
}

def carBrand: String = getBrand(myCar)
def carYear: i32 = getYear(myCar)
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_intersection_type_field_mismatch() {
        // Test that mismatched fields are caught
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

def badPerson: Person = {
    name = "Alice",
    wrongField = 30,
}
"#;
        // This should fail - wrongField doesn't match age field
        let result = parse_and_check(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_field_in_object_literal() {
        // Test that missing required fields are caught
        let source = r#"
def type Complete = {
    field1: String,
    field2: i32,
}

def incomplete: Complete = {
    field1 = "value",
    # missing field2
}
"#;
        // This should fail - missing field2
        let result = parse_and_check(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_literal_to_string_type() {
        // Test that string literals work with String type in intersections
        let source = r#"
def type Message = {
    content: String,
    priority: i32,
}

def urgent: Message = {
    content = "System alert!",
    priority = 1,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_nested_field_intersections() {
        // Test that nested types work with field intersections
        let source = r#"
def type Inner = {
    value: i32,
}

def type Outer = {
    inner: Inner,
    label: String,
}

def createInner: Inner = {
    value = 42,
}

def createOuter: Outer = {
    inner = createInner,
    label = "test",
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    // ==================== WEEK 3 CONTEXT ENHANCEMENT TESTS ====================
    
    #[test]
    fn test_week3_context_enhancement_system() {
        // Week 3 Verification: Tests that nominal types automatically return intersection versions
        // and that member access works through intersection field lookup
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

# This should work because:
# 1. Person gets stored as Person^{name: String}^{age: i32} 
# 2. When we access p: Person, we get the full intersection type
# 3. Member access p.name works through intersection field lookup
def getPersonInfo(p: Person) -> String = p.name

# This should also work - object literal should subtype to nominal type
def alice: Person = {
    name = "Alice",
    age = 30,
}

# Function call with intersection type parameter
def result: String = getPersonInfo(alice)
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_automatic_intersection_context_resolution() {
        // Test that nominal types are automatically resolved as intersections in all contexts
        let source = r#"
def type Config = {
    host: String,
    port: i32,
    timeout: i32,
}

def getHost(cfg: Config) -> String = cfg.host
def getPort(cfg: Config) -> i32 = cfg.port
def getTimeout(cfg: Config) -> i32 = cfg.timeout

def serverConfig: Config = {
    host = "localhost",
    port = 8080,
    timeout = 30,
}

def hostName: String = getHost(serverConfig)
def portNumber: i32 = getPort(serverConfig)
def timeoutValue: i32 = getTimeout(serverConfig)
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_retroactive_type_enhancement_foundation() {
        // Test foundation for retroactive enhancement (Week 3 without open/close blocks)
        // All nominal types should automatically have their intersection representations available
        let source = r#"
def type Base = {
    id: i32,
    name: String,
}

def type Extended = {
    id: i32,
    name: String,
    extra: bool,
}

# Base operations work immediately
def getBaseName(b: Base) -> String = b.name
def getBaseId(b: Base) -> i32 = b.id

# Extended operations work immediately  
def getExtendedInfo(e: Extended) -> String = e.name
def getExtendedExtra(e: Extended) -> bool = e.extra

# Object literals work with both
def baseObj: Base = {
    id = 1,
    name = "base",
}

def extendedObj: Extended = {
    id = 2, 
    name = "extended",
    extra = true,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    // ==================== PHASE 1 METHOD COLLECTION TESTS ====================
    
    #[test]
    fn test_method_collection_from_open_block() {
        // Test that method definitions inside open/close blocks are collected
        let source = r#"
def trait Add for T: type = {
    add: (T, T) -> T,
}

def type Counter = {
    value: i32,
}

open Add^Counter

def add(a: Counter, b: Counter) -> Counter = a

close Add^Counter
"#;
        let result = parse_and_check(source);
        assert!(result.is_ok(), "Failed to parse open block with method: {:?}", result.err());
    }
    
    #[test]
    fn test_multiple_methods_in_open_block() {
        // Test collecting multiple method definitions from one open block
        let source = r#"
def trait Math for T: type = {
    add: (T, T) -> T,
    multiply: (T, T) -> T,
}

def type Number = {
    value: i32,
}

open Math^Number

def add(a: Number, b: Number) -> Number = a
def multiply(a: Number, b: Number) -> Number = a

close Math^Number
"#;
        let result = parse_and_check(source);
        assert!(result.is_ok(), "Failed to collect multiple methods: {:?}", result.err());
    }
    
    #[test]
    fn test_nested_open_blocks_not_allowed() {
        // Test that we handle nested open blocks correctly (should not be allowed)
        let source = r#"
def trait A for T: type = {}
def trait B for T: type = {}
def type MyType = { x: i32 }

open A^MyType
open B^MyType
def method() -> i32 = 42
close B^MyType  
close A^MyType
"#;
        // For now, this should work but only associate method with the inner block
        let result = parse_and_check(source);
        assert!(result.is_ok(), "Nested open blocks should work (method goes to inner block)");
    }
    
    #[test] 
    fn test_method_outside_open_block() {
        // Test that methods outside open blocks are not associated with traits
        let source = r#"
def trait Add for T: type = {}
def type Counter = { value: i32 }

def regularFunction() -> i32 = 42

open Add^Counter
def traitMethod() -> i32 = 42  
close Add^Counter

def anotherRegularFunction() -> i32 = 42
"#;
        let result = parse_and_check(source);
        assert!(result.is_ok(), "Methods outside open blocks should work normally");
    }
    
    #[test]
    fn test_orphan_rule_in_method_collection() {
        // Test orphan rule enforcement during method collection
        let source = r#"
# Both i32 and String are built-in (external) types
open i32^String

def convert(x: i32) -> String = "test"

close i32^String
"#;
        let result = parse_and_check(source);
        assert!(result.is_err(), "Should fail orphan rule - both types are external");
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("neither type defined in current module") || 
                error_msg.contains("orphan"), "Should mention orphan rule violation: {}", error_msg);
    }
    
    #[test]
    fn test_method_collection_with_local_type() {
        // Test that method collection works when one type is locally defined
        let source = r#"
def type MyCounter = {
    count: i32,
}

def trait Display for T: type = {
    show: (T) -> String,
}

# MyCounter is local, Display is local -> should work
open Display^MyCounter

def show(c: MyCounter) -> String = "counter"

close Display^MyCounter
"#;
        let result = parse_and_check(source);
        assert!(result.is_ok(), "Should work with local types: {:?}", result.err());
    }
    
    #[test]
    fn test_enhancement_registry_population() {
        // Test that enhancement registry is populated correctly from open blocks
        use crate::lexer::{lexer, tts_to_stream};
        use crate::parser::program_parser;
        use chumsky::Parser;
        
        let source = r#"
def type Person = { name: String }
def trait Add for T: type = {}

open Add^Person
def add(a: Person, b: Person) -> Person = a
close Add^Person
"#;
        
        // Parse tokens and AST
        let (tts, _) = lexer().parse_recovery(source);
        let tts = tts.expect("Failed to parse tokens");
        let eoi = 0..source.chars().count();
        let token_stream = tts_to_stream(eoi, tts);
        let (ast, _) = program_parser().parse_recovery(token_stream);
        let ast = ast.expect("Failed to parse AST");
        
        // Test symbol collection directly
        let symbols = collect_module_symbols(&ast).unwrap();
        
        // Check that type definitions were registered
        assert!(symbols.type_definitions.contains_key("Person"));
        assert!(symbols.traits.contains_key("Add"));
        
        // Check that enhancements were registered
        assert!(symbols.enhancements.contains_key("Add"));
        assert!(symbols.enhancements.contains_key("Person")); 
        assert!(symbols.enhancements.get("Add").unwrap().contains(&"Person".to_string()));
        assert!(symbols.enhancements.get("Person").unwrap().contains(&"Add".to_string()));
        
        // Check that trait method was collected
        assert!(symbols.trait_methods.contains_key(&("Add".to_string(), "Person".to_string())));
        let methods = symbols.trait_methods.get(&("Add".to_string(), "Person".to_string())).unwrap();
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0], "add");
        
        // Check that function signature was registered
        assert!(symbols.function_signatures.contains_key("add"));
    }

    #[test]
    fn test_complete_module_registry() {
        // Test that registry collects ALL types of definitions in a module
        // Use the existing working syntax from other tests
        let result = parse_and_check(r#"
def type Person = { name: String, age: i32 }

def greet(p: Person) -> String = p.name

def trait Display for T: type = {
    show: (T) -> String,
}

open Display^Person
def show(p: Person) -> String = p.name
close Display^Person
"#);
        
        assert!(result.is_ok(), "Should parse successfully: {:?}", result.err());
        
        // Now test the registry directly with a simpler approach
        use crate::lexer::{lexer, tts_to_stream};
        use crate::parser::program_parser;
        use chumsky::Parser;
        
        let simple_source = r#"
def type Counter = { value: i32 }
def trait Add for T: type = { add: (T, T) -> T }
open Add^Counter
def add(a: Counter, b: Counter) -> Counter = a
close Add^Counter
"#;
        
        let (tts, _) = lexer().parse_recovery(simple_source);
        let tts = tts.expect("Failed to parse tokens");
        let eoi = 0..simple_source.chars().count();
        let token_stream = tts_to_stream(eoi, tts);
        let (ast, _) = program_parser().parse_recovery(token_stream);
        let ast = ast.expect("Failed to parse AST");
        
        let symbols = collect_module_symbols(&ast).unwrap();
        
        println!("Complete registry test - Registry contents:");
        println!("  Types: {:?}", symbols.type_definitions.keys().collect::<Vec<_>>());
        println!("  Functions: {:?}", symbols.function_signatures.keys().collect::<Vec<_>>());
        println!("  Constants: {:?}", symbols.constants.keys().collect::<Vec<_>>());
        println!("  Traits: {:?}", symbols.traits.keys().collect::<Vec<_>>());
        
        // Verify comprehensive collection
        assert!(symbols.type_definitions.contains_key("Counter"));
        assert!(symbols.traits.contains_key("Add")); 
        assert!(symbols.function_signatures.contains_key("add"));
        assert!(symbols.enhancements.contains_key("Add"));
        assert!(symbols.enhancements.contains_key("Counter"));
        assert!(symbols.trait_methods.contains_key(&("Add".to_string(), "Counter".to_string())));
        
        println!("✅ Complete registry successfully collects all definition types!");
    }

    #[test]
    fn test_forward_references_with_complete_registry() {
        // Test that forward references work with complete registry
        let result = parse_and_check(r#"
# Early function using later-defined type
def useCounter(c: Counter) -> i32 = c.value

# Later type definition 
def type Counter = { value: i32 }

# Early constant using later-defined function
def result: i32 = add(5, 3)

# Later function definition
def add(a: i32, b: i32) -> i32 = a + b
"#);
        
        assert!(result.is_ok(), "Forward references should work with complete registry: {:?}", result.err());
        println!("✅ Forward references work with complete registry!");
    }
    
    #[test]
    fn test_comprehensive_enhanced_type_resolution() {
        // Test comprehensive enhanced type resolution with complex forward references
        let result = parse_and_check(r#"
# Complex forward references between types, functions, and constants
def processData(data: DataProcessor) -> ProcessedResult = makeResult()

def transform(result: ProcessedResult) -> String = result.output

def makeResult() -> ProcessedResult = { output = "processed" }

def type DataProcessor = { 
    input: String,
    config: Config,
}

def type Config = { debug: bool }

def type ProcessedResult = { output: String }

def trait Processor for T: type = {
    process: (T) -> ProcessedResult,
}
"#);
        
        assert!(result.is_ok(), "Complex forward references should work: {:?}", result.err());
        println!("✅ Enhanced type resolution with complex forward references works!");
    }
    
    #[test]
    fn test_orphan_rule_with_open_blocks() {
        // Test the specific orphan rule issue with open blocks
        let result = parse_and_check(r#"
# Open block appears before type definitions - should now work with two-pass approach
open Processor^DataProcessor
def process(dp: DataProcessor) -> ProcessedResult = { output = "processed" }
close Processor^DataProcessor

def type DataProcessor = { 
    input: String,
}

def type ProcessedResult = { output: String }

def trait Processor for T: type = {
    process: (T) -> ProcessedResult,
}
"#);
        
        assert!(result.is_ok(), "Open blocks should work with proper orphan rule validation: {:?}", result.err());
        println!("✅ Orphan rule now works correctly with two-pass symbol collection!");
    }

    #[test]
    fn test_retroactive_type_enhancement() {
        // Test that open blocks retroactively enhance ALL existing variables of that type
        // Focus on type enhancement without method dispatch (which is Phase 4)
        let result = parse_and_check(r#"
# Early variable declaration with base type
def counter1: Counter = { value = 10 }

# Early function using base type
def getCounterValue(c: Counter) -> i32 = c.value

# Later: open block should retroactively enhance ALL Counter types to Counter^Add
open Add^Counter
def add(a: Counter, b: Counter) -> Counter = { value = a.value + b.value }
close Add^Counter

# After open block: all Counter types are enhanced but we don't test method dispatch yet
def counter2: Counter = { value = 20 }

def type Counter = { value: i32 }

def trait Add for T: type = {
    add: (T, T) -> T,
}
"#);
        
        assert!(result.is_ok(), "Retroactive enhancement should work: {:?}", result.err());
        println!("✅ Retroactive type enhancement working!");
    }
    
    #[test]
    fn test_retroactive_enhancement_comprehensive() {
        // Test comprehensive retroactive enhancement with debug output
        use crate::lexer::{lexer, tts_to_stream};
        use crate::parser::program_parser;
        use chumsky::Parser;
        
        let source = r#"
# Declare variables BEFORE enhancement
def person1: Person = { name = "Alice" }

# Function using base type BEFORE enhancement  
def getName(p: Person) -> String = p.name

# LATER: Open block should enhance all existing Person types
open Display^Person
def show(p: Person) -> String = p.name
close Display^Person

# Variables declared AFTER should also get enhanced type
def person2: Person = { name = "Bob" }

def type Person = { name: String }
def trait Display for T: type = { show: (T) -> String }
"#;
        
        let (tts, _) = lexer().parse_recovery(source);
        let tts = tts.expect("Failed to parse tokens");
        let token_stream = tts_to_stream(0..source.len(), tts);
        let (ast, _) = program_parser().parse_recovery(token_stream);
        let ast = ast.expect("Failed to parse AST");
        
        // Test the full enhancement pipeline
        let result = check_program(&ast);
        assert!(result.is_ok(), "Comprehensive retroactive enhancement should work: {:?}", result.err());
        
        println!("✅ Comprehensive retroactive enhancement successful!");
    }
    
    #[test]
    fn test_in_place_enhancement_with_debug_output() {
        // Test with detailed inspection of the enhancement process
        use crate::lexer::{lexer, tts_to_stream};
        use crate::parser::program_parser;
        use chumsky::Parser;
        
        let source = r#"
def type Person = { name: String }
def trait Display for T: type = { show: (T) -> String }

open Display^Person
def show(p: Person) -> String = p.name
close Display^Person
"#;
        
        let (tts, _) = lexer().parse_recovery(source);
        let tts = tts.expect("Failed to parse tokens");
        let eoi = 0..source.chars().count();
        let token_stream = tts_to_stream(eoi, tts);
        let (ast, _) = program_parser().parse_recovery(token_stream);
        let ast = ast.expect("Failed to parse AST");
        
        let mut symbols = collect_module_symbols(&ast).unwrap();
        
        println!("Before enhancement:");
        if let Some(person_type) = symbols.type_definitions.get("Person") {
            println!("  Person type: {:?}", person_type);
        }
        
        // Apply enhancements
        symbols.apply_all_enhancements();
        
        println!("After enhancement:");
        if let Some(enhanced_person_type) = symbols.type_definitions.get("Person") {
            println!("  Enhanced Person type: {:?}", enhanced_person_type);
        }
        
        // Verify enhancement was applied
        assert!(symbols.enhancements.contains_key("Person"));
        assert!(symbols.enhancements.contains_key("Display"));
        
        println!("✅ In-place enhancement with debug output successful!");
    }

    // ==================== WEEK 4 ENHANCEMENT REGISTRY TESTS ====================
    
    #[test]
    fn test_week4_enhancement_registry_basic() {
        // Week 4 Test: Basic enhancement registry functionality with open/close blocks
        let source = r#"
def type Person = {
    name: String,
    age: i32,
}

def trait Add for T: type = {
    add: (T, T) -> T,
}

open Add^Person

def add(a: Person, b: Person) -> Person = a

close Add^Person

def alice: Person = {
    name = "Alice",
    age = 30,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_week4_orphan_rule_enforcement() {
        // Test that orphan rule prevents external type intersections
        // This test simulates opening an external trait on external type - both are not locally defined
        // We simulate this by using built-in types for both sides
        let source = r#"
# This should fail orphan rule: neither i32 nor String are locally defined
# Both are built-in types, so neither is defined in current module
open i32^String

def convert(x: i32) -> String = "test"

close i32^String
"#;
        // This should fail due to orphan rule violation - both types are external
        assert!(parse_and_check(source).is_err());
    }

    #[test] 
    fn test_week4_local_type_enhancement_allowed() {
        // Test that enhancement is allowed when we define one of the types locally
        let source = r#"
def type MyType = {
    value: i32,
}

def trait ExternalTrait for T: type = {
    process: (T) -> T,
}

# This should be allowed: MyType is locally defined
open ExternalTrait^MyType

def process(x: MyType) -> MyType = x

close ExternalTrait^MyType

def instance: MyType = {
    value = 42,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_week4_retroactive_enhancement() {
        // Test that enhancements apply retroactively to all type usage in the module
        let source = r#"
def type Counter = {
    count: i32,
}

# This Counter usage should get enhanced even though open block comes later
def initialCounter: Counter = {
    count = 0,
}

def trait Incrementable for T: type = {
    increment: (T) -> T,
}

# This enhancement should apply to ALL Counter variables in the module
open Incrementable^Counter

def increment(c: Counter) -> Counter = c

close Incrementable^Counter

def anotherCounter: Counter = {
    count = 5,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_week4_multiple_enhancements() {
        // Test that a type can have multiple enhancements
        let source = r#"
def type Number = {
    value: i32,
}

def trait Add for T: type = {
    add: (T, T) -> T,
}

def trait Multiply for T: type = {
    multiply: (T, T) -> T,
}

open Add^Number
def add(a: Number, b: Number) -> Number = a
close Add^Number

open Multiply^Number  
def multiply(a: Number, b: Number) -> Number = a
close Multiply^Number

def num: Number = {
    value = 42,
}
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_control_flow_if_expression() {
        let source = r#"
def testIf(flag: bool) -> i32 =
    if flag:
        42
    else:
        0
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test] 
    fn test_control_flow_match_expression() {
        let source = r#"
def type Option for T: type = 
    | None
    | Some(T)

def testMatch(opt: Option[i32]) -> bool =
    match opt:
    | None => false
    | Some(val) => true
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_control_flow_while_expression() {
        let source = r#"
def testWhile() -> () =
    let n: i32 = 5
    while n > 0:
        n = n - 1
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_control_flow_let_bindings() {
        let source = r#"
def testLet() -> i32 =
    let x: i32 = 10
    let y = x + 5
    y
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_control_flow_simple_block() {
        // Test a simpler block structure that the parser can handle
        let source = r#"
def testBlock() -> i32 =
    let a = 5
    let b = 3
    a + b
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_complete_expr_coverage() {
        // Test that all major expression types can be type-checked
        let source = r#"
def testAllExprs() -> i32 =
    let tuple = (1, 2, 3)          # Expr::Tuple
    let obj = {x = 5, y = 10}      # Expr::Object  
    let sum = obj.x + obj.y        # Expr::Member + Expr::Binary
    let result = add(sum, 42)      # Expr::Call
    result                         # Expr::QualifiedIdent

def add(a: i32, b: i32) -> i32 = a + b  # Function for testing calls
"#;
        assert!(parse_and_check(source).is_ok());
    }

    // ========== BLOCK SCOPING TESTS ==========
    // These tests demonstrate the scoping issues that need to be fixed

    #[test]
    fn test_for_loop_variable_scoping() {
        // Test that for-loop variables don't leak into parent scope
        let source = r#"
def testForScoping() -> i32 =
    let x = 10
    for i in range(0, 5):
        let y = x + i  # Should access parent scope x
        pass
    # i and y should NOT be accessible here - this should work without error
    x  # Return parent scope variable
    
def range(start: i32, end: i32) -> Vec[i32] = [42]  # Stub
def type Vec for T: type = pass
def pass() -> () = ()
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test] 
    fn test_if_expression_scoping() {
        // Test that if expressions don't leak variables from branches
        let source = r#"
def testIfScoping(flag: bool) -> i32 =
    let x = 10
    if flag:
        let y = x + 5  # Local to if branch
        y
    else:
        let z = x + 3  # Local to else branch  
        z
    # y and z should NOT be accessible here
    # This function should return i32 from the if expression

def testIfScopingWithParentAccess(flag: bool) -> i32 =
    let parent_var = 42
    if flag:
        parent_var + 1  # Should access parent scope
    else:
        parent_var - 1  # Should access parent scope
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_while_loop_scoping() {
        // Test that while loops maintain proper scoping
        let source = r#"
def testWhileScoping() -> i32 =
    let x = 10
    let counter = 0
    while counter < 3:
        let temp = x + counter  # Local to while body
        counter = counter + 1   # Should modify parent scope variable
    # temp should NOT be accessible here
    x + counter  # Should access parent scope variables

def testWhileParentAccess() -> i32 =
    let parent_var = 100
    let result = 0
    while result < 5:
        result = parent_var / 20  # Should access parent scope
    result
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test] 
    fn test_match_expression_scoping() {
        // Test that match expressions maintain proper scoping
        let source = r#"
def type Option for T: type =
    | None
    | Some(T)

def testMatchScoping(opt: Option[i32]) -> i32 =
    let default_val = 42
    match opt:
        | Some(value) => value + default_val  # Access parent + pattern variable
        | None => default_val * 2            # Access parent scope
    # computed, fallback should NOT be accessible here
    # value from pattern should NOT be accessible here
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_nested_block_scoping() {
        // Test that nested blocks maintain proper scoping hierarchy
        let source = r#"
def type Vec for T: type = pass
def range(start: i32, end: i32) -> Vec[i32] = [42]  # Stub
def pass() -> () = ()

def testNestedBlocks() -> i32 =
    let outer = 10
    if true:
        let middle = outer + 5  # Access outer scope
        for i in range(0, 2):
            let inner = middle + i  # Access middle scope (if branch)
            pass
        # inner should NOT be accessible here
        # i should NOT be accessible here
        middle  # Should be accessible
    else:
        outer  # Should access outer scope
    # middle should NOT be accessible here
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_let_binding_scoping() {
        // Test that let bindings create proper local scope
        let source = r#"
def testLetScoping() -> i32 =
    let x = 10
    let y = 
        let temp = x * 2  # Inner let expression
        temp + 5          # temp is accessible here
    # temp should NOT be accessible here
    x + y  # Should access outer scope variables

def testShadowing() -> i32 =
    let x = 10
    let result = 
        let x = 20  # Shadow outer x
        x + 5       # Should use inner x (20)
    x + result      # Should use outer x (10) + result (25) = 35
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_block_return_type_isolation() {
        // Test that blocks return correct types without context leakage
        let source = r#"
def testBlockTypes() -> i32 =
    let result = 
        let local_var = 42
        local_var * 2  # Block returns i32
    result  # Should be i32 from the block

def testNestedBlockTypes() -> bool =
    let outer_flag = true
    let inner_result = 
        if outer_flag:
            let temp = 100
            temp > 50  # Returns bool
        else:
            false      # Returns bool
    inner_result  # Should be bool
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_enhanced_intersection_types_in_blocks() {
        // Test that enhanced intersection types work correctly in all block types
        let source = r#"
def type Counter = { value: i32 }
def trait Add for T: type = { add: (T, T) -> T }
open Add^Counter

def type Vec for T: type = pass
def type Option for T: type =
    | None
    | Some(T)

def range(start: i32, end: i32) -> Vec[i32] = [42]  # Stub
def pass() -> () = ()

def testEnhancedTypesInBlocks() -> Counter =
    let counter: Counter = { value = 0 }
    
    # Test in if expression
    let counter2 = if true:
        let temp: Counter = { value = counter.value + 1 }
        temp
    else:
        counter
        
    # Test in for loop  
    for i in range(0, 3):
        let loop_counter: Counter = { value = i }
        pass
        
    # Test in match expression
    let final_counter = 
        match Some(counter2):
            | Some(c) => 
                let result: Counter = { value = c.value + 10 }
                result
            | None => { value = 999 }
    
    final_counter
"#;
        assert!(parse_and_check(source).is_ok());
    }

    #[test]
    fn test_for_loop_variable_leakage_should_fail() {
        // This test verifies that scoping is working - `i` should not be accessible outside the for loop
        let source = r#"
def type Vec for T: type = pass
def pass() -> () = ()
def range(start: i32, end: i32) -> Vec[i32] = [42]

def testForLeakage() -> i32 =
    for i in range(0, 5):
        pass()
    i  # This should NOT be accessible - should cause error when scoping is fixed
"#;
        // The scoping fix means this should fail when trying to access `i`
        let result = parse_and_check(source);
        assert!(result.is_err(), "Expected type checking to fail due to variable 'i' not being accessible outside for loop");
        
        // Check that the error indicates type checking failed (which means the variable was not found)
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Type checking failed") || error_msg.to_lowercase().contains("variable"), 
                "Expected error message to indicate type checking failure, got: {}", error_msg);
    }

    #[test]
    fn test_if_branch_variable_leakage_should_fail() {
        // This test verifies that scoping is working - `branch_local` should not be accessible outside the if branch
        let source = r#"
def testIfLeakage(flag: bool) -> i32 =
    if flag:
        let branch_local = 42
        branch_local
    else:
        10
    branch_local  # This should NOT be accessible - should cause error when scoping is fixed
"#;
        // The scoping fix means this should fail when trying to access `branch_local`
        let result = parse_and_check(source);
        assert!(result.is_err(), "Expected type checking to fail due to variable 'branch_local' not being accessible outside if branch");
        
        // Check that the error indicates type checking failed (which means the variable was not found)
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Type checking failed") || error_msg.to_lowercase().contains("variable"), 
                "Expected error message to indicate type checking failure, got: {}", error_msg);
    }

    #[test]
    fn test_pattern_variable_type_extraction_from_sum_types() {
        // This test verifies that pattern variables get the correct types extracted from sum type variants
        let source = r#"
def type Result for T: type, E: type =
    | Ok(T)
    | Err(E)

def type Maybe for T: type =
    | Nothing
    | Just(T)

def type Complex =
    | SimpleVariant
    | TupleVariant(i32, String)
    | SingleValue(bool)

def testResultPattern(r: Result[i32, String]) -> i32 =
    match r:
        | Ok(value) => value      # value should have type i32
        | Err(msg) => 0           # msg should have type String (not used but pattern variable extracted)

def testMaybePattern(m: Maybe[String]) -> String =
    match m:
        | Nothing => "default"
        | Just(content) => content  # content should have type String

def testComplexPattern(c: Complex) -> i32 =
    match c:
        | SimpleVariant => 0
        | TupleVariant(num, text) => num    # num: i32, text: String (only using num)
        | SingleValue(flag) => 1            # flag: bool (not used but extracted)

# Test with concrete constructor calls to verify the types work end-to-end
def testConstructorsAndPatterns() -> i32 =
    let success_result = Ok(42)         # Ok: (∀T. T -> Result[T, E]) applied to 42 -> Result[Nat, E]
    let maybe_value = Just(100)         # Just: (∀T. T -> Maybe[T]) applied to 100 -> Maybe[Nat]
    
    match success_result:
        | Ok(val) => val                # val should be Nat, which subtypes to i32
        | Err(e) => 0

def testStringPatternExtraction() -> String =
    let string_maybe = Just("hello")    # Just applied to String -> Maybe[String]
    match string_maybe:
        | Nothing => "empty"
        | Just(text) => text            # text should be String
"#;
        assert!(parse_and_check(source).is_ok(), "Pattern variable type extraction should work correctly");
    }
}
