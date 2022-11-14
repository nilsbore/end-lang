use chumsky::prelude::*; //{prelude::*, BoxStream, Flat};
use crate::utils::*;
use crate::ast::*;
use crate::lexer::*;

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let val = select! {
            //Token::Null => Expr::Value(Value::Null),
            Token::Bool(x) => Expr::Value(Value::Bool(x)),
            Token::Int(n) => Expr::Value(Value::Nat(n)),
            Token::Str(s) => Expr::Value(Value::Str(s)),
        }
        .labelled("value");

        //let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");
        let atom = val.map_with_span(|expr, span: Span| (expr, span));

        // Product ops (multiply and divide) have equal precedence
        let op = just(Token::Op(Op::Mul))
            .to(BinaryOp::Mul)
            .or(just(Token::Op(Op::Div)).to(BinaryOp::Div));
        let product = atom
            .clone()
            .then(op.then(atom).repeated())
            .foldl(|a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::Binary(Box::new(a), op, Box::new(b)), span)
            });

        // Sum ops (add and subtract) have equal precedence
        let op = just(Token::Op(Op::Add))
            .to(BinaryOp::Add)
            .or(just(Token::Op(Op::Sub)).to(BinaryOp::Sub));
        let sum = product
            .clone()
            .then(op.then(product).repeated())
            .foldl(|a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::Binary(Box::new(a), op, Box::new(b)), span)
            });

        // Comparison ops (equal, not-equal) have equal precedence
        let op = just(Token::Op(Op::Eq))
            .to(BinaryOp::Eq)
            .or(just(Token::Op(Op::NotEq)).to(BinaryOp::NotEq));

        let compare = sum
            .clone()
            .then(op.then(sum).repeated())
            .foldl(|a, (op, b)| {
                let span = a.1.start..b.1.end;
                (Expr::Binary(Box::new(a), op, Box::new(b)), span)
            });

        compare

    })
}

pub fn funcs_parser() -> impl Parser<Token, Func, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    // Argument lists are just identifiers separated by commas, surrounded by parentheses
    let args = ident
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::Open(Delim::Paren)), just(Token::Close(Delim::Paren)))
        .labelled("function args");

    let func = just(Token::Def)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then(args)
        .then_ignore(just(Token::Op(Op::RArrow)))
        .then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then_ignore(just(Token::Assign))
        //.then(just(Token::Assign)) //.padded()
        .then(
            expr_parser()
            //.repeated()
            .delimited_by(just(Token::Open(Delim::Block)), just(Token::Close(Delim::Block)))
                // Attempt to recover anything that looks like a function body but contains errors
                .recover_with(nested_delimiters(
                    Token::Open(Delim::Block),
                    Token::Close(Delim::Block),
                    [
                        (Token::Open(Delim::Paren), Token::Close(Delim::Paren)),
                        (Token::Open(Delim::SquareBrackets), Token::Close(Delim::SquareBrackets)),
                        (Token::Open(Delim::CurlyBrackets), Token::Close(Delim::CurlyBrackets)),
                    ],
                    |span| (Expr::Error, span),
                )),
        )
        .map(|(((name, args), rtn), body)| Func {name:name, args:args, body:vec![body]})
        .labelled("function");

    func
}
