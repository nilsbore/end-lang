use std::ops::Range;

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);

//#[derive(Debug, PartialEq)]
//pub struct Spanned<T>(T, Span);
