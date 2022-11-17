use internment::Intern;
use crate::utils::*;

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
    Add, Sub,
    // Product
    Mul, Div, Rem,
    // Equality
    Eq, NotEq,
    // Comparison
    Less, LessEq,
    Greater, GreaterEq,
    // Logical
    And, Or, Xor,
    // Lists
    //Join,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Bool(bool),
    Nat(u64),
    Int(i64),
    Real(f64),
    Char(char),
    Str(String), //Intern<String>),
}

/*
#[derive(Debug, PartialEq)]
pub struct GenericTy {
    pub Spanned<String>
}
*/

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    // Generated only by parser errors.
    Error,
    Unknown,
    //Named(String),
    Named(String, Vec<Spanned<Self>>),
    List(Box<Spanned<Self>>),
    Dict(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Tuple(Vec<Spanned<Self>>),
    Func(Vec<Spanned<Self>>, Box<Spanned<Self>>),
}

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
pub enum Expr {
    Error,
    Literal(Literal),
    List(Vec<Spanned<Self>>),
    Local(String),
    Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    //Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    //Print(Box<Spanned<Self>>),
}

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub ty: Spanned<Type>,
}

#[derive(Debug)]
pub struct Func {
    pub name: Spanned<String>, //SrcNode<Ident>,
    //pub generics: Generics,
    //pub ty_hint: SrcNode<Type>,
    pub args: Vec<Item>,
    pub rtn: Spanned<Type>,
    //body: Spanned<Expr>,
    pub body: Vec<Spanned<Expr>>, //SrcNode<Expr>,
}
