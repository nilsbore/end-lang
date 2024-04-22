#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

fn literal_checks_against(literal: &Literal, type_: &LiteralType) -> bool {
    match (literal, type_) {
        (Literal::Char(_), LiteralType::Char) => true,
        (Literal::String(_), LiteralType::String) => true,
        (Literal::Int(_), LiteralType::Int) => true,
        (Literal::Float(_), LiteralType::Float) => true,
        (Literal::Bool(_), LiteralType::Bool) => true,
        (Literal::Unit, LiteralType::Unit) => true,
        _ => false,
    }
}

/// Figure 11.
fn checks_against(
    state: &mut State,
    context: &Context,
    expr: &Expression,
    type_: &Type,
) -> Context {
    print_helper("check", format!("{}", expr), format!("{}", type_), context);
    assert!(is_well_formed(context, type_));
    match (expr, type_) {
        //1I
        (Expression::Literal(lit), Type::Literal(lit_ty)) => {
            print_rule("1I");
            assert!(literal_checks_against(lit, lit_ty));
            context.clone()
        }
        //->I
        (Expression::Abstraction(x, e), Type::Function(a, b)) => {
            print_rule("->I");
            let typed_var = ContextElement::TypedVariable(x.clone(), *a.clone());
            let gamma = context.add(typed_var.clone());
            checks_against(state, &gamma, e, b).drop(typed_var)
        }
        //forallI
        (_, Type::Quantification(alpha, a)) => {
            print_rule("∀I");
            let var = ContextElement::Variable(alpha.clone());
            let gamma = context.add(var.clone());
            checks_against(state, &gamma, expr, a).drop(var)
        }
        //xI
        (Expression::Tuple(fst, snd), Type::Product(a, b)) => {
            print_rule("xI");
            let gamma = checks_against(state, context, fst, a);
            checks_against(state, &gamma, snd, b)
        }
        //Sub
        (_, _) => {
            print_rule("Sub");
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
