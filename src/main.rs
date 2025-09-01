use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
// use chumsky::stream::Stream;
use std::{env, fs};

mod ast;
mod checker;
mod lexer;
mod parser;
mod utils;
use crate::lexer::{lexer, tts_to_stream};
use crate::parser::program_parser;
use crate::checker::check_program;

fn parse(src: &str) -> Option<ast::Program> {
    let (tts, errs) = lexer().parse_recovery(src);

    let (ast, parse_errs) = if let Some(tts) = tts {
        //println!("--- Token Trees ---\n{:#?}", tts);

        // Next, flatten
        let eoi = 0..src.chars().count();
        let token_stream = tts_to_stream(eoi, tts);

        // At this point, we have a token stream that can be fed into the main parser! Because this is just an example,
        // we're instead going to just collect the token stream into a vector and print it.

        //let flattened_trees = token_stream.fetch_tokens().collect::<Vec<_>>();

        //println!("--- Flattened Token Trees ---\n{:?}", flattened_trees);

        //let len = src.chars().count();
        let (ast, parse_errs) = program_parser().parse_recovery(token_stream);
        println!("{:#?}", ast);
        (ast, parse_errs)
    } else {
        (None, Vec::new())
    };

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
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

    ast
}

fn check(ast: ast::Program) -> Option<ast::Program> {
    match check_program(&ast) {
        Ok(()) => {
            println!("Type checking completed successfully!");
            Some(ast)
        }
        Err(error) => {
            eprintln!("Type checking failed: {}", error);
            None
        }
    }
}

fn main() {
    let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
        .expect("Failed to read file");
    let ast = parse(&src);
    let Some(ast) = ast else { return };
    check(ast);
}
