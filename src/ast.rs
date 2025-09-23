use std::fmt;

use crate::utils::*;
// use internment::Intern;

/*
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ident(Intern<String>);

impl Ident {
    pub fn new<S: ToString>(s: S) -> Self { Self(Intern::new(s.to_string())) }
    pub fn as_ref(self) -> &'static String { self.0.as_ref() }
}

impl Deref for Ident {
    type Target = String;
    fn deref(&self) -> &Self::Target { &self.0 }
}
*/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    //Propagate,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // Sum
    Add,
    Sub,
    // Product
    Mul,
    Div,
    Rem,
    // Equality
    Eq,
    NotEq,
    // Comparison
    Less,
    LessEq,
    Greater,
    GreaterEq,
    // Logical
    And,
    Or,
    Xor,
    // Lists
    //Join,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Bool(bool),
    Nat(u64),
    Int(i64),
    Real(f64),
    Char(char),
    Str(String), //Intern<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Generics {
    pub args: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub ty: Spanned<Type>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Struct {
    // pub name: Spanned<String>,
    // pub generics: Option<Generics>,
    pub members: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumVariant {
    pub name: Spanned<String>,
    pub values: Vec<Spanned<Type>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    // pub name: Spanned<String>,
    // pub generics: Option<Generics>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralType {
    Bool,
    Nat,
    Int,
    Real,
    Char,
    Str, //Intern<String>),
}

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            LiteralType::Char => write!(f, "Char"),
            LiteralType::Str => write!(f, "String"),
            LiteralType::Int => write!(f, "Int"),
            LiteralType::Nat => write!(f, "Nat"),
            LiteralType::Real => write!(f, "Real"),
            LiteralType::Bool => write!(f, "Bool"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    // Generated only by parser errors.
    Error,
    Unknown,
    //Named(String),
    // Named(String, Vec<Spanned<Self>>),
    Literal(LiteralType),
    Ident(QualifiedType),
    // List(Box<Spanned<Self>>),
    // Dict(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Intersection(Box<Self>, Box<Self>),
    UniverseType,
    Tuple(Vec<Spanned<Self>>),
    Enum(Vec<EnumVariant>),
    Struct(Vec<Item>),
    FieldConstraint(String, Box<Spanned<Self>>),  // {field: Type} constraint
    Func(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Existential(String),               // unknown variable, to be decided!
    Quantification(String, Box<Self>), // generics!! does the type tree end in existentials with the same name?
}

impl Type {
    // what about existentials? can they not resolve to quantification?
    fn is_monotype(&self) -> bool {
        match self {
            Type::Quantification(..) => false,
            Type::Func(args, rtn) => {
                // args.into_iter().fold(true, |a, b| a && b.0.is_monotype()) && rtn.0.is_monotype()
                args.0.is_monotype() && rtn.0.is_monotype()
            }
            Type::Ident(ident) => ident
                .type_args
                .iter()
                .fold(true, |a, b| a && b.0.is_monotype()),
            Type::Tuple(elements) => elements
                .into_iter()
                .fold(true, |a, b| a && b.0.is_monotype()),
            Type::Struct(items) => items
                .into_iter()
                .fold(true, |a, b| a && b.ty.0.is_monotype()),
            Type::FieldConstraint(_, ty) => ty.0.is_monotype(),
            Type::Enum(variants) => variants.into_iter().fold(true, |a, b| {
                a && b
                    .values
                    .clone()
                    .into_iter()
                    .fold(true, |c, d| c && d.0.is_monotype())
            }),
            Type::Intersection(a, b) => a.is_monotype() && b.is_monotype(),
            _ => true,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Type::Error => write!(f, "Error!"),
            Type::Unknown => write!(f, "Unknown!"),
            Type::Ident(ident) => write!(f, "{}", ident),
            Type::UniverseType => write!(f, "Universe type"),
            Type::Tuple(elements) => write!(f, "({:?})", elements),
            Type::Enum(variants) => write!(f, "{:?}", variants),
            Type::Struct(members) => write!(f, "{:?}", members),
            Type::FieldConstraint(field, ty) => write!(f, "{{{}: {}}}", field, ty.0),
            Type::Func(rtn, args) => write!(f, "({}) -> {}", rtn.0, args.0),
            Type::Literal(literal) => write!(f, "{}", literal),
            Type::Existential(ex) => write!(f, "{}^", ex),
            Type::Quantification(a, ty) => write!(f, "(∀{}. {})", a, ty),
            Type::Intersection(a, b) => write!(f, "{}^{}", a, b),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefType {
    pub name: Spanned<String>,
    pub generics: Option<Generics>,
    pub ty: Spanned<Type>,
    pub exported: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedType {
    pub path: Vec<Spanned<String>>,
    pub type_args: Vec<Spanned<Type>>,
}

impl fmt::Display for QualifiedType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.path.len() > 0 {
            write!(
                f,
                "{}",
                self.path
                    .clone()
                    .into_iter()
                    .map(|s| s.0)
                    .collect::<Vec<String>>()
                    .join("::")
            )?;
        }
        if self.type_args.len() > 0 {
            write!(
                f,
                "[{}]",
                self.type_args
                    .clone()
                    .into_iter()
                    .map(|s| format!("{}", s.0))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Open {
    pub ty: Spanned<Type>,
    pub generics: Option<Generics>,
    pub exported: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Close {
    pub ty: Spanned<Type>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FromImport {
    pub module_path: Vec<String>,
    pub items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectImport {
    pub module_path: Vec<String>,
    pub item: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportStatement {
    From(FromImport),
    Direct(DirectImport),
}

// #[derive(Debug)]
// pub enum LExpr<TExpr> {
//     Error,
//     Ident(QualifiedIdent),
//     Call(Box<Spanned<TExpr>>, Vec<Spanned<Expr>>),
//     Member(Box<Spanned<TExpr>>, Spanned<String>),
// }

// pub struct LLExpr {
//     expr: Spanned<LExpr<LLExpr>>,
// }

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Clone, Debug)]
pub enum Expr {
    Error,
    Literal(Literal),
    Tuple(Vec<Spanned<Self>>),
    Object(Vec<(Spanned<String>, Spanned<Self>)>),
    //Local(String),
    // Ident(Spanned<String>),
    QualifiedIdent(QualifiedType),
    //Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    //Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(
        Box<Spanned<Self>>,
        Vec<(Option<Spanned<String>>, Spanned<Self>)>,
    ),
    Member(Box<Spanned<Self>>, Spanned<String>),
    //If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    //Print(Box<Spanned<Self>>),
    Annotation(Box<Self>, Type),
}

#[derive(Debug)]
pub struct Trait {
    pub name: Spanned<String>,
    pub generics: Generics,
    pub members: Vec<Item>,
    pub exported: bool,
}

#[derive(Debug)]
pub struct Impl {
    pub name: Spanned<String>,
    pub generic_args: Vec<Spanned<Type>>,
    pub funcs: Vec<Func>,
}

#[derive(Debug)]
pub struct Const {
    pub name: Spanned<String>, //SrcNode<Ident>,
    pub ty: Spanned<Type>,
    pub generics: Option<Generics>,
    pub value: Box<Spanned<Expr>>, //SrcNode<Expr>,
    pub exported: bool,
}

#[derive(Debug)]
pub struct Assign {
    pub name: Spanned<String>,  //SrcNode<Ident>,
    pub value: Box<AssignExpr>, //SrcNode<Expr>,
}

#[derive(Debug)]
pub struct Let {
    pub name: Spanned<String>, //SrcNode<Ident>,
    pub ty: Option<Spanned<Type>>,
    pub value: Box<AssignExpr>, //SrcNode<Expr>,
}

#[derive(Debug)]
pub struct ForExpr {
    pub ident: Spanned<String>,
    pub expr: Box<Spanned<Expr>>,
    pub body: Box<ControlExpr>,
}

#[derive(Debug)]
pub struct WhileExpr {
    pub expr: Box<Spanned<Expr>>,
    pub body: Box<ControlExpr>,
}

#[derive(Debug)]
pub struct Pattern {
    pub ident: Spanned<String>,
    pub members: Option<Vec<Self>>,
}

#[derive(Debug)]
pub struct MatchBranch {
    pub pattern: Pattern,
    pub body: Box<ControlExpr>,
}

#[derive(Debug)]
pub struct MatchExpr {
    pub expr: Box<Spanned<Expr>>,
    pub branches: Vec<MatchBranch>,
}

#[derive(Debug)]
pub struct IfExpr {
    pub main_expr: Box<Spanned<Expr>>,
    pub main_body: Box<ControlExpr>,
    pub elif_expr_body: Vec<(Spanned<Expr>, ControlExpr)>,
    pub else_body: Box<Option<ControlExpr>>,
}

#[derive(Debug)]
pub enum Statement {
    Error,
    Expr(Box<AssignExpr>),
    Let(Box<Let>),
    Assign(Box<Assign>),
}

#[derive(Debug)]
pub struct Block {
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum BlockExpr {
    For(ForExpr),
    While(WhileExpr),
    Match(MatchExpr),
    If(IfExpr),
    Block(Block),
}

#[derive(Debug)]
pub enum ControlExpr {
    Expr(Box<Spanned<Expr>>),
    Block(Block),
    //Stmts(Vec<Statement>),
}

#[derive(Debug)]
pub enum AssignExpr {
    Expr(Box<Spanned<Expr>>),
    BlockExpr(Box<BlockExpr>),
    //Stmts(Vec<Statement>),
}

#[derive(Debug)]
pub struct Func {
    // pub name: Spanned<String>, //SrcNode<Ident>,
    pub ty: Option<Spanned<Type>>,
    pub ident: String,
    //pub generics: Generics,
    //pub ty_hint: SrcNode<Type>,
    pub args: Vec<Item>,
    pub rtn: Spanned<Type>,
    pub generics: Option<Generics>,
    //body: Spanned<Expr>,
    //pub body: Vec<Spanned<Expr>>, //SrcNode<Expr>,
    pub body: Box<AssignExpr>,
    pub exported: bool,
}

#[derive(Debug)]
pub enum Def {
    Func(Func),
    // Enum(Enum),
    // Struct(Struct),
    Type(DefType),
    // Impl(Impl),
    Trait(Trait),
    Const(Const),
    Open(Open),
    Close(Close),
    Import(ImportStatement),
}

#[derive(Debug)]
pub struct Program {
    pub defs: Vec<Def>,
}
