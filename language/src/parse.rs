use core::f64;
use std::{
    cell::RefCell,
    f64::consts::{PI, TAU},
    ops::Range,
    sync::Arc,
    vec,
};

use nom::{
    branch::*,
    bytes::complete::*,
    character::complete::{char, *},
    combinator::{cut, map, opt, peek, recognize, value, verify},
    error,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use nom_locate::position;

use crate::ast::{SyntaxNode, *};

/// Error containing a text span and an error message to display.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(pub Range<usize>, pub String);

/// Carried around in the `LocatedSpan::extra` field in
/// between `nom` parsers.
#[derive(Clone, Debug)]
pub struct ParseState(pub Arc<RefCell<Vec<ParseError>>>);

impl ParseState {
    /// Pushes an error onto the errors stack from within a `nom`
    /// parser combinator while still allowing parsing to continue.
    #[allow(unused)]
    pub fn report_error(&self, error: ParseError) {
        self.0.borrow_mut().push(error);
    }
}

pub type Span<'a> = nom_locate::LocatedSpan<&'a str, ParseState>;

pub type ParseResult<'a, T> = nom::IResult<Span<'a>, T>;

pub fn span_range(span: &Span) -> Range<usize> {
    Range {
        start: span.location_offset(),
        end: span.location_offset() + span.len(),
    }
}

/// Evaluate `parser` and wrap the result in a `Some(_)`. Otherwise,
/// emit the  provided `error_msg` and return a `None` while allowing
/// parsing to continue.
fn expecting<'a, F, E, T>(
    mut parser: F,
    error_msg: E,
) -> impl FnMut(Span<'a>) -> ParseResult<Option<T>>
where
    F: FnMut(Span<'a>) -> ParseResult<T>,
    E: ToString,
{
    move |input: Span| {
        match parser.parse(input) {
            Ok((remaining, out)) => Ok((remaining, Some(out))),
            Err(nom::Err::Error(nom::error::Error { input, .. }))
            | Err(nom::Err::Failure(nom::error::Error { input, .. })) => {
                let err = ParseError(span_range(&input), error_msg.to_string());
                input.extra.report_error(err); // Push error onto stack.
                Ok((input, None)) // Parsing failed, but keep going.
            }
            Err(err) => Err(err),
        }
    }
}

#[allow(unused)]
fn p_op(input: &str) -> IResult<&str, Op> {
    alt((
        value(Op::Add, tag("+")),
        value(Op::Sub, tag("-")),
        value(Op::Mul, tag("*")),
        value(Op::Div, tag("*")),
    ))
    .parse(input)
}

fn p_boolean(input: Span) -> ParseResult<Primitive> {
    alt((
        value(Primitive::Bool(true), tag("true")),
        value(Primitive::Bool(false), tag("false")),
    ))
    .parse(input)
}

pub fn syntax_node<'a, T, E>(
    mut parser: impl Parser<Span<'a>, T, E>,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, SyntaxNode<T>, E>
where
    E: nom::error::ParseError<Span<'a>>,
{
    move |s: Span<'a>| {
        let (s, start) = position(s)?;
        let (s, result) = parser.parse(s)?;
        let (s, end) = position(s)?;
        let range = start.location_offset()..end.location_offset();
        Ok((s, SyntaxNode::new(Some(range), Some(result))))
    }
}

fn math_constants(input: Span) -> ParseResult<Primitive> {
    alt((
        value(Primitive::Float(PI), tag("pi")),
        value(Primitive::Float(TAU), tag("tau")),
    ))
    .parse(input)
}

fn p_integer(input: Span) -> ParseResult<i64> {
    map(recognize(many1(alt((digit1::<Span, _>, tag("_"))))), |s| {
        s.chars()
            .filter(|&c| c != '_')
            .collect::<String>()
            .parse::<i64>()
            .unwrap()
    })
    .parse(input)
}

// not amazingly written, but, well, works for now ;)
fn p_numeric_primitive(input: Span) -> ParseResult<Primitive> {
    map(
        tuple((
            opt(terminated(
                alt((char::<Span, error::Error<Span>>('+'), char('-'))),
                space0,
            )),
            alt((
                map(
                    tuple((p_integer, opt(recognize(pair(char('.'), opt(digit1)))))),
                    |(int, rest)| match rest {
                        None => Primitive::Int(int as i64),
                        Some(rest) => Primitive::Float(int as f64 + rest.parse::<f64>().unwrap()),
                    },
                ),
                map(recognize(tuple((char('.'), digit1))), |s: Span| {
                    Primitive::Float(s.parse::<f64>().unwrap())
                }),
            )),
            opt(preceded(
                multispace0,
                map(
                    alt((tag("min"), tag("ms"), tag("s"), tag("khz"), tag("hz"))),
                    |span: Span| Unit::from(span.to_string().as_ref()),
                ),
            )),
        )),
        |(sign, mut num, unit)| {
            if let Some('-') = sign {
                num = num.negate();
            }

            if let Some(unit) = unit {
                num = num.with_unit(unit);
            }

            num
        },
    )
    .parse(input)
}

fn str(input: Span) -> ParseResult<String> {
    escaped_transform(alphanumeric1, '\\', one_of("\"\n\\")).parse(input)
}

fn p_string(input: Span) -> ParseResult<Primitive> {
    map(
        preceded(char('\"'), cut(terminated(str, char('\"')))),
        Primitive::Str,
    )
    .parse(input)
}

fn p_primitive(input: Span) -> ParseResult<SyntaxNode<Primitive>> {
    syntax_node(alt((
        p_boolean,
        p_numeric_primitive,
        math_constants,
        p_string,
    )))
    .parse(input)
}

fn p_parenthesized_expr(i: Span) -> ParseResult<SyntaxNode<Expr>> {
    syntax_node(map(
        delimited(
            tag("("),
            expecting(p_expression, "expected expression after `(`"),
            expecting(tag(")"), "missing `)`"),
        ),
        |inner| Expr::Paren(inner.unwrap_or(SyntaxNode::MISSING)),
    ))
    .parse(i)
}

fn p_block_inner(mut input: Span) -> ParseResult<Block> {
    #[derive(Debug)]
    enum Item {
        Stmt(Stmt),
        Decl(SyntaxNode<Decl>),
        Expr(SyntaxNode<Expr>),
        Ws,
        Semi,
    }

    let mut item = alt((
        map(p_statement_bare, |stmt| Item::Stmt(stmt)),
        map(p_declaration, |decl| Item::Decl(decl)),
        map(p_expression, |expr| Item::Expr(expr)),
        map(tag(";"), |_| Item::Semi),
        map(multispace1, |_| Item::Ws),
    ));

    let mut block = Block {
        stmts: vec![],
        expr: None,
    };

    let mut missing_stmt_semi = false;

    loop {
        match item.parse(input.clone()) {
            Ok((rem, item)) => {
                if matches!(item, Item::Stmt(_) | Item::Expr(_) | Item::Decl(_))
                    && let Some(expr) = block.expr.take() {
                    let err = ParseError(span_range(&rem), "missing `;`".into());
                    rem.extra.report_error(err);
                    missing_stmt_semi = false;
                    block.stmts.push(Stmt::Expr(expr));
                }

                match item {
                    Item::Stmt(stmt) => {
                        block.stmts.push(stmt);
                        missing_stmt_semi = true;
                    }
                    Item::Expr(expr) => {
                        block.expr = Some(expr);
                    }
                    Item::Decl(decl) => {
                        block.stmts.push(Stmt::Decl(decl));
                    }
                    Item::Semi => {
                        if let Some(expr) = block.expr.take() {
                            block.stmts.push(Stmt::Expr(expr))
                        }
                        missing_stmt_semi = false;
                    }
                    _ => {}
                }

                input = rem;
            }
            Err(nom::Err::Error(_)) => {
                if missing_stmt_semi {
                    let err = ParseError(span_range(&input), "missing `;`".into());
                    input.extra.report_error(err);
                }
                return Ok((input, block));
            }
            // TODO - is this necessary?
            Err(e) => {
                return Err(e);
            }
        }
    }
}

fn p_block(input: Span) -> ParseResult<SyntaxNode<Block>> {
    syntax_node(delimited(
        tag("{"),
        p_block_inner,
        expecting(tag("}"), "missing `}`"),
    ))
    .parse(input)
}

fn p_access_or_call(input: Span) -> ParseResult<SyntaxNode<Expr>> {
    syntax_node(map(
        pair(
            p_identifier,
            opt(tuple((
                multispace0,
                tag("("),
                multispace0,
                separated_list0(tuple((multispace0, tag(","), multispace0)), p_expression),
                multispace0,
                opt(tag(",")),
                multispace0,
                expecting(tag(")"), "missing `)` after call"),
            ))),
        ),
        |(id, rest)| match rest {
            None => Expr::Var(id),
            Some((_, _, _, args, _, _, _, _)) => Expr::Call(CallExpr { id, args }),
        },
    ))
    .parse(input)
}

fn p_factor(i: Span) -> ParseResult<SyntaxNode<Expr>> {
    delimited(
        multispace0,
        alt((
            p_access_or_call,
            syntax_node(map(p_primitive, Expr::Prim)),
            p_parenthesized_expr,
            syntax_node(map(p_block, |block| Expr::Block(block))),
            syntax_node(map(p_anonymous_function, |fun| Expr::AnonymousFn(fun))),
        )),
        multispace0,
    )
    .parse(i)
}

fn fold_exprs(
    initial: SyntaxNode<Expr>,
    remainder: Vec<(Op, SyntaxNode<Expr>)>,
) -> SyntaxNode<Expr> {
    remainder.into_iter().fold(initial, |acc, (op, expr)| {
        SyntaxNode::new(
            cover_ranges(acc.range(), expr.range()),
            Some(Expr::BinOp(acc, op, expr)),
        )
    })
}

fn p_term(i: Span) -> ParseResult<SyntaxNode<Expr>> {
    let (i, initial) = p_factor(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, mul) = preceded(tag("*"), p_factor).parse(i)?;
            Ok((i, (Op::Mul, mul)))
        },
        |i| {
            let (i, div) = preceded(tag("/"), p_factor).parse(i)?;
            Ok((i, (Op::Div, div)))
        },
    )))
    .parse(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

fn p_expression(i: Span) -> ParseResult<SyntaxNode<Expr>> {
    let (i, initial) = p_term(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, add) = preceded(tag("+"), p_term).parse(i)?;
            Ok((i, (Op::Add, add)))
        },
        |i| {
            let (i, sub) = preceded(tag("-"), p_term).parse(i)?;
            Ok((i, (Op::Sub, sub)))
        },
    )))
    .parse(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

const KEYWORDS: &'static [&'static str] = &["let", "fn", "return", "play", "pause"];

