use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, BoxStream, Flat};
use std::ops::Range;
use std::{env, fs};

// Represents the different kinds of delimiters we care about
#[derive(Copy, Clone, Debug)]
enum Delim {
    Paren,
    SquareBrackets,
    CurlyBrackets,
    Block,
}

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
enum Token {
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
enum TokenTree {
    Token(Token),
    Tree(Delim, Vec<Spanned<TokenTree>>),
}

type Span = Range<usize>;

type Spanned<T> = (T, Span);

// A parser that turns pythonic code with semantic whitespace into a token tree
fn lexer() -> impl Parser<char, Vec<Spanned<TokenTree>>, Error = Simple<char>> {
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
            just('=').to(Op::Eq),
            just('.').to(Op::Dot),
            just("!=").to(Op::NotEq),
            just('!').to(Op::Not),
            just("<=").to(Op::LessEq),
            just('<').to(Op::Less),
            just(">=").to(Op::MoreEq),
            just('>').to(Op::More),
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
fn tts_to_stream(
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

fn main() {
    let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
            .expect("Failed to read file");

    let (tts, errs) = lexer().parse_recovery(src.as_str());

    if let Some(tts) = tts {
        println!("--- Token Trees ---\n{:#?}", tts);

        // Next, flatten
        let eoi = 0..src.chars().count();
        let mut token_stream = tts_to_stream(eoi, tts);

        // At this point, we have a token stream that can be fed into the main parser! Because this is just an example,
        // we're instead going to just collect the token stream into a vector and print it.

        let flattened_trees = token_stream.fetch_tokens().collect::<Vec<_>>();

        println!("--- Flattened Token Trees ---\n{:?}", flattened_trees);
    }

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        //.chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    )
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Must be closed before this {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token in input"
                        } else {
                            "Unexpected end of input"
                        },
                        if e.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Unexpected token {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&src)).unwrap();
        });
}
