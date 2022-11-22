use crate::ast::*;
use crate::lexer::*;
use crate::utils::*;
use chumsky::prelude::*; //{prelude::*, BoxStream, Flat};

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let val = select! {
            //Token::Null => Expr::Value(Value::Null),
            Token::Bool(x) => Expr::Literal(Literal::Bool(x)),
            Token::Int(n) => Expr::Literal(Literal::Nat(n)),
            Token::Str(s) => Expr::Literal(Literal::Str(s)),
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

pub fn generics_parser() -> impl Parser<Token, Generics, Error = Simple<Token>> + Clone {
    // for T: type, N: uint64
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let item = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(|(name, ty)| Item { name, ty });

    let generics = just(Token::For)
        .ignore_then(
            item.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
        )
        .map(|args| Generics { args });

    generics
}

pub fn struct_parser() -> impl Parser<Token, Struct, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let item = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(|(name, ty)| Item { name, ty });

    let struct_ = just(Token::Def)
        .ignore_then(just(Token::Struct))
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("struct name"),
        )
        .then(generics_parser().or_not())
        .then_ignore(just(Token::Assign))
        //.then(just(Token::Assign)) //.padded()
        //.then(item.repeated())
        .then(
            item.repeated().delimited_by(
                just(Token::Open(Delim::Block)),
                just(Token::Close(Delim::Block)),
            ), /*
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
               */
        )
        .map(|((name, generics), members)| Struct { name, generics, members })
        .labelled("struct");

    struct_
}

pub fn enum_parser() -> impl Parser<Token, Enum, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let variant = 
        ident.clone()
            .map_with_span(|name, span| (name, span))
        .then(
            type_parser().separated_by(just(Token::Comma)).allow_trailing().delimited_by(
                just(Token::Open(Delim::Paren)),
                just(Token::Close(Delim::Paren)),
            )
            .or_not()
        )
        .map(|(name, values)| match values {
            Some(values) => EnumVariant { name, values },
            None => EnumVariant { name, values: Vec::new() },
        }); 

    let enum_ = just(Token::Def)
        .ignore_then(just(Token::Enum))
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("enum name"),
        )
        .then(generics_parser().or_not())
        .then_ignore(just(Token::Assign))
        //.then(just(Token::Assign)) //.padded()
        //.then(item.repeated())
        .then(
            variant.separated_by(just(Token::Pipe)).allow_leading().delimited_by(
                just(Token::Open(Delim::Block)),
                just(Token::Close(Delim::Block)),
            ),
        )
        .map(|((name, generics), variants)| Enum { name, generics, variants })
        .labelled("enum");

    enum_
}

pub fn type_parser() -> impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone {
    recursive(|type_| {
        let ident = filter_map(|span, tok| match tok {
            Token::Ident(ident) => Ok(ident.clone()),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
        });

        let generic_args = type_
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::Open(Delim::SquareBrackets)),
                just(Token::Close(Delim::SquareBrackets)),
            );
        // Attempt to recover anything that looks like a function body but contains errors
        /*
        .recover_with(nested_delimiters(
            Token::Open(Delim::SquareBrackets),
            Token::Close(Delim::SquareBrackets),
            [
                (Token::Open(Delim::Paren), Token::Close(Delim::Paren)),
                (Token::Open(Delim::Block), Token::Close(Delim::Block)),
                (Token::Open(Delim::CurlyBrackets), Token::Close(Delim::CurlyBrackets)),
            ],
            |span| vec![(Type::Error, span)],
        ))
        */

        ident
            .then(generic_args.or_not())
            //.ok_or_else(|| Vec::new())
            .map_with_span(|(name, generic_args), span: Span| match generic_args {
                Some(args) => (Type::Named(name, args), span.clone()),
                None => (Type::Named(name, Vec::new()), span.clone()),
            })
            .labelled("type")
    })
}

pub fn funcs_parser() -> impl Parser<Token, Func, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let item = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(|(name, ty)| Item { name, ty });

    // Argument lists are just identifiers separated by commas, surrounded by parentheses
    let args = item //ident
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(
            just(Token::Open(Delim::Paren)),
            just(Token::Close(Delim::Paren)),
        )
        .labelled("function args");

    let func = just(Token::Def)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then(args)
        .then_ignore(just(Token::Op(Op::RArrow)))
        .then(type_parser())
        .then(generics_parser().or_not())
        /*
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function return type"),
        )
        */
        .then_ignore(just(Token::Assign))
        //.then(just(Token::Assign)) //.padded()
        .then(
            expr_parser()
                //.repeated()
                .delimited_by(
                    just(Token::Open(Delim::Block)),
                    just(Token::Close(Delim::Block)),
                )
                // Attempt to recover anything that looks like a function body but contains errors
                .recover_with(nested_delimiters(
                    Token::Open(Delim::Block),
                    Token::Close(Delim::Block),
                    [
                        (Token::Open(Delim::Paren), Token::Close(Delim::Paren)),
                        (
                            Token::Open(Delim::SquareBrackets),
                            Token::Close(Delim::SquareBrackets),
                        ),
                        (
                            Token::Open(Delim::CurlyBrackets),
                            Token::Close(Delim::CurlyBrackets),
                        ),
                    ],
                    |span| (Expr::Error, span),
                )),
        )
        .map(|((((name, args), rtn), generics), body)| Func {
            name: name,
            args: args,
            rtn: rtn,
            generics: generics,
            body: vec![body],
        })
        .labelled("function");

    func
}
