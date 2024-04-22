#[derive(Debug)]
pub enum BasicType {
    Bool,
    Nat,
    Int,
    Real,
    Char,
    Str, //Intern<String>),
}

#[derive(Debug)]
pub struct Func {
    pub name: Spanned<String>,
    pub args: Vec<Type>,
    pub rtn: Spanned<Type>,
    pub generics: Option<Generics>,
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub members: Dict<String, Type>,
}

#[derive(Debug)]
pub enum Type {
    Basic(BasicType),
    // Named(String, Vec<Self>),
    Object(Object),
    List(Box<Self>),
    Dict(Box<Self>, Box<Self>),
    Tuple(Vec<Self>),
    Func(Vec<Self>, Box<Self>),
    Error,
}