fn is_keyword(str: &str) -> bool {
    KEYWORDS.contains(&str)
}

fn p_identifier(input: Span) -> ParseResult<SyntaxNode<Identifier>> {
    map(
        verify(
            recognize(tuple((
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            ))),
            |span: &Span| !is_keyword(&span.to_string()),
        ),
        |span: Span| SyntaxNode::from((span.clone(), Identifier(span.to_string()))),
    )
    .parse(input)
}

fn p_param(input: Span) -> ParseResult<SyntaxNode<Param>> {
    syntax_node(map(
        pair(opt(terminated(p_identifier, multispace1)), p_identifier),
        |(ty, name)| Param { ty, name },
    ))
    .parse(input)
}

fn p_anonymous_function(input: Span) -> ParseResult<SyntaxNode<AnonymousFn>> {
    syntax_node(map(
        preceded(
            pair(tag("|"), space0),
            cut(tuple((
                separated_list0(tuple((multispace0, tag(","), multispace0)), p_param),
                multispace0,
                opt(tag("|")),
                multispace0,
                expecting(p_expression, "expected anonymous function body"),
            ))),
        ),
        |(params, _, _, _, body)| AnonymousFn {
            params: ParamList(params),
            body: body.unwrap_or(SyntaxNode::MISSING),
        },
    ))
    .parse(input)
}

