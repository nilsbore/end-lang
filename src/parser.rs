use crate::ast::*;
use crate::lexer::*;
use crate::utils::*;
use chumsky::prelude::*; //{prelude::*, BoxStream, Flat};

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

        let qualified_ident =
            qualified_type_parser().map(|(ident, span)| (Expr::QualifiedIdent(ident), span));

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

        let object_items = ident
            .map_with_span(|name, span| (name, span))
            .then_ignore(just(Token::Assign))
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
                (Expr::Tuple(items), span)
            });

        let object = object_items
            .clone()
            .delimited_by(
                just(Token::Open(Delim::CurlyBrackets)),
                just(Token::Close(Delim::CurlyBrackets)),
            )
            .map(|args| Expr::Object(args))
            .map_with_span(|val, span| (val, span));

        let atom = literal
            .map_with_span(|expr, span: Span| (expr, span))
            .or(qualified_ident)
            .or(list)
            .or(object)
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

        let call = access
            .clone()
            .or(atom.clone())
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

pub fn struct_parser(
    type_: impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, Spanned<Vec<Item>>, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let item = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(type_)
        .map(|(name, ty)| Item { name, ty });

    item.separated_by(just(Token::Comma))
        .allow_trailing()
        // .delimited_by(
        //     just(Token::Open(Delim::Block)),
        //     just(Token::Close(Delim::Block)),
        // )
        .delimited_by(
            just(Token::Open(Delim::CurlyBrackets)),
            just(Token::Close(Delim::CurlyBrackets)),
        )
        .map_with_span(|items, span| (items, span))
    /*
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
            item.separated_by(just(Token::Comma))
                .allow_trailing()
                // .delimited_by(
                //     just(Token::Open(Delim::Block)),
                //     just(Token::Close(Delim::Block)),
                // )
                .delimited_by(
                    just(Token::Open(Delim::CurlyBrackets)),
                    just(Token::Close(Delim::CurlyBrackets)),
                ), //     repeated().delimited_by(
                   //     just(Token::Open(Delim::Block)),
                   //     just(Token::Close(Delim::Block)),
                   // ),
                   /*
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
            exported: false,
        })
        .labelled("trait");

    trait_
}

// pub fn impl_parser() -> impl Parser<Token, Impl, Error = Simple<Token>> + Clone {
//     let ident = filter_map(|span, tok| match tok {
//         Token::Ident(ident) => Ok(ident.clone()),
//         _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
//     });

//     let impl_ = just(Token::Def)
//         .ignore_then(just(Token::Impl))
//         .ignore_then(
//             ident
//                 .map_with_span(|name, span| (name, span))
//                 .labelled("trait impl name"),
//         )
//         .then_ignore(just(Token::For))
//         .then(
//             type_parser()
//                 .separated_by(just(Token::Comma))
//                 .allow_trailing(),
//         )
//         .then_ignore(just(Token::Assign))
//         //.then(just(Token::Assign)) //.padded()
//         //.then(item.repeated())
//         .then(funcs_parser().repeated().delimited_by(
//             just(Token::Open(Delim::Block)),
//             just(Token::Close(Delim::Block)),
//         ))
//         .map(|((name, generic_args), funcs)| Impl {
//             name,
//             generic_args,
//             funcs,
//         })
//         .labelled("trait impl");

//     impl_
// }

pub fn def_type_parser() -> impl Parser<Token, DefType, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let deftype = just(Token::Def)
        .ignore_then(just(Token::Type))
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("enum name"),
        )
        .then(generics_parser().or_not())
        .then_ignore(just(Token::Assign))
        .then(type_parser())
        .map(|((name, generics), ty)| DefType { name, generics, ty, exported: false })
        .labelled("deftype");

    deftype
}

pub fn enum_parser(
    type_: impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, Spanned<Vec<EnumVariant>>, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let variant = ident
        .clone()
        .map_with_span(|name, span| (name, span))
        .then(
            type_
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

    variant
        .separated_by(just(Token::Pipe))
        .allow_leading()
        .delimited_by(
            just(Token::Open(Delim::Block)),
            just(Token::Close(Delim::Block)),
        )
        .map_with_span(|v, span| (v, span))
}

pub fn match_parser(
    block: impl Parser<Token, Block, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, MatchExpr, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let pattern = recursive(|pattern| {
        // Base case: simple identifier pattern
        let simple_pattern = ident
            .clone()
            .map_with_span(|name, span| (name, span))
            .map(|ident| Pattern { ident, members: None });

        // Recursive case: identifier with nested patterns
        let complex_pattern = ident
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
            )
            .map(|(ident, members)| Pattern { ident, members: Some(members) });

        // Try complex pattern first, fall back to simple pattern
        complex_pattern.or(simple_pattern)
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
        .repeated();

    let match_ = just(Token::Match)
        .ignore_then(expr.clone().map(Box::new))
        .then_ignore(just(Token::Colon))
        .then(
            match_arms
                .clone()
                .delimited_by(
                    just(Token::Open(Delim::Block)),
                    just(Token::Close(Delim::Block)),
                )
                .or(match_arms),
        )
        .map(|(expr, branches)| MatchExpr { expr, branches })
        .labelled("match expr");

    match_
}

pub fn type_parser() -> impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone {
    recursive(|type_| type_parser_rec(type_))
}

pub fn literal_type_parser() -> impl Parser<Token, LiteralType, Error = Simple<Token>> + Clone {
    select! {
        //Token::Null => Expr::Value(Value::Null),
        Token::BoolType => LiteralType::Bool,
        Token::IntType => LiteralType::Int,
        Token::NatType => LiteralType::Nat,
        Token::RealType => LiteralType::Real,
        Token::CharType => LiteralType::Char,
        Token::StrType => LiteralType::Str,
    }
    .labelled("value")
}

pub fn type_parser_rec(
    type_: impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone {
    // let ident = filter_map(|span, tok| match tok {
    //     Token::Ident(ident) => Ok(ident.clone()),
    //     _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    // });

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

    // FIXME: this should be qualified ident instead, and that
    // needs to take type as input since it might be generic
    // that means that we need a _rec version of qualified_ident
    let universe_type = just(Token::Type).map_with_span(|_, span| (Type::UniverseType, span));
    // let named = ident
    //     .then(generic_args.or_not())
    //     //.ok_or_else(|| Vec::new())
    //     .map_with_span(|(name, generic_args), span: Span| match generic_args {
    //         Some(args) => (Type::Named(name, args), span.clone()),
    //         None => (Type::Named(name, Vec::new()), span.clone()),
    //     })
    //     .labelled("named type");
    let literal_type =
        literal_type_parser().map_with_span(|literal, span| (Type::Literal(literal), span));
    let qualified_type = qualified_type_parser_rec(type_.clone()).map(|(v, s)| (Type::Ident(v), s));

    let tuple_list = type_
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(
            just(Token::Open(Delim::Paren)),
            just(Token::Close(Delim::Paren)),
        );

    let tuple = tuple_list
        .clone()
        .map_with_span(|args, span: Span| (Type::Tuple(args), span.clone()))
        .labelled("function type");

    // we need to make these be spanned
    let enum_ = enum_parser(type_.clone()).map(|(e, span)| (Type::Enum(e), span));
    
    // Convert struct syntax to field constraint intersections: {x: i32, y: i32} → {x: i32}^{y: i32}
    let struct_ = struct_parser(type_.clone()).map(|(items, span)| {
        if items.is_empty() {
            // Empty struct - create empty intersection (could be enhanced)
            // For now, we could use a unit type or special empty intersection
            // Using Unknown as placeholder for empty intersection
            (Type::Unknown, span)
        } else if items.len() == 1 {
            // Single field - create single FieldConstraint
            let item = &items[0];
            (Type::FieldConstraint(item.name.clone(), Box::new(item.ty.clone())), span)
        } else {
            // Multiple fields - create intersection of FieldConstraints: {x: T1}^{y: T2}^...
            let field_constraints: Vec<Type> = items.into_iter()
                .map(|item| Type::FieldConstraint(item.name, Box::new(item.ty)))
                .collect();
            
            // Build intersection from right to left
            let intersection = field_constraints.into_iter()
                .reduce(|acc, field_constraint| {
                    Type::Intersection(Box::new(field_constraint), Box::new(acc))
                })
                .unwrap();
            
            (intersection, span)
        }
    });

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

    // enum shouldn't be a problem:
    // Some(Person^Print) | None
    // func will be a problem:
    // (a: str) -> A^B
    // the solution should be to let
    // ^ have the highest priority

    // Let's think of these as operators instead
    // we have the following:
    // ->, ^
    // ^ needs to have highest priority
    // -> 2nd highest

    // Forward declaration support: "pass" creates Type::Unknown
    let pass_type = filter_map(|span, tok| match tok {
        Token::Ident(ident) if ident == "pass" => Ok((Type::Unknown, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    let simple_type = universe_type
        .or(literal_type)
        .or(pass_type)
        .or(qualified_type)
        .or(tuple)
        .or(enum_)
        .or(struct_); // or(tuple).or(func)

    let intersection_type = simple_type
        .clone()
        .then(just(Token::Op(Op::Hat)).ignore_then(simple_type).repeated())
        .foldl(|a, b| {
            let span = a.1.start..b.1.end;
            (Type::Intersection(Box::new(a.0), Box::new(b.0)), span)
        });

    let func = intersection_type
        .clone()
        .then(
            just(Token::Op(Op::RArrow))
                .ignore_then(intersection_type)
                .repeated(),
        )
        .foldl(|a, b| {
            let span = a.1.start..b.1.end;
            (Type::Func(Box::new(a), Box::new(b)), span)
        });

    // let func = tuple_list
    //     .then(just(Token::Op(Op::RArrow)).ignore_then(type_.clone()))
    //     .map_with_span(|(args, rtn), span: Span| (Type::Func(args, Box::new(rtn)), span.clone()))
    //     .labelled("function type");

    func
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

    // let qualified_ident = qualified_ident_parser();
    let ty = type_parser();

    let func = just(Token::Def)
        .ignore_then(ty.then_ignore(just(Token::Op(Op::Dot))).or_not())
        .then(ident)
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
        .map(|(((((ty, ident), args), rtn), generics), body)| Func {
            ty,
            ident,
            args,
            rtn,
            generics,
            body: Box::new(body), //vec![body],
            exported: false,
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
            exported: false,
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

pub fn qualified_type_parser(
) -> impl Parser<Token, Spanned<QualifiedType>, Error = Simple<Token>> + Clone {
    let type_ = type_parser();
    qualified_type_parser_rec(type_)
}

pub fn qualified_type_parser_rec(
    type_: impl Parser<Token, Spanned<Type>, Error = Simple<Token>> + Clone,
) -> impl Parser<Token, Spanned<QualifiedType>, Error = Simple<Token>> + Clone {
    // let type_ = type_parser();

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
    // let named = ident
    //     .then(generic_args.or_not())
    //     //.ok_or_else(|| Vec::new())
    //     .map_with_span(|(name, generic_args), span: Span| match generic_args {
    //         Some(args) => (Type::Named(name, args), span.clone()),
    //         None => (Type::Named(name, Vec::new()), span.clone()),
    //     })
    //     .labelled("named type");

    ident
        .map_with_span(|name, span| (name, span))
        .labelled("ident path")
        .separated_by(just(Token::DoubleColon))
        .at_least(1) // .allow_trailing()
        .then(generic_args.or_not())
        .map(|(path, type_args)| match type_args {
            None => QualifiedType {
                path,
                type_args: Vec::new(),
            },
            Some(type_args) => QualifiedType { path, type_args },
        })
        // .map(Expr::Ident) //|ident| Expr::Ident(ident))
        // TODO: improve this??
        .map_with_span(|name, span| (name, span))
        .labelled("ident name")
}

pub fn open_parser() -> impl Parser<Token, Open, Error = Simple<Token>> + Clone {
    just(Token::Begin)
        .ignore_then(type_parser())
        .then(generics_parser().or_not())
        .map(|(ty, generics)| Open { ty, generics, exported: false })
        .labelled("open namespace")
}

pub fn close_parser() -> impl Parser<Token, Close, Error = Simple<Token>> + Clone {
    just(Token::End)
        .ignore_then(type_parser())
        .map(|ty| Close { ty })
        .labelled("close namespace")
}

// Export-aware parsers that handle both regular and exported definitions
pub fn exportable_func_parser() -> impl Parser<Token, Func, Error = Simple<Token>> + Clone {
    let regular = funcs_parser();
    let exported = just(Token::Export)
        .ignore_then(funcs_parser())
        .map(|mut func| {
            func.exported = true;
            func
        });

    exported.or(regular)
}

pub fn exportable_def_type_parser() -> impl Parser<Token, DefType, Error = Simple<Token>> + Clone {
    let regular = def_type_parser();
    let exported = just(Token::Export)
        .ignore_then(def_type_parser())
        .map(|mut def_type| {
            def_type.exported = true;
            def_type
        });

    exported.or(regular)
}

pub fn exportable_trait_parser() -> impl Parser<Token, Trait, Error = Simple<Token>> + Clone {
    let regular = trait_parser();
    let exported = just(Token::Export)
        .ignore_then(trait_parser())
        .map(|mut trait_def| {
            trait_def.exported = true;
            trait_def
        });

    exported.or(regular)
}

pub fn exportable_const_parser() -> impl Parser<Token, Const, Error = Simple<Token>> + Clone {
    let regular = const_parser();
    let exported = just(Token::Export)
        .ignore_then(const_parser())
        .map(|mut const_def| {
            const_def.exported = true;
            const_def
        });

    exported.or(regular)
}

pub fn exportable_open_parser() -> impl Parser<Token, Open, Error = Simple<Token>> + Clone {
    let regular = open_parser();
    let exported = just(Token::Export)
        .ignore_then(open_parser())
        .map(|mut open_def| {
            open_def.exported = true;
            open_def
        });

    exported.or(regular)
}

// Import statement parsers
pub fn from_import_parser() -> impl Parser<Token, FromImport, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    // Parse module path like: math.geometry.shapes
    let module_path = ident
        .clone()
        .separated_by(just(Token::Op(Op::Dot)))
        .at_least(1);

    // Parse import list like: Vector, Circle, Add^Number
    let import_item = ident.clone()
        .then(
            just(Token::Op(Op::Hat))
                .ignore_then(ident.clone())
                .or_not()
        )
        .map(|(base, enhancement)| {
            if let Some(enh) = enhancement {
                format!("{}^{}", enh, base)
            } else {
                base
            }
        });

    let import_list = import_item
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .at_least(1);

    just(Token::From)
        .ignore_then(module_path)
        .then_ignore(just(Token::Import))
        .then(import_list)
        .map(|(module_path, items)| FromImport {
            module_path,
            items,
        })
        .labelled("from import")
}

pub fn direct_import_parser() -> impl Parser<Token, DirectImport, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    // Parse module.path.item
    let full_path = ident
        .clone()
        .separated_by(just(Token::Op(Op::Dot)))
        .at_least(2); // At least module.item

    just(Token::Import)
        .ignore_then(full_path)
        .map(|path| {
            let mut path_parts = path;
            let item = path_parts.pop().unwrap(); // Last part is the item
            DirectImport {
                module_path: path_parts,
                item,
            }
        })
        .labelled("direct import")
}

pub fn import_statement_parser() -> impl Parser<Token, ImportStatement, Error = Simple<Token>> + Clone {
    from_import_parser()
        .map(ImportStatement::From)
        .or(direct_import_parser().map(ImportStatement::Direct))
        .labelled("import statement")
}

pub fn program_parser() -> impl Parser<Token, Program, Error = Simple<Token>> + Clone {
    import_statement_parser()
        .map(Def::Import)
        .or(exportable_func_parser().map(Def::Func))
        // .or(enum_parser().map(Def::Enum))
        // .or(struct_parser().map(Def::Struct))
        .or(exportable_def_type_parser().map(Def::Type))
        // .or(impl_parser().map(Def::Impl))
        .or(exportable_trait_parser().map(Def::Trait))
        .or(exportable_const_parser().map(Def::Const))
        .or(exportable_open_parser().map(Def::Open))
        .or(close_parser().map(Def::Close))
        .repeated()
        .then_ignore(end())
        .map(|defs| Program { defs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{lexer, tts_to_stream};

    fn parse_test_program(input: &str) -> Result<Program, Vec<Simple<Token>>> {
        let (tts, _) = lexer().parse_recovery(input);
        if let Some(tts) = tts {
            let eoi = 0..input.chars().count();
            let token_stream = tts_to_stream(eoi, tts);
            let (result, errs) = program_parser().parse_recovery(token_stream);
            if errs.is_empty() {
                Ok(result.unwrap())
            } else {
                Err(errs)
            }
        } else {
            Err(vec![])
        }
    }

    fn parse_test_import(input: &str) -> Result<ImportStatement, Vec<Simple<Token>>> {
        let (tts, _) = lexer().parse_recovery(input);
        if let Some(tts) = tts {
            let eoi = 0..input.chars().count();
            let token_stream = tts_to_stream(eoi, tts);
            let (result, errs) = import_statement_parser().parse_recovery(token_stream);
            if errs.is_empty() {
                Ok(result.unwrap())
            } else {
                Err(errs)
            }
        } else {
            Err(vec![])
        }
    }

    #[test]
    fn test_export_type_definition() {
        let input = "export def type Number = {value: f64}";
        let program = parse_test_program(input).expect("Should parse export type");

        match &program.defs[0] {
            Def::Type(type_def) => {
                assert_eq!(type_def.name.0, "Number");
                assert!(type_def.exported, "Type should be exported");
            }
            _ => panic!("Expected type definition"),
        }
    }

    #[test]
    fn test_private_type_definition() {
        let input = "def type LocalType = {value: i32}";
        let program = parse_test_program(input).expect("Should parse private type");

        match &program.defs[0] {
            Def::Type(type_def) => {
                assert_eq!(type_def.name.0, "LocalType");
                assert!(!type_def.exported, "Type should be private");
            }
            _ => panic!("Expected type definition"),
        }
    }

    #[test]
    fn test_export_function_definition() {
        let input = "export def add(a: i32, b: i32) -> i32 = a + b";
        let program = parse_test_program(input).expect("Should parse export function");

        match &program.defs[0] {
            Def::Func(func) => {
                assert_eq!(func.ident, "add");
                assert!(func.exported, "Function should be exported");
                assert_eq!(func.args.len(), 2);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_private_function_definition() {
        let input = "def localHelper(x: i32) -> i32 = x + 1";
        let program = parse_test_program(input).expect("Should parse private function");

        match &program.defs[0] {
            Def::Func(func) => {
                assert_eq!(func.ident, "localHelper");
                assert!(!func.exported, "Function should be private");
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_export_open_block() {
        let input = "export open Print^Number";
        let program = parse_test_program(input).expect("Should parse export open");

        match &program.defs[0] {
            Def::Open(open) => {
                assert!(open.exported, "Open block should be exported");
                match &open.ty.0 {
                    Type::Intersection(_, _) => {}, // Expected intersection type
                    _ => panic!("Expected intersection type"),
                }
            }
            _ => panic!("Expected open definition"),
        }
    }

    #[test]
    fn test_private_open_block() {
        let input = "open LocalTrait^Number";
        let program = parse_test_program(input).expect("Should parse private open");

        match &program.defs[0] {
            Def::Open(open) => {
                assert!(!open.exported, "Open block should be private");
            }
            _ => panic!("Expected open definition"),
        }
    }

    #[test]
    fn test_export_const_definition() {
        let input = "export def answer: i32 = 42";
        let program = parse_test_program(input).expect("Should parse export const");

        match &program.defs[0] {
            Def::Const(const_def) => {
                assert_eq!(const_def.name.0, "answer");
                assert!(const_def.exported, "Const should be exported");
            }
            _ => panic!("Expected const definition"),
        }
    }

    #[test]
    fn test_from_import_simple() {
        let input = "from math import Number, Vector";
        let import = parse_test_import(input).expect("Should parse from import");

        match import {
            ImportStatement::From(from_import) => {
                assert_eq!(from_import.module_path, vec!["math"]);
                assert_eq!(from_import.items, vec!["Number", "Vector"]);
            }
            _ => panic!("Expected from import"),
        }
    }

    #[test]
    fn test_from_import_nested_module() {
        let input = "from math.geometry.shapes import Circle, Rectangle, Triangle";
        let import = parse_test_import(input).expect("Should parse nested from import");

        match import {
            ImportStatement::From(from_import) => {
                assert_eq!(from_import.module_path, vec!["math", "geometry", "shapes"]);
                assert_eq!(from_import.items, vec!["Circle", "Rectangle", "Triangle"]);
            }
            _ => panic!("Expected from import"),
        }
    }

    #[test]
    fn test_from_import_with_enhancements() {
        let input = "from utils import Print^Number, Scale^Vector";
        let import = parse_test_import(input).expect("Should parse enhancement import");

        match import {
            ImportStatement::From(from_import) => {
                assert_eq!(from_import.module_path, vec!["utils"]);
                assert_eq!(from_import.items, vec!["Number^Print", "Vector^Scale"]);
            }
            _ => panic!("Expected from import"),
        }
    }

    #[test]
    fn test_direct_import_simple() {
        let input = "import math.Number";
        let import = parse_test_import(input).expect("Should parse direct import");

        match import {
            ImportStatement::Direct(direct_import) => {
                assert_eq!(direct_import.module_path, vec!["math"]);
                assert_eq!(direct_import.item, "Number");
            }
            _ => panic!("Expected direct import"),
        }
    }

    #[test]
    fn test_direct_import_nested() {
        let input = "import geometry.shapes.Triangle";
        let import = parse_test_import(input).expect("Should parse nested direct import");

        match import {
            ImportStatement::Direct(direct_import) => {
                assert_eq!(direct_import.module_path, vec!["geometry", "shapes"]);
                assert_eq!(direct_import.item, "Triangle");
            }
            _ => panic!("Expected direct import"),
        }
    }

    #[test]
    fn test_mixed_export_import_program() {
        let input = r#"
from math import Number
export def type Counter = {value: Number}
import geometry.Circle
export def increment(c: Counter) -> Counter = Counter(value = c.value + 1)
def localHelper() -> i32 = 42
        "#;

        let program = parse_test_program(input).expect("Should parse mixed program");
        assert_eq!(program.defs.len(), 5);

        // Check import
        match &program.defs[0] {
            Def::Import(ImportStatement::From(from_import)) => {
                assert_eq!(from_import.module_path, vec!["math"]);
                assert_eq!(from_import.items, vec!["Number"]);
            }
            _ => panic!("Expected from import"),
        }

        // Check exported type
        match &program.defs[1] {
            Def::Type(type_def) => {
                assert_eq!(type_def.name.0, "Counter");
                assert!(type_def.exported);
            }
            _ => panic!("Expected exported type"),
        }

        // Check direct import
        match &program.defs[2] {
            Def::Import(ImportStatement::Direct(direct_import)) => {
                assert_eq!(direct_import.module_path, vec!["geometry"]);
                assert_eq!(direct_import.item, "Circle");
            }
            _ => panic!("Expected direct import"),
        }

        // Check exported function
        match &program.defs[3] {
            Def::Func(func) => {
                assert_eq!(func.ident, "increment");
                assert!(func.exported);
            }
            _ => panic!("Expected exported function"),
        }

        // Check private function
        match &program.defs[4] {
            Def::Func(func) => {
                assert_eq!(func.ident, "localHelper");
                assert!(!func.exported);
            }
            _ => panic!("Expected private function"),
        }
    }

    #[test]
    fn test_complex_enhancement_imports() {
        let input = "from math.advanced import Add^Number, Multiply^Vector, Print^Circle";
        let import = parse_test_import(input).expect("Should parse complex enhancement import");

        match import {
            ImportStatement::From(from_import) => {
                assert_eq!(from_import.module_path, vec!["math", "advanced"]);
                assert_eq!(from_import.items, vec!["Number^Add", "Vector^Multiply", "Circle^Print"]);
            }
            _ => panic!("Expected from import"),
        }
    }

    #[test]
    fn test_export_trait_definition() {
        let input = "export def trait Print for T: type = { print: (T) -> String }";
        let program = parse_test_program(input).expect("Should parse export trait");

        match &program.defs[0] {
            Def::Trait(trait_def) => {
                assert_eq!(trait_def.name.0, "Print");
                assert!(trait_def.exported, "Trait should be exported");
            }
            _ => panic!("Expected trait definition"),
        }
    }

    #[test]
    fn test_invalid_import_parsing() {
        // Test that invalid syntax properly fails
        let invalid_inputs = vec![
            "from", // Incomplete
            "import", // Incomplete
            "from math", // Missing import keyword
            "import math", // Missing item (at least module.item required)
            "from math import", // Missing items
        ];

        for input in invalid_inputs {
            let result = parse_test_import(input);
            assert!(result.is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_export_keyword_combinations() {
        // Test that export only works with valid keywords
        let valid_inputs = vec![
            "export def type A = {x: i32}",
            "export def func() -> i32 = 42",
            "export def const: i32 = 42",
            "export open Print^A",
        ];

        for input in valid_inputs {
            let result = parse_test_program(input);
            assert!(result.is_ok(), "Should parse: {}", input);
        }
    }
}
