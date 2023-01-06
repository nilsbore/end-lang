use crate::ast::*;
use crate::lexer::*;
use crate::utils::*;
use chumsky::prelude::*; //{prelude::*, BoxStream, Flat};

// pub fn lexpr_parser_rec<TExpr>(
//     expr: impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone,
// ) -> impl Parser<Token, Spanned<LExpr<TExpr>>, Error = Simple<Token>> + Clone {
//     let ident = filter_map(|span, tok| match tok {
//         Token::Ident(ident) => Ok(ident.clone()),
//         _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
//     });
//     let type_ = type_parser();

//     let qualified_ident = ident
//         .map_with_span(|name, span| (name, span))
//         .labelled("ident path")
//         .separated_by(just(Token::DoubleColon))
//         .then_ignore(just(Token::DoubleColon))
//         .or_not()
//         .then(type_.then_ignore(just(Token::DoubleColon)).or_not())
//         .then(
//             ident
//                 .map_with_span(|name, span| (name, span))
//                 .labelled("ident name"),
//         )
//         .map(|((path, ty), ident)| match path {
//             None => QualifiedIdent {
//                 path: Vec::new(),
//                 ty,
//                 ident,
//             },
//             Some(path) => QualifiedIdent { path, ty, ident },
//         });

//     let items = expr
//         .clone()
//         .separated_by(just(Token::Comma))
//         .allow_trailing();

//     //let member = atom.then_ignore(just(Token::Op(Op::Dot))).then(ident)
//     let access = qualified_ident
//         .clone()
//         .then_ignore(just(Token::Op(Op::Dot)))
//         .then(
//             ident
//                 .map_with_span(|name, span| (name, span))
//                 .labelled("member name"),
//         )
//         .map_with_span(|(expr, member), span| (LExpr::Member(Box::new(expr), member), span));

//     let call = qualified_ident
//         .clone()
//         .or(access.clone())
//         .then(
//             items.delimited_by(
//                 just(Token::Open(Delim::Paren)),
//                 just(Token::Close(Delim::Paren)),
//             ), //.map_with_span(|args, span: Span| (args, span)),
//                //.or_not(),
//         )
//         //.map(|(expr, args): (_, Option<Vec<(_, Span)>>)| match args {
//         .map(|(expr, args): (_, Vec<(_, Span)>)| {
//             let call_span: Span = expr.1.start..args.last().unwrap().1.end;
//             (Expr::Call(Box::new(expr), args), call_span)
//         }); // Expr::Call(Box::new(expr), args));
//             // .foldl(|f, args| {
//             //     let span = f.1.start..args.1.end;
//             //     (Expr::Call(Box::new(f), args.0), span)
//             // });

//     qualified_ident.or(access).or(call)
// }

