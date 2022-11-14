use chumsky::{prelude::*, BoxStream, Flat};
use crate::utils::*;

// Represents the different kinds of delimiters we care about
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Delim {
    Paren,
    SquareBrackets,
    CurlyBrackets,
    Block,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Op {
    // Sum
    Add, Sub,
    // Product
    Mul, Div, Rem,
    // Equality
    Eq, NotEq,
    // Comparison
    Less, LessEq,
    More, MoreEq,
    // Logical
    Not, And, Or, Xor,
    // Lists
    Dot,
    RArrow,
    RFlow,
}

// An 'atomic' token (i.e: it has no child tokens)
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Token {
    Assign,
    Comma,
    Colon,
    Pipe,
    Int(u64),
    Ident(String),
    Op(Op),
    Open(Delim),
    Close(Delim),
    Str(String),
    Bool(bool),
    Fn,
    Def,
    Trait,
    Let,
    Of,
    For,
    If,
    Else,
    While,
    Impl,
    Begin,
    End,
    Namespace,
    Struct,
    Enum,
    Block,
    Wildcard,
}

// The output of the lexer: a recursive tree of nested tokens
#[derive(Debug)]
pub enum TokenTree {
    Token(Token),
    Tree(Delim, Vec<Spanned<TokenTree>>),
}

// A parser that turns pythonic code with semantic whitespace into a token tree
pub fn lexer() -> impl Parser<char, Vec<Spanned<TokenTree>>, Error = Simple<char>> {
    let tt = recursive(|tt| {
        // Define some atomic tokens
        let int = text::int(10).from_str().unwrapped().map(Token::Int);

        let str_ = just('"')
            .ignore_then(filter(|c| *c != '"').repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .map(Token::Str);

        // A parser for identifiers and keywords
        let ident = text::ident().map(|ident: String| match ident.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "def" => Token::Def,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "of" => Token::Of,
            "while" => Token::While,
            "block" => Token::Block,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "trait" => Token::Trait,
            "impl" => Token::Impl,
            "begin" => Token::Begin,
            "end" => Token::End,
            "namespace" => Token::Namespace,
            "_" => Token::Wildcard,
            _ => Token::Ident(ident),
        });

        let ctrl = choice((
            just(',').to(Token::Comma),
            just(':').to(Token::Colon),
            just('|').to(Token::Pipe),
            just('=').to(Token::Assign),
        ));

        /*
        let op = one_of("!+-*<>/=.%")
            .repeated()
            .at_least(1)
            .collect()
            .map(Token::Op);
        */

        let op = choice((
            just("=>").to(Op::RFlow),
            just("==").to(Op::Eq),
            just('.').to(Op::Dot),
            just("!=").to(Op::NotEq),
            just('!').to(Op::Not),
            just("<=").to(Op::LessEq),
            just('<').to(Op::Less),
            just(">=").to(Op::MoreEq),
            just('>').to(Op::More),
            just("&&").to(Op::And),
            just("||").to(Op::Or),
            just('+').to(Op::Add),
            just("->").to(Op::RArrow),
            just('-').to(Op::Sub),
            just('*').to(Op::Mul),
            just('/').to(Op::Div),
            just('%').to(Op::Rem),
        ))
            .map(Token::Op);

        let single_token = int.or(str_).or(ctrl).or(op).or(ident).map(TokenTree::Token);

        // Tokens surrounded by parentheses get turned into parenthesised token trees
        let parens_token_tree = tt.clone()
            .padded()
            .repeated()
            .delimited_by(just('('), just(')'))
            .map(|tts| TokenTree::Tree(Delim::Paren, tts));

        let brackets_token_tree = tt.clone()
            .padded()
            .repeated()
            .delimited_by(just('['), just(']'))
            .map(|tts| TokenTree::Tree(Delim::SquareBrackets, tts));

        let curly_token_tree = tt.clone()
            .padded()
            .repeated()
            .delimited_by(just('{'), just('}'))
            .map(|tts| TokenTree::Tree(Delim::CurlyBrackets, tts));

        let comment = just("#").then(take_until(just('\n'))).padded();

        single_token
            .or(parens_token_tree)
            //.or(token_tree2)
            .or(brackets_token_tree)
            .or(curly_token_tree)
            .padded_by(comment.repeated())
            //.padded()
            //.repeated()
            .map_with_span(|tt, span| (tt, span))
    });

    // Whitespace indentation creates code block token trees
    text::semantic_indentation(tt, |tts, span| (TokenTree::Tree(Delim::Block, tts), span))
        .then_ignore(end())
}

/// Flatten a series of token trees into a single token stream, ready for feeding into the main parser
pub fn tts_to_stream(
    eoi: Span,
    token_trees: Vec<Spanned<TokenTree>>,
) -> BoxStream<'static, Token, Span> {
    use std::iter::once;

    BoxStream::from_nested(eoi, token_trees.into_iter(), |(tt, span)| match tt {
        // Single tokens remain unchanged
        TokenTree::Token(token) => Flat::Single((token, span)),
        // Nested token trees get flattened into their inner contents, surrounded by `Open` and `Close` tokens
        TokenTree::Tree(delim, tree) => Flat::Many(
            once((TokenTree::Token(Token::Open(delim)), span.clone()))
                .chain(tree.into_iter())
                .chain(once((TokenTree::Token(Token::Close(delim)), span))),
        ),
    })
}