fn p_function_declaration(input: Span) -> ParseResult<SyntaxNode<FnDecl>> {
    syntax_node(map(
        preceded(
            pair(tag("fn"), space1),
            cut(tuple((
                expecting(p_identifier, "expected function name"),
                multispace0,
                expecting(tag("("), "expected function parameters opening `(`"),
                multispace0,
                separated_list0(tuple((multispace0, tag(","), multispace0)), p_param),
                multispace0,
                opt(tag(",")),
                multispace0,
                expecting(tag(")"), "expected function parameters closing `)`"),
                multispace0,
                expecting(p_block, "expected function body"),
            ))),
        ),
        |(name, _, _, _, params, _, _, _, _, _, body)| FnDecl {
            name: name.unwrap_or(SyntaxNode::MISSING),
            params: ParamList(params),
            body: body.unwrap_or(SyntaxNode::MISSING),
        },
    ))
    .parse(input)
}

fn p_declaration(input: Span) -> ParseResult<SyntaxNode<Decl>> {
    syntax_node(alt((
        map(p_function_declaration, |fndecl| Decl::FnDecl(fndecl)),
        // others to come..
    )))
    .parse(input)
}

/// Parses an expression, but WITHOUT the delimiting semicolon, and NOT INCLUDING an expression statement or declaration statement
fn p_statement_bare(input: Span) -> ParseResult<Stmt> {
    alt((
        map(
            preceded(
                pair(tag("return"), alt((peek(tag(";")), space1))),
                cut(opt(p_expression)),
            ),
            |expr| Stmt::Return(expr.map(|node| node)),
        ),
        map(
            preceded(
                pair(tag("play"), space1),
                cut(expecting(p_expression, "missing play expression")),
            ),
            |expr| Stmt::Play(expr.unwrap_or(SyntaxNode::MISSING)),
        ),
        map(
            preceded(
                pair(tag("let"), space1),
                cut(tuple((
                    expecting(p_identifier, "missing let identifier"),
                    multispace0,
                    expecting(tag("="), "missing `=`"),
                    multispace0,
                    expecting(p_expression, "missing let expression"),
                ))),
            ),
            |(id, _, _, _, expr)| {
                Stmt::Let((
                    id.unwrap_or(SyntaxNode::MISSING),
                    expr.unwrap_or(SyntaxNode::MISSING),
                ))
            },
        ),
    ))
    .parse(input)
}