// pub fn llexpr_parser() -> impl Parser<Token, LLExpr, Error = Simple<Token>> + Clone {
//     recursive(|expr| lexpr_parser_rec(expr)).map(|expr| LLExpr { expr })
// }

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let ident = filter_map(|span, tok| match tok {
            Token::Ident(ident) => Ok(ident.clone()),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
        });

        let literal = select! {
            //Token::Null => Expr::Value(Value::Null),
            Token::Bool(x) => Expr::Literal(Literal::Bool(x)),
            Token::Int(n) => Expr::Literal(Literal::Nat(n)),
            Token::Str(s) => Expr::Literal(Literal::Str(s)),
        }
        .labelled("value");

        let type_ = type_parser();

        let qualified_ident = ident
            .map_with_span(|name, span| (name, span))
            .labelled("ident path")
            .separated_by(just(Token::DoubleColon))
            .then_ignore(just(Token::DoubleColon))
            .or_not()
            .then(type_.then_ignore(just(Token::DoubleColon)).or_not())
            .then(
                ident
                    .map_with_span(|name, span| (name, span))
                    .labelled("ident name"),
            )
            .map(|((path, ty), ident)| match path {
                None => QualifiedIdent {
                    path: Vec::new(),
                    ty,
                    ident,
                },
                Some(path) => QualifiedIdent { path, ty, ident },
            })
            .map(Expr::Ident) //|ident| Expr::Ident(ident))
            // TODO: improve this??
            .map_with_span(|name, span| (name, span))
            .labelled("ident name");

        let items = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing();

        let call_items = ident
            .map_with_span(|name, span| (name, span))
            .labelled("keyword arg")
            .then_ignore(just(Token::Assign))
            .or_not()
            .then(expr.clone())
            .separated_by(just(Token::Comma))
            .allow_trailing();

        //let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");
        //let atom = literal.map_with_span(|expr, span: Span| (expr, span));
        let list = items
            .clone()
            .delimited_by(
                just(Token::Open(Delim::SquareBrackets)),
                just(Token::Close(Delim::SquareBrackets)),
            )
            .map(|items: Vec<(_, Span)>| {
                let span: Span = items.first().unwrap().1.start..items.last().unwrap().1.end;
                (Expr::List(items), span)
            });

        let atom = literal
            .map_with_span(|expr, span: Span| (expr, span))
            .or(qualified_ident)
            .or(list)
            // .or(
            //     expr.clone()
            //         .delimited_by(just(Token::Open(Delim::Paren)), just(Token::Close(Delim::Paren)))
            // )
            // Attempt to recover anything that looks like a parenthesised expression but contains errors
            .recover_with(nested_delimiters(
                Token::Open(Delim::Paren),
                Token::Close(Delim::Paren),
                [
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
            ))
            // Attempt to recover anything that looks like a list but contains errors
            .recover_with(nested_delimiters(
                Token::Open(Delim::SquareBrackets),
                Token::Close(Delim::SquareBrackets),
                [
                    (Token::Open(Delim::Paren), Token::Close(Delim::Paren)),
                    (
                        Token::Open(Delim::CurlyBrackets),
                        Token::Close(Delim::CurlyBrackets),
                    ),
                ],
                |span| (Expr::Error, span),
            ));

        //let member = atom.then_ignore(just(Token::Op(Op::Dot))).then(ident)
        let access = atom
            .clone()
            .then_ignore(just(Token::Op(Op::Dot)))
            .then(
                ident
                    .map_with_span(|name, span| (name, span))
                    .labelled("member name"),
            )
            .map_with_span(|(expr, member), span| (Expr::Member(Box::new(expr), member), span));

        let call = atom
            .clone()
            .or(access.clone())
            .then(
                call_items.delimited_by(
                    just(Token::Open(Delim::Paren)),
                    just(Token::Close(Delim::Paren)),
                ), //.map_with_span(|args, span: Span| (args, span)),
                   //.or_not(),
            )
            //.map(|(expr, args): (_, Option<Vec<(_, Span)>>)| match args {
            .map(|(expr, args)| {
                // TODO: note that args may be empty
                //let call_span: Span = expr.1.start..args.last().unwrap().1.end;
                Expr::Call(Box::new(expr), args)
            })
            .map_with_span(|val, span| (val, span));
        // Expr::Call(Box::new(expr), args));
        // .foldl(|f, args| {
        //     let span = f.1.start..args.1.end;
        //     (Expr::Call(Box::new(f), args.0), span)
        // });

        let val = call.or(access).or(atom);

        // Product ops (multiply and divide) have equal precedence
        let op = just(Token::Op(Op::Mul))
            .to(BinaryOp::Mul)
            .or(just(Token::Op(Op::Div)).to(BinaryOp::Div));
        let product = val
            .clone()
            .then(op.then(val).repeated())
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
            .or(just(Token::Op(Op::Less)).to(BinaryOp::Less))
            .or(just(Token::Op(Op::LessEq)).to(BinaryOp::LessEq))
            .or(just(Token::Op(Op::More)).to(BinaryOp::Greater))
            .or(just(Token::Op(Op::MoreEq)).to(BinaryOp::GreaterEq))
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

pub fn match_parser(
    block: impl Parser<Token, Block, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, MatchExpr, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let pattern = recursive(|pattern| {
        ident
            .clone()
            .map_with_span(|name, span| (name, span))
            .then(
                pattern
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(
                        just(Token::Open(Delim::Paren)),
                        just(Token::Close(Delim::Paren)),
                    )
                    .or_not(),
            )
            .map(|(ident, members)| Pattern { ident, members })
    });

    let expr = expr_parser();

    let control_expr = block
        .clone()
        .map(|block| ControlExpr::Block(block))
        .or(expr.clone().map(Box::new).map(ControlExpr::Expr));

    let match_arms = just(Token::Pipe)
        .ignore_then(pattern)
        .then_ignore(just(Token::RFlow))
        .then(control_expr.map(Box::new))
        .map(|(pattern, body)| MatchBranch { pattern, body })
        .repeated()
        .delimited_by(
            just(Token::Open(Delim::Block)),
            just(Token::Close(Delim::Block)),
        );

    let match_ = just(Token::Match)
        .ignore_then(expr.clone().map(Box::new))
        .then_ignore(just(Token::Colon))
        .then(match_arms)
        .map(|(expr, branches)| MatchExpr { expr, branches })
        .labelled("match expr");

    match_
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

pub fn assign_expr_parser() -> impl Parser<Token, AssignExpr, Error = Simple<Token>> + Clone {
    let block = block_parser();
    assign_expr_parser_rec(block)
}

pub fn assign_expr_parser_rec(
    block: impl Parser<Token, Block, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, AssignExpr, Error = Simple<Token>> + Clone {
    //let block = statements_parser_rec(expr.clone());
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });
    let expr = expr_parser();
    let expr_block = expr.clone().map(|expr| AssignExpr::Expr(Box::new(expr)));
    let control_expr = block
        .clone()
        .map(|block| ControlExpr::Block(block))
        .or(expr.clone().map(Box::new).map(ControlExpr::Expr));

    let if_ = just(Token::If)
        .ignore_then(expr.clone())
        .then_ignore(just(Token::Colon))
        .then(control_expr.clone())
        .then(
            just(Token::Elif)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::Colon))
                .then(control_expr.clone())
                .repeated(),
        )
        .then(
            just(Token::Else)
                .then(just(Token::Colon))
                .ignore_then(control_expr.clone())
                .or_not(),
        )
        .map(|(((main_expr, main_body), elifs), else_body)| {
            BlockExpr::If(IfExpr {
                main_expr: Box::new(main_expr),
                main_body: Box::new(main_body),
                elif_expr_body: elifs,
                else_body: Box::new(else_body),
            })
        });

    let while_ = just(Token::While)
        .ignore_then(expr.clone())
        .then_ignore(just(Token::Colon))
        .then(control_expr.clone())
        .map(|(expr, body)| {
            BlockExpr::While(WhileExpr {
                expr: Box::new(expr),
                body: Box::new(body),
            })
        });

    let for_ = just(Token::For)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("for loop var"),
        )
        .then_ignore(just(Token::In))
        .then(expr.clone())
        .then_ignore(just(Token::Colon))
        .then(control_expr.clone())
        .map(|((ident, expr), body)| {
            BlockExpr::For(ForExpr {
                ident: ident,
                expr: Box::new(expr),
                body: Box::new(body),
            })
        });

    let match_ = match_parser(block.clone()).map(BlockExpr::Match);

    let block_ = block.clone().map(|block| BlockExpr::Block(block));

    expr_block.or(while_
        .or(block_)
        .or(if_)
        .or(for_)
        .or(match_)
        .map(|block_expr| AssignExpr::BlockExpr(Box::new(block_expr))))
}

// NOTE: this is used by exprs (e.g. block, for, if etc.)
pub fn block_parser() -> impl Parser<Token, Block, Error = Simple<Token>> + Clone {
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
        let stmt = let_
            .map(|let_| Statement::Let(Box::new(let_)))
            .or(assign.map(|assign| Statement::Assign(Box::new(assign))))
            .or(assign_expr
                .clone()
                .map(|expr| Statement::Expr(Box::new(expr))));
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

pub fn program_parser() -> impl Parser<Token, Program, Error = Simple<Token>> + Clone {
    funcs_parser()
        .map(Def::Func)
        .or(enum_parser().map(Def::Enum))
        .or(struct_parser().map(Def::Struct))
        .or(impl_parser().map(Def::Impl))
        .or(trait_parser().map(Def::Trait))
        .or(const_parser().map(Def::Const))
        .repeated()
        .then_ignore(end())
        .map(|defs| Program { defs })
}
