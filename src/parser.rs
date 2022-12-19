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
                .allow_trailing(),
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
        .map(|((name, generics), members)| Struct {
            name,
            generics,
            members,
        })
        .labelled("struct");

    struct_
}

pub fn trait_parser() -> impl Parser<Token, Trait, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let item = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(|(name, ty)| Item { name, ty });

    let trait_ = just(Token::Def)
        .ignore_then(just(Token::Trait))
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("trait name"),
        )
        .then(generics_parser())
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
        .map(|((name, generics), members)| Trait {
            name,
            generics,
            members,
        })
        .labelled("trait");

    trait_
}

pub fn impl_parser() -> impl Parser<Token, Impl, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let impl_ = just(Token::Def)
        .ignore_then(just(Token::Impl))
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("trait impl name"),
        )
        .then_ignore(just(Token::For))
        .then(
            type_parser()
                .separated_by(just(Token::Comma))
                .allow_trailing(),
        )
        .then_ignore(just(Token::Assign))
        //.then(just(Token::Assign)) //.padded()
        //.then(item.repeated())
        .then(funcs_parser().repeated().delimited_by(
            just(Token::Open(Delim::Block)),
            just(Token::Close(Delim::Block)),
        ))
        .map(|((name, generic_args), funcs)| Impl {
            name,
            generic_args,
            funcs,
        })
        .labelled("trait impl");

    impl_
}

pub fn enum_parser() -> impl Parser<Token, Enum, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let variant = ident
        .clone()
        .map_with_span(|name, span| (name, span))
        .then(
            type_parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(
                    just(Token::Open(Delim::Paren)),
                    just(Token::Close(Delim::Paren)),
                )
                .or_not(),
        )
        .map(|(name, values)| match values {
            Some(values) => EnumVariant { name, values },
            None => EnumVariant {
                name,
                values: Vec::new(),
            },
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
            variant
                .separated_by(just(Token::Pipe))
                .allow_leading()
                .delimited_by(
                    just(Token::Open(Delim::Block)),
                    just(Token::Close(Delim::Block)),
                ),
        )
        .map(|((name, generics), variants)| Enum {
            name,
            generics,
            variants,
        })
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
            .clone()
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

        let named = ident
            .then(generic_args.or_not())
            //.ok_or_else(|| Vec::new())
            .map_with_span(|(name, generic_args), span: Span| match generic_args {
                Some(args) => (Type::Named(name, args), span.clone()),
                None => (Type::Named(name, Vec::new()), span.clone()),
            })
            .labelled("named type");

        let tuple_or_func = type_
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::Open(Delim::Paren)),
                just(Token::Close(Delim::Paren)),
            )
            .then(
                just(Token::Op(Op::RArrow))
                    .ignore_then(type_.clone())
                    .or_not(),
            )
            .map_with_span(|(args, maybe_rtn), span: Span| match maybe_rtn {
                Some(rtn) => (Type::Func(args, Box::new(rtn)), span.clone()),
                None => (Type::Tuple(args), span.clone()),
            })
            .labelled("function type");

        /*
        let tuple =  type_.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::Open(Delim::Paren)),
                just(Token::Close(Delim::Paren)),
            )
            .map_with_span(|members, span: Span| {
                (Type::Tuple(members), span.clone())
            })
            .labelled("tuple type");
        */

        named.or(tuple_or_func) // or(tuple).or(func)
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
            // expr_parser()
            //     //.repeated()
            //     .delimited_by(
            //         just(Token::Open(Delim::Block)),
            //         just(Token::Close(Delim::Block)),
            //     )
            //     // Attempt to recover anything that looks like a function body but contains errors
            //     .recover_with(nested_delimiters(
            //         Token::Open(Delim::Block),
            //         Token::Close(Delim::Block),
            //         [
            //             (Token::Open(Delim::Paren), Token::Close(Delim::Paren)),
            //             (
            //                 Token::Open(Delim::SquareBrackets),
            //                 Token::Close(Delim::SquareBrackets),
            //             ),
            //             (
            //                 Token::Open(Delim::CurlyBrackets),
            //                 Token::Close(Delim::CurlyBrackets),
            //             ),
            //         ],
            //         |span| (Expr::Error, span),
            //     )),
            assign_expr_parser(),
        )
        .map(|((((name, args), rtn), generics), body)| Func {
            name,
            args,
            rtn,
            generics,
            body: Box::new(body), //vec![body],
        })
        .labelled("function");

    func
}

pub fn const_parser() -> impl Parser<Token, Const, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let const_ = just(Token::Def)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .then(generics_parser().or_not())
        .then_ignore(just(Token::Assign))
        //.then(just(Token::Assign)) //.padded()
        .then(expr_parser())
        .map(|(((name, ty), generics), value)| Const {
            name,
            ty,
            generics,
            value: Box::new(value),
        })
        .labelled("const");

    const_
}

pub fn let_parser_rec(
    block: impl Parser<Token, Block, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, Let, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let const_ = just(Token::Let)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .then_ignore(just(Token::Assign))
        .then(assign_expr_parser_rec(block.clone()))
        .map(|((name, ty), value)| Let {
            name,
            ty,
            value: Box::new(value),
        })
        .labelled("let");

    const_
}

// NOTE: this is used by functions
pub fn control_expr_parser_rec(
    block: impl Parser<Token, Block, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, ControlExpr, Error = Simple<Token>> + Clone {
    //let block = statements_parser_rec(expr.clone());
    let expr = expr_parser();
    block
        .map(|block| ControlExpr::Block(block))
        .or(expr.map(|expr| ControlExpr::Expr(Box::new(expr))))
}

pub fn assign_expr_parser() -> impl Parser<Token, AssignExpr, Error = Simple<Token>> + Clone {
    let block = block_parser();
    assign_expr_parser_rec(block)
}

pub fn assign_expr_parser_rec(
    block: impl Parser<Token, Block, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, AssignExpr, Error = Simple<Token>> + Clone {
    //let block = statements_parser_rec(expr.clone());
    let expr = expr_parser();
    expr.clone()
        .map(|expr| AssignExpr::Expr(Box::new(expr)))
        .or(block.map(|block| AssignExpr::BlockExpr(Box::new(BlockExpr::Block(block)))))
}

// NOTE: this is used by exprs (e.g. block, for, if etc.)
pub fn block_parser() -> impl Parser<Token, Block, Error = Simple<Token>> + Clone {
    let expr = expr_parser();
    recursive(|block| {
        let ident = filter_map(|span, tok| match tok {
            Token::Ident(ident) => Ok(ident.clone()),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
        });

        let assign_expr = assign_expr_parser_rec(block.clone());

        let assign = ident
            .map_with_span(|name, span| (name, span))
            .labelled("identifier name")
            .then_ignore(just(Token::Assign))
            .then(assign_expr.clone())
            .map(|(name, value)| Assign {
                name,
                value: Box::new(value),
            })
            .labelled("assign");

        let let_ = let_parser_rec(block.clone());
        let stmt = assign_expr
            .clone()
            .map(|expr| Statement::Expr(Box::new(expr)))
            .or(let_.map(|let_| Statement::Let(Box::new(let_))))
            .or(assign.map(|assign| Statement::Assign(Box::new(assign))));
        let block = stmt
            .repeated()
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
                // TODO: add span to return
                |span| vec![Statement::Error],
            ))
            .map(|body| Block { body })
            .labelled("block");

        block
        // expr.clone()
        //     .map(|expr| BlockExpr::Expr(Box::new(expr)))
        //     .or(block)
    })
}