fn p_statement_complete(input: Span) -> ParseResult<Stmt> {
    alt((
        terminated(p_statement_bare, expecting(tag(";"), "missing `;`")),
        map(p_declaration, |decl| Stmt::Decl(decl)),
        map(
            terminated(p_expression, expecting(tag(";"), "missing `;`")),
            |expr| Stmt::Expr(expr),
        ),
    ))
    .parse(input)
}

fn p_document(mut input: Span) -> ParseResult<Document> {
    let mut stmts = vec![];

    loop {
        match p_statement_complete.parse(input.clone()) {
            Ok((rem, stmt)) => {
                stmts.push(stmt);
                input = rem;
            }
            Err(nom::Err::Error(_)) => {
                if input.is_empty() {
                    return Ok((input, Document { stmts }));
                }

                let res = take(1usize).parse(input)?;
                input = res.0;
            }
            // TODO - is this necessary?
            Err(e) => {
                return Err(e);
            }
        }
    }
}

pub fn parse_document<'a>(source: impl Into<&'a str>) -> (Document, Vec<ParseError>) {
    let errors = Arc::new(RefCell::new(vec![]));
    let span = Span::new_extra(source.into(), ParseState(errors.clone()));

    let (rem, doc) = p_document(span).expect("could not parse document");

    debug_assert!(rem.is_empty());

    let errors = rem.extra.0.take();

    (doc, errors)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::{self, assert_matches};

    use super::*;

    fn parse<'a, R, E>(
        mut parser: impl Parser<Span<'a>, R, E>,
        str: &'a str,
    ) -> Result<(&'a str, R, Vec<ParseError>), nom::Err<E>>
    where
        E: std::fmt::Debug,
    {
        // Store our error stack external to our `nom` parser here. It
        // is wrapped in a `RefCell` so parser functions down the line
        // can remotely push errors onto it as they run.
        let errors = Arc::new(RefCell::new(vec![]));
        let span = Span::new_extra(str, ParseState(errors.clone()));

        parser
            .parse(span)
            .map(|(span, result)| (*span.fragment(), result, errors.take()))
    }

    fn test_parse<'a, R, E>(
        parser: impl Parser<Span<'a>, R, E>,
        input: &'a str,
        rem: &'a str,
        res: R,
        errors: Vec<&str>,
    ) where
        R: std::fmt::Debug + PartialEq,
        E: std::fmt::Debug + PartialEq,
    {
        let parse_result = parse(parser, input);
        let (remaining, result, parse_errors) = parse_result.expect("test parse failed");
        assert_eq!(remaining, rem);
        assert_eq!(result, res);
        assert_eq!(
            errors,
            parse_errors
                .into_iter()
                .map(|err| err.1)
                .collect::<Vec<_>>()
        );
    }

    fn test_parse_doc<'a>(input: &'a str, should_be: Vec<&str>, errors: Vec<&str>) {
        let r = parse_document(input);
        assert_eq!(
            r.0.stmts.into_iter().map(debug).collect::<Vec<_>>(),
            should_be
        );
        assert_eq!(r.1.into_iter().map(|err| err.1).collect::<Vec<_>>(), errors);
    }

    fn debug<T: std::fmt::Debug>(x: T) -> String {
        format!("{:?}", x)
    }

    // #[test]
    // fn test_primitives() {
    //     assert_eq!(
    //         parse(p_primitive, "true!"),
    //         Ok(("!", Primitive::Bool(true), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "3.14!"),
    //         Ok(("!", Primitive::Float(3.14), vec![]))
    //     );

    //     assert!(match parse(p_primitive, "440hz!") {
    //         Ok((
    //             "!",
    //             Primitive::Quantity((
    //                 freq,
    //                 SyntaxNode {
    //                     node: Some(Unit::Hz),
    //                     ..
    //                 },
    //             )),
    //             errs,
    //         )) => {
    //             assert_eq!(errs, vec![]);
    //             assert_eq!(freq, 440.0);
    //             true
    //         }
    //         _ => false,
    //     });

    //     assert!(match parse(p_primitive, "- 41khz!") {
    //         Ok((
    //             "!",
    //             Primitive::Quantity((
    //                 freq,
    //                 SyntaxNode {
    //                     node: Some(Unit::Khz),
    //                     ..
    //                 },
    //             )),
    //             errs,
    //         )) => {
    //             assert_eq!(errs, vec![]);
    //             assert_eq!(freq, -41.0);
    //             true
    //         }
    //         _ => false,
    //     });

    //     assert_eq!(
    //         parse(p_primitive, "40!"),
    //         Ok(("!", Primitive::Int(40), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "-0!"),
    //         Ok(("!", Primitive::Int(0), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "-0.0!"),
    //         Ok(("!", Primitive::Float(0.0), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "-.0!"),
    //         Ok(("!", Primitive::Float(-0.0), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "-.023!"),
    //         Ok(("!", Primitive::Float(-0.023), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "40_000!"),
    //         Ok(("!", Primitive::Int(40_000), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, r#""hello"!"#),
    //         Ok(("!", Primitive::Str("hello".into()), vec![]))
    //     );
    //     assert_eq!(
    //         parse(p_primitive, "\"he\\\"llo\"!"),
    //         Ok(("!", Primitive::Str("he\"llo".into()), vec![]))
    //     );
    // }

    #[test]
    fn test_expr_errors() {
        test_parse(
            map(p_expression, debug),
            "(123)!",
            "!",
            "(123)".into(),
            vec![],
        );

        test_parse(
            map(p_expression, debug),
            "(123!",
            "!",
            "(123)".into(),
            vec!["missing `)`"],
        );

        test_parse(
            map(p_expression, debug),
            "(123 + 456!",
            "!",
            "((123 + 456))".into(),
            vec!["missing `)`"],
        );

        test_parse(
            map(p_expression, debug),
            "123 + ()!",
            "!",
            "(123 + (<MISSING>))".into(),
            vec!["expected expression after `(`"],
        );
    }

    #[test]
    fn test_anonymous_fn() {
        test_parse(map(p_expression, debug), "|| 5", "", "|| 5".into(), vec![]);
        test_parse(
            map(p_expression, debug),
            "||",
            "",
            "|| <MISSING>".into(),
            vec!["expected anonymous function body"],
        );
    }

    #[test]
    fn test_expr_factor() {
        test_parse(map(p_factor, debug), "  3  ", "", "3".into(), vec![]);
    }

    #[test]
    fn test_term() {
        test_parse(
            map(p_term, debug),
            " 3 *  5   ",
            "",
            "(3 * 5)".into(),
            vec![],
        );
        test_parse(
            map(p_term, debug),
            " 3 *  5hz   ",
            "",
            "(3 * 5hz)".into(),
            vec![],
        );
    }

    #[test]
    fn test_expr() {
        test_parse(
            map(p_expression, debug),
            " 1 + 2 *  3 ",
            "",
            "(1 + (2 * 3))".into(),
            vec![],
        );
        test_parse(
            map(p_expression, debug),
            " 1 + 2 hz *  3 / 4 - 5 ",
            "",
            "((1 + ((2hz * 3) / 4)) - 5)".into(),
            vec![],
        );
        test_parse(
            map(p_expression, debug),
            " 72 / 2 / 3 ",
            "",
            "((72 / 2) / 3)".into(),
            vec![],
        );
    }

    #[test]
    fn test_parens() {
        test_parse(
            map(p_expression, debug),
            " ( 1.2s + (2) ) *  3 ",
            "",
            "(((1.2s + (2))) * 3)".into(),
            vec![],
        );
    }

    #[test]
    fn test_block_expr() {
        test_parse(
            map(p_expression, debug),
            " ( 1.2s + { let x = 2; 5; x + 1 } ) *  3 ",
            "",
            "(((1.2s + { let x = 2; 5; (x + 1) })) * 3)".into(),
            vec![],
        );

        test_parse(
            map(p_expression, debug),
            " ( 1.2s + { let x = 2; 5; x + 1; } ) *  3 ",
            "",
            "(((1.2s + { let x = 2; 5; (x + 1); })) * 3)".into(),
            vec![],
        );

        test_parse(
            map(p_expression, debug),
            " ( 1.2s + { let x = 2; 5; x + 1;  ) *  3 ",
            "",
            "(((1.2s + { let x = 2; 5; (x + 1); })) * 3)".into(),
            vec!["missing `}`"],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let x = a(2, 3;more",
            ";more",
            "let x = a(2, 3);".into(),
            vec!["missing `)` after call"],
        );

        assert_matches!(parse(p_statement_bare, "lets"), Err(_));
    }

    #[test]
    fn test_fn_expr() {
        test_parse(map(p_param, debug), "osc s, ", ", ", "osc s".into(), vec![]);

        test_parse(
            map(p_expression, debug),
            "|osc s| s + 5hz?!",
            "?!",
            "|osc s| (s + 5hz)".into(),
            vec![],
        );

        test_parse(
            map(p_expression, debug),
            "|osc s| { s + 5hz }?!",
            "?!",
            "|osc s| { (s + 5hz) }".into(),
            vec![],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let x = |osc s| { s + 5hz };?!",
            ";?!",
            "let x = |osc s| { (s + 5hz) };".into(),
            vec![],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let = |osc s| { s + 5hz };?!",
            ";?!",
            "let <MISSING> = |osc s| { (s + 5hz) };".into(),
            vec!["missing let identifier"],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let xyz =  ?!",
            "?!",
            "let xyz = <MISSING>;".into(),
            vec!["missing let expression"],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let xyz; let a = 4",
            "; let a = 4",
            "let xyz = <MISSING>;".into(),
            vec!["missing `=`", "missing let expression"],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let x = { a b }",
            "",
            "let x = { a; b };".into(),
            vec!["missing `;`"],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let xyz 234; let a = 4",
            "; let a = 4",
            "let xyz = 234;".into(),
            vec!["missing `=`"],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let = let a=b; let xyz=23;",
            "let a=b; let xyz=23;",
            "let <MISSING> = <MISSING>;".into(),
            vec!["missing let identifier", "missing let expression"],
        );
    }

    #[test]
    fn test_expr_syntax_1() {
        let p = parse(p_expression, "4 + 12").unwrap().1;
        assert_eq!(p.range(), Some(0..6));
        assert!(match p {
            SyntaxNode {
                node: Some(box Expr::BinOp(a, Op::Add, b)),
                ..
            } => {
                assert_eq!(a.range(), Some(0..1));
                assert_eq!(b.range(), Some(4..6));
                true
            }
            _ => false,
        });
    }

    #[test]
    fn test_expr_syntax_2() {
        let p = parse(p_expression, "4 + 12 * 13 + 1").unwrap().1;
        assert_eq!(p.range(), Some(0..15));
        assert_eq!(format!("{:?}", p), "((4 + (12 * 13)) + 1)");
        assert!(match p {
            SyntaxNode {
                node: Some(box Expr::BinOp(a, Op::Add, b)),
                ..
            } => {
                assert_eq!(a.range(), Some(0..11));
                assert!(match a {
                    SyntaxNode {
                        node: Some(box Expr::BinOp(a, Op::Add, b)),
                        ..
                    } => {
                        assert_eq!(a.range(), Some(0..1));
                        assert_eq!(b.range(), Some(4..11));
                        assert!(match b {
                            SyntaxNode {
                                node: Some(box Expr::BinOp(a, Op::Mul, b)),
                                ..
                            } => {
                                assert_eq!(a.range(), Some(4..6));
                                assert_eq!(b.range(), Some(9..11));
                                true
                            }
                            _ => false,
                        });

                        true
                    }
                    _ => false,
                });

                assert_eq!(b.range(), Some(14..15));
                true
            }
            _ => false,
        });
    }

    #[test]
    fn test_stmts() {
        test_parse(
            map(p_statement_bare, debug),
            "return 26; }",
            "; }",
            "return 26;".into(),
            vec![],
        );

        assert_matches!(parse(p_statement_bare, "return26;"), Err(_));

        test_parse(
            map(p_statement_bare, debug),
            "return; }",
            "; }",
            "return;".into(),
            vec![],
        );

        test_parse(
            map(p_statement_bare, debug),
            "return ; }",
            "; }",
            "return;".into(),
            vec![],
        );

        test_parse(
            map(p_statement_bare, debug),
            "let x= (26 * 1hz); }",
            "; }",
            "let x = ((26 * 1hz));".into(),
            vec![],
        );
        test_parse(
            map(p_declaration, debug),
            "fn add( int x, wave bla) { 5 }?",
            "?",
            "fn add(int x, wave bla) { 5 }".into(),
            vec![],
        );
        test_parse(
            map(p_declaration, debug),
            "fn add( int x, wave bla, ) { 5 }?",
            "?",
            "fn add(int x, wave bla) { 5 }".into(),
            vec![],
        );

        test_parse(
            map(p_statement_bare, debug),
            "play ?",
            "?",
            "play <MISSING>;".into(),
            vec!["missing play expression"],
        );

        test_parse(
            map(p_declaration, debug),
            "fn () { 5",
            "",
            "fn <MISSING>() { 5 }".into(),
            vec!["expected function name", "missing `}`"],
        );

        test_parse(
            map(p_declaration, debug),
            "fn ( { 5 let h = 6",
            "",
            "fn <MISSING>() { 5; let h = 6; }".into(),
            vec![
                "expected function name",
                "expected function parameters closing `)`",
                "missing `;`",
                "missing `;`",
                "missing `}`",
            ],
        );

        test_parse(
            map(p_declaration, debug),
            "fn { 5; let h = 6",
            "",
            "fn <MISSING>() { 5; let h = 6; }".into(),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "expected function parameters closing `)`",
                "missing `;`",
                "missing `}`",
            ],
        );

        test_parse(
            map(p_declaration, debug),
            "fn ",
            "",
            "fn <MISSING>() <MISSING>".into(),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "expected function parameters closing `)`",
                "expected function body",
            ],
        );

        test_parse(
            map(p_declaration, debug),
            "fn ;",
            ";",
            "fn <MISSING>() <MISSING>".into(),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "expected function parameters closing `)`",
                "expected function body",
            ],
        );

        test_parse(
            map(p_declaration, debug),
            "fn );",
            ";",
            "fn <MISSING>() <MISSING>".into(),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "expected function body",
            ],
        );

        test_parse(
            map(p_declaration, debug),
            "fn )};",
            "};",
            "fn <MISSING>() <MISSING>".into(),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "expected function body",
            ],
        );

        test_parse(
            map(p_declaration, debug),
            "fn ){;",
            "",
            "fn <MISSING>() { }".into(),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "missing `}`",
            ],
        );

        assert_matches!(parse(p_declaration, "fn){;"), Err(_));
    }

    #[test]
    fn test_document() {
        test_parse(
            map(p_document, debug),
            "fn { 5; let h = 6 }}; let h = 6;; 123 *68 play 6;",
            "",
            vec![
                "fn <MISSING>() { 5; let h = 6; }",
                "let h = 6;",
                "(123 * 68);",
                "play 6;",
            ]
            .join("\n\n"),
            vec![
                "expected function name",
                "expected function parameters opening `(`",
                "expected function parameters closing `)`",
                "missing `;`",
                "missing `;`",
            ],
        );
    }

    #[test]
    fn test_all_together() {
        test_parse_doc(
            " fn beat_reverb(int a, int b) {
                          let bla = 5hz; a
                             + b
                     }

          let audio   = midi_in * bla  * bla(  4, 6)
          ; play audio;  ",
            vec![
                "fn beat_reverb(int a, int b) { let bla = 5hz; (a + b) }",
                "let audio = ((midi_in * bla) * bla(4, 6));",
                "play audio;",
            ],
            vec![],
        );
    }
}

