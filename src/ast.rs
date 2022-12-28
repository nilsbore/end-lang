use crate::utils::*;
use internment::Intern;

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

#[derive(Debug, PartialEq)]
pub enum Literal {
    Bool(bool),
    Nat(u64),
    Int(i64),
    Real(f64),
    Char(char),
    Str(String), //Intern<String>),
}

#[derive(Debug)]
pub struct Generics {
    pub args: Vec<Item>,
}

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

#[derive(Debug)]
pub struct QualifiedIdent {
    pub path: Vec<Spanned<String>>,
    pub ty: Option<Spanned<Type>>,
    pub ident: Spanned<String>,
}

#[derive(Debug)]
pub enum LExpr<TExpr> {
    Error,
    Ident(QualifiedIdent),
    Call(Box<Spanned<TExpr>>, Vec<Spanned<Expr>>),
    Member(Box<Spanned<TExpr>>, Spanned<String>),
}

pub struct LLExpr {
    expr: Spanned<LExpr<LLExpr>>,
}

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
pub enum Expr {
    Error,
    Literal(Literal),
    List(Vec<Spanned<Self>>),
    //Local(String),
    Ident(QualifiedIdent),
    //Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    //Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    Member(Box<Spanned<Self>>, Spanned<String>),
    //If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    //Print(Box<Spanned<Self>>),
}

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub ty: Spanned<Type>,
}

#[derive(Debug)]
pub struct Struct {
    pub name: Spanned<String>,
    pub generics: Option<Generics>,
    pub members: Vec<Item>,
}

#[derive(Debug)]
pub struct Trait {
    pub name: Spanned<String>,
    pub generics: Generics,
    pub members: Vec<Item>,
}

#[derive(Debug)]
pub struct Impl {
    pub name: Spanned<String>,
    pub generic_args: Vec<Spanned<Type>>,
    pub funcs: Vec<Func>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: Spanned<String>,
    pub values: Vec<Spanned<Type>>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: Spanned<String>,
    pub generics: Option<Generics>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub struct Const {
    pub name: Spanned<String>, //SrcNode<Ident>,
    pub ty: Spanned<Type>,
    pub generics: Option<Generics>,
    pub value: Box<Spanned<Expr>>, //SrcNode<Expr>,
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
pub struct MatchBranch {}

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
    pub name: Spanned<String>, //SrcNode<Ident>,
    //pub generics: Generics,
    //pub ty_hint: SrcNode<Type>,
    pub args: Vec<Item>,
    pub rtn: Spanned<Type>,
    pub generics: Option<Generics>,
    //body: Spanned<Expr>,
    //pub body: Vec<Spanned<Expr>>, //SrcNode<Expr>,
    pub body: Box<AssignExpr>,
}