/*

pub struct Duration(f32);

pub struct LinearEasing {}

pub struct BezierCurve {
    c1: Pos2,
    c2: Pos2,
}

pub trait HasDuration {
    fn duration(&self) -> f32;
}

impl HasDuration for Duration {
    fn duration(&self) -> f32 {
        self.0
    }
}

pub trait Ease {
    /// for x in [0,1]
    fn interpolate(&self, x: f32) -> f32;
}

impl Ease for LinearEasing {
    fn interpolate(&self, x: f32) -> f32 {
        x
    }
}

impl Ease for BezierCurve {
    fn interpolate(&self, x: f32) -> f32 {
        pos2(0.0, 0.0)
            .lerp(self.c1, x)
            .lerp(self.c2.lerp(pos2(1.0, 1.0), x), x)
            .y
    }
}

impl Ease for Duration {
    fn interpolate(&self, x: f32) -> f32 {
        x / self.0
    }
}

pub struct EasingDuration {
    duration: Duration,
    easing: Box<dyn Ease>,
}

impl HasDuration for EasingDuration {
    fn duration(&self) -> f32 {
        self.duration.duration()
    }
}

impl std::ops::Mul<Box<dyn Ease>> for Duration {
    type Output = EasingDuration;

    fn mul(self, easing: Box<dyn Ease>) -> Self::Output {
        EasingDuration {
            duration: self,
            easing,
        }
    }
}

impl std::ops::Mul<Duration> for Box<dyn Ease> {
    type Output = EasingDuration;

    fn mul(self, duration: Duration) -> Self::Output {
        EasingDuration {
            duration,
            easing: self,
        }
    }
}

pub trait HasDurationAndCanEase: HasDuration + Ease {}

pub struct Envelope {
    attack: Box<dyn HasDurationAndCanEase>,
    decay: Box<dyn HasDurationAndCanEase>,
    sustain: f32,
    release: Box<dyn HasDurationAndCanEase>,
}

pub trait Source {}

===

"RUST DSL ATTEMPT"

type Pos2 { x: float, y: float };
type Linear;
type Bezier { c1: Pos2, c2: Pos2 };
type Duration { secs: float };

fn convert(float secs) -> Duration = Duration { secs };

fn duration(Bezier _) -> Duration = 1s;

fn duration(Duration { secs }) -> float = secs;

fn ease(Duration { secs }, float x) -> float = {
    x / secs
};

fn mul(Duration duration, Bezier bezier) -> {}

// this is an existential type
class DurationAndEase A {
    duration(A) -> float;
    ease(A, float) -> float;
};

type Envelope(fn envelope) {
    attack: DurationAndEase;
    decay: DurationAndEase;
    sustain: float;
    release: DurationAndEase;
};

let env = Envelope {
    attack: .3s;
};

===

"JULIA/JS DYNAMIC DISPATCH RUNTIME ATTEMPT"

let f = 440;

play sin[f];

===

object type v. literal
(dynamically constructible v. constant expression)

arrays:
    [1, 2, 3]
    v.
    csv.lines().map(|line| { ... })

timelines:
    t{
        ??
    }
    v.
    timeline()

timeline of what?
    - single value over time -> "modulation"
    - various changes -> isn't that just a set of modulations?
    - anything -> dynamic code
        - event-based?
        - time-based?

let f = 440 * ease(600, .2s);

let mod = sin[f + { sin(1) * 50 }];

play mod;


===

identity = variable name, otherwise nesting structure beneath nearest identity

let f = 440; // id = f

play sin[f]; // id = program.play[0]


let f = 440 * ease(600, .2s);


play [(1,1), (3,.5), (5, .2), (7, .1)] * map(|(h,v)| {}) * sum;

play [(1,1), (3,.5), (5, .2), (7, .1)].map(|(h,v)| {
    sine[440 * ease[600, .2s] * h] * v
}).sum(); // id = program.play[1]

===

`play <expression>` statements are only evaluated in the main file, not in imports,
 nor in sub- (modules or blocks, TODO decide)
 -- that makes them useful for testing parts of sounds on themselves

===



===

semantics:
- parse tree
    - if invalid, do nothing
- enrich nodes with identity keys
    - this can be used for "latching" easings etc.
- check validity of typing etc.
- evaluate
    - => object graph
- apply object graph
    - => ?? only once? or every frame?
        - "every frame" is like react jsx template / dom updating
            - would also update on `map` change
            - needs keys in order to diff
                - MAYBE .. we can just always make sure that every array is keyed from the start
                - and `map` just transfers the keys
                - (and if you want to override it, pass an optional named `:key` arg to `map`)
        - "only once"

===

let s = { let y = sin[440]; y };

play s;

===

let i = false;

fn f() -> source {
    if i {
        sin[440]
    } else {
        let y = sin[1] * 50;
        sin[440 + y]
    }
}

let s = f();

play s;

===

source/wave:    audio wave(s)
trigger:        ranging from [0-1], e.g. a midi note over time
pattern:        labeled set of triggers
                - "sparse"
                - e.g. midi input (with envelope etc. applied)
                [XXX] data structure at time t: Array<{ label: L, value: [0..1] }>
                - data structure: dict<label, trigger>
modulation:     generic type of value changing over time

===

XXX let sound = midi_in * envelope(...) * map(|(freq, amount)| sin(freq) * amount);

let sound = midi_in * envelope(...) * map(|amount, freq| sin(freq) * amount);
# or, w/o automatic lifting:
let sound = midi_in * map(|amount, freq| envelope(...) * sin(freq) * amount);

let sound = midi_in * |amount, freq| { envelope(...) * sin(freq) * amount };

play sound;

===

general:
    built_in *(envelope, node<T>) where T: mul<float> & T: is-zero;

more specific:
    built_in *(envelope, pattern)
        - because it shouldn't be applied on the array of labeled-values, but on each trigger separately

general:
    built_in *([T], fn(T) -> U) -> [U]
    built_in *([node<T>], fn(T) -> U) -> [node<U>]

===

(rust)
let midi_in: Dict<freq, Trigger/ControlSource> = ...;
...

===

`sin[mod]` <- is this a sin, or a "modulated sin" or smth?
    - e.g. is there lifting? or is it built-in to the language that all things might change over time?
    - => is `sin` a number or a wave?

conflicting!
    - I want `sin[440hz]` to be a wave
    - I want `sin[mod]` to also just be a wave (or whatever `sin` is)

resolution?

    - `sin[x]` just happens to be a function that takes a "maybe modulated" x

fn sin(node<freq> x) -> wave { [built-in] };

    alias wave = audio_node<sample>;

    alias sample = float | [float];

===

TYPE SYSTEM

- try to lift / convert types so that they fit
- dynamic dispatch of functions, "most specific implementation first"
    - (maybe don't even support user-defined function, only overloads of operators? :P)
        - no, that's silly -- we need "named things", and would otherwise need to introduce extra types to make it work, kinda stupid

fn euclid(int x, int y) -> pattern {
    let x = 6;
}

===

fn envelope[duration & ease a, ...] { ... };

type bezier;

fn bezier[float c1x, float c1y, float c2x, float c2y] -> bezier {  };

fn 'unordered *(duration d, easing e) -> {};

===

syntax:

atomic literals
    false, true
    .4, 0.4, 4., 100_000.123, -15
    0.3s, 440hz, 4ms, 4min, 400khz
    pi, tau

expressions
    []
    [a,b, c]
    [a, b , c,]

operators with known bindings / precedences / associativities
    *, %, +, -

rust-like expression vs statement handling
    let y = {
        let x = 5hz;
        6 + x
    };

===

let bpm = 120;
let beat = trigger(60/bpm); // like square, except 0..1 instead of -1..1
let beat = max(square(60/bpm), 0);

pause sound @ beat;

===

A syntax node can be:

    - Editable,
      because compile-time evaluatable

      ```
      bezier(.46, .1, .77, .47)
      ```

      ```
      let x = 0.46;

      bezier(x, .1, .77, .47)
      ```

    - Visualizable, but not (fully) editable (if only for demonstration purposes),
      because we enforce that all (or most?) parameters always have defaults

        ...or do we want to fallback to editing the default value?

      ```
      let kick = |x = .46| {
          sample * bezier(x, .1, .77, .47)
      };
      ```

    - Not visualizable, because we have no (default) values for the variables involved,
      I guess this will happen anyway, because otherwise we'd have to also
      default things like patterns and samples etc, and that might be TOO HARD

*/
