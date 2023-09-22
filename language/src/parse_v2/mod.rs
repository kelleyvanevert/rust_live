use core::f64;
use std::{
    assert_matches::assert_matches,
    cell::RefCell,
    f64::consts::{PI, TAU},
    fmt::Write,
    ops::Range,
    path::Iter,
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
    IResult, Offset, Parser, Slice,
};
use nom_locate::position;

/// Error containing a text span and an error message to display.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(pub SpanRange, pub String);

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

pub fn span_range(span: &Span) -> SpanRange {
    SpanRange {
        start: Loc(span.location_offset()),
        end: Loc(span.location_offset() + span.len()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Loc(usize); // TODO: text offset + (col, row). For now: just text offset

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpanRange {
    start: Loc,
    end: Loc,
}

fn cover_ranges(a: SpanRange, b: SpanRange) -> SpanRange {
    SpanRange {
        start: Loc(a.start.0.min(b.start.0)),
        end: Loc(a.end.0.min(b.end.0)),
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

pub fn with_span<'a, T, E>(
    mut parser: impl Parser<Span<'a>, T, E>,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (Span<'a>, T), E>
where
    E: nom::error::ParseError<Span<'a>>,
{
    move |input: Span<'a>| {
        let i = input.clone();
        let (i, result) = parser.parse(i)?;
        let recognized_span = input.slice(..input.offset(&i));

        Ok((i, (recognized_span, result)))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Kind {
    Ws,
    Op,
    Bool,
    MathConstant,
    Num,
    Amount,
    Unit,
    Str,

    Ident,

    ParenLeft,
    ParenRight,

    BracketLeft,
    BracketRight,

    CurlyLeft,
    CurlyRight,

    Dot,
    Comma,

    ParenExpr,
    MemberExpr,
    IndexExpr,
    CallExpr,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SyntaxNode<'a> {
    kind: Kind,
    range: SpanRange,

    // either this (leaf)
    fragment: Option<&'a str>,

    // or this (parent)
    children: Vec<SyntaxNode<'a>>,
}

trait CollectibleNodes<'a>
where
    Self: Sized,
{
    fn collect_into(self, nodes: &mut Vec<SyntaxNode<'a>>);
}

impl<'a> CollectibleNodes<'a> for SyntaxNode<'a> {
    fn collect_into(self, nodes: &mut Vec<SyntaxNode<'a>>) {
        if !self.empty() {
            nodes.push(self.clone());
        }
    }
}

impl<'a, C> CollectibleNodes<'a> for Vec<C>
where
    C: CollectibleNodes<'a> + Sized,
{
    fn collect_into(self, nodes: &mut Vec<SyntaxNode<'a>>) {
        for item in self {
            item.collect_into(nodes);
        }
    }
}

impl<'a, C> CollectibleNodes<'a> for Option<C>
where
    C: CollectibleNodes<'a> + Sized,
{
    fn collect_into(self, nodes: &mut Vec<SyntaxNode<'a>>) {
        if let Some(node) = self {
            node.collect_into(nodes);
        }
    }
}

impl<'a, C, D> CollectibleNodes<'a> for (C, D)
where
    C: CollectibleNodes<'a> + Sized,
    D: CollectibleNodes<'a> + Sized,
{
    fn collect_into(self, nodes: &mut Vec<SyntaxNode<'a>>) {
        self.0.collect_into(nodes);
        self.1.collect_into(nodes);
    }
}

impl<'a, C, D, E> CollectibleNodes<'a> for (C, D, E)
where
    C: CollectibleNodes<'a> + Sized,
    D: CollectibleNodes<'a> + Sized,
    E: CollectibleNodes<'a> + Sized,
{
    fn collect_into(self, nodes: &mut Vec<SyntaxNode<'a>>) {
        self.0.collect_into(nodes);
        self.1.collect_into(nodes);
    }
}

impl<'a> SyntaxNode<'a> {
    pub fn leaf(kind: Kind, span: Span<'a>) -> Self {
        Self {
            kind,
            range: span_range(&span),
            fragment: Some(span.fragment()),
            children: vec![],
        }
    }

    pub fn new(kind: Kind, range: SpanRange) -> Self {
        Self {
            kind,
            range,
            fragment: None,
            children: vec![],
        }
    }

    pub fn empty(&self) -> bool {
        self.range.end.0 == self.range.start.0
    }

    fn with_collect_children<I>(mut self, collect: I) -> Self
    where
        I: CollectibleNodes<'a>,
    {
        collect.collect_into(&mut self.children);
        self
    }

    #[allow(unused)]
    pub fn fold_preorder<A>(&self, acc: A, f: &mut impl FnMut(A, &SyntaxNode<'a>) -> A) -> A {
        let acc = f(acc, &self);

        self.children
            .iter()
            .fold(acc, |acc, tree| tree.fold_preorder(acc, f))
    }

    #[allow(unused)]
    pub fn fold_postorder<A>(&self, acc: A, f: &mut impl FnMut(A, &SyntaxNode<'a>) -> A) -> A {
        let acc = self
            .children
            .iter()
            .fold(acc, |acc, tree| tree.fold_postorder(acc, f));

        f(acc, &self)
    }

    pub fn walk_postorder(&self, f: &mut impl FnMut(&SyntaxNode<'a>)) {
        for child in self.children.iter() {
            child.walk_postorder(f);
        }
        f(self);
    }

    #[allow(unused)]
    pub fn stringify(&self) -> String {
        let mut str: String = "".into();

        self.walk_postorder(&mut |node: &SyntaxNode<'a>| {
            if let Some(fragment) = node.fragment {
                write!(&mut str, "{}", fragment).unwrap();
            }
        });

        str
    }
}

impl<'a> std::fmt::Debug for SyntaxNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)?;

        if self.children.len() > 0 {
            write!(
                f,
                "[{}]",
                self.children
                    .iter()
                    .map(|child| format!("{:?}", child))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        } else if let Some(fragment) = self.fragment && !matches!(self.kind, Kind::Ws | Kind::BracketLeft | Kind::BracketRight | Kind::ParenLeft | Kind::ParenRight | Kind::CurlyLeft | Kind::CurlyRight | Kind::Comma | Kind::Dot) {
            write!(f, "[{}]", fragment)?;
        }

        Ok(())
    }
}

pub fn leaf<'a, T, E>(
    kind: Kind,
    mut parser: impl Parser<Span<'a>, T, E>,
) -> impl FnMut(Span<'a>) -> nom::IResult<Span<'a>, SyntaxNode, E>
where
    E: nom::error::ParseError<Span<'a>>,
{
    move |input: Span<'a>| {
        let i = input.clone();
        let (i, _) = parser.parse(i)?;
        let recognized_span = input.slice(..input.offset(&i));

        let node = SyntaxNode::leaf(kind, recognized_span);

        Ok((i, node))
    }
}

fn p_ws0(input: Span) -> ParseResult<SyntaxNode> {
    map(multispace0, |span| SyntaxNode::leaf(Kind::Ws, span)).parse(input)
}

fn p_ws1(input: Span) -> ParseResult<SyntaxNode> {
    map(multispace1, |span| SyntaxNode::leaf(Kind::Ws, span)).parse(input)
}

fn p_op(input: Span) -> ParseResult<SyntaxNode> {
    map(recognize(one_of("+-*/")), |span| {
        SyntaxNode::leaf(Kind::Op, span)
    })
    .parse(input)
}

#[test]
fn test_op() {
    assert_eq!(
        test_parse_debug(p_op, "+ 3"),
        Ok((" 3", "Op[+]".into(), vec![]))
    );
}

fn p_bool(input: Span) -> ParseResult<SyntaxNode> {
    map(recognize(alt((tag("true"), tag("false")))), |span| {
        SyntaxNode::leaf(Kind::Bool, span)
    })
    .parse(input)
}

#[test]
fn test_bool() {
    assert_eq!(
        test_parse_debug(p_bool, "true "),
        Ok((" ", "Bool[true]".into(), vec![]))
    );

    assert_matches!(test_parse_debug(p_bool, " true "), Err(_));
}

fn p_math_constant(input: Span) -> ParseResult<SyntaxNode> {
    map(recognize(alt((tag("pi"), tag("tau")))), |span| {
        SyntaxNode::leaf(Kind::MathConstant, span)
    })
    .parse(input)
}

fn decimal(input: Span) -> IResult<Span, Span> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))).parse(input)
}

fn p_num(input: Span) -> ParseResult<SyntaxNode> {
    map(
        recognize(tuple((
            opt(terminated(one_of("+-"), space0)),
            alt((
                recognize(tuple((char('.'), decimal))),
                recognize(tuple((decimal, char('.'), opt(decimal)))),
                decimal,
            )),
        ))),
        |span| SyntaxNode::leaf(Kind::Num, span),
    )
    .parse(input)
}

fn p_unit(input: Span) -> ParseResult<SyntaxNode> {
    map(
        alt((tag("min"), tag("ms"), tag("s"), tag("khz"), tag("hz"))),
        |span| SyntaxNode::leaf(Kind::Unit, span),
    )
    .parse(input)
}

fn p_num_or_amount(input: Span) -> ParseResult<SyntaxNode> {
    map(
        with_span(tuple((p_num, opt(tuple((p_ws0, p_unit)))))),
        |(span, (num, and_unit))| match and_unit {
            None => num,
            Some(items) => SyntaxNode::leaf(Kind::Amount, span).with_collect_children((num, items)),
        },
    )
    .parse(input)
}

#[test]
fn test_num_or_amount() {
    assert_eq!(
        test_parse_debug(p_num_or_amount, "3.14 "),
        Ok((" ", "Num[3.14]".into(), vec![]))
    );

    assert_eq!(
        test_parse_debug(p_num_or_amount, ".1hz "),
        Ok((" ", "Amount[Num[.1], Unit[hz]]".into(), vec![]))
    );

    assert_eq!(
        test_parse_debug(p_num_or_amount, "4 "),
        Ok((" ", "Num[4]".into(), vec![]))
    );
}

fn str(input: Span) -> IResult<Span, Span> {
    escaped(none_of("\\\""), '\\', one_of("\"\n")).parse(input)
}

fn p_str(input: Span) -> ParseResult<SyntaxNode> {
    map(
        recognize(preceded(
            char('\"'),
            cut(terminated(
                str,
                expecting(char('\"'), "expected closing quote for string"),
            )),
        )),
        |span| SyntaxNode::leaf(Kind::Str, span),
    )
    .parse(input)
}

fn p_primitive(input: Span) -> ParseResult<SyntaxNode> {
    alt((
        //
        p_bool,
        p_num_or_amount,
        p_math_constant,
        p_str,
    ))
    .parse(input)
}

#[test]
fn test_primitive() {
    assert_eq!(
        test_parse_debug(p_primitive, "true "),
        Ok((" ", "Bool[true]".into(), vec![]))
    );

    assert_eq!(
        test_parse_debug(p_num_or_amount, "12 "),
        Ok((" ", "Num[12]".into(), vec![]))
    );
}

const KEYWORDS: &'static [&'static str] = &["let", "fn", "return", "play", "pause"];

fn is_keyword(str: &str) -> bool {
    KEYWORDS.contains(&str)
}

fn p_identifier(input: Span) -> ParseResult<SyntaxNode> {
    map(
        verify(
            recognize(tuple((
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            ))),
            |span: &Span| !is_keyword(&span.to_string()),
        ),
        |span: Span| SyntaxNode::leaf(Kind::Ident, span),
    )
    .parse(input)
}

enum SubsequenctUse {
    Index,
    AccessMember,
    Call,
}

fn p_use_index(input: Span) -> ParseResult<(SubsequenctUse, Vec<SyntaxNode>)> {
    map(
        tuple((
            p_ws0,
            leaf(Kind::BracketLeft, tag("[")),
            cut(tuple((
                expecting(tuple((p_ws0, p_expression)), "expected index expression"),
                expecting(
                    tuple((p_ws0, leaf(Kind::BracketRight, tag("]")))),
                    "expected closing `]` for index",
                ),
            ))),
        )),
        |(ws, open, (expr, close))| {
            let mut children = vec![];

            if !ws.empty() {
                children.push(ws);
            }
            children.push(open);

            if let Some((ws, expr)) = expr {
                if !ws.empty() {
                    children.push(ws);
                }
                children.push(expr);
            }

            if let Some((ws, close)) = close {
                if !ws.empty() {
                    children.push(ws);
                }
                children.push(close);
            }

            (SubsequenctUse::Index, children)
        },
    )
    .parse(input)
}

fn p_use_access_member(input: Span) -> ParseResult<(SubsequenctUse, Vec<SyntaxNode>)> {
    map(
        tuple((
            p_ws0,
            leaf(Kind::Dot, tag(".")),
            cut(tuple((expecting(
                tuple((p_ws0, p_identifier)),
                "expected member identifier",
            ),))),
        )),
        |(ws, dot, (id,))| {
            let mut children = vec![];

            if !ws.empty() {
                children.push(ws);
            }
            children.push(dot);

            if let Some((ws, id)) = id {
                if !ws.empty() {
                    children.push(ws);
                }
                children.push(id);
            }

            (SubsequenctUse::AccessMember, children)
        },
    )
    .parse(input)
}

fn p_comma(input: Span) -> ParseResult<SyntaxNode> {
    leaf(Kind::Comma, tag(",")).parse(input)
}

fn p_args(input: Span) -> ParseResult<Vec<SyntaxNode>> {
    let (input, nodes) = many0(alt((p_ws1, p_comma, p_expression))).parse(input)?;

    enum State {
        AwaitingExpr,
        AwaitingComma,
    }

    let mut state = State::AwaitingExpr;

    for node in nodes.iter() {
        match node.kind {
            Kind::Ws => {}
            Kind::Comma => match state {
                State::AwaitingComma => {
                    state = State::AwaitingExpr;
                }
                State::AwaitingExpr => {
                    let err = ParseError(node.range, "expected expression".into());
                    input.extra.report_error(err); // because it's an arc, it's OK...
                }
            },
            _ => match state {
                State::AwaitingExpr => {
                    state = State::AwaitingComma;
                }
                State::AwaitingComma => {
                    let err = ParseError(node.range, "expected comma".into());
                    input.extra.report_error(err); // because it's an arc, it's OK...
                }
            },
        }
    }

    Ok((input, nodes))
}

#[test]
fn test_args() {
    assert_eq!(
        test_parse_debug(p_args, "4 "),
        Ok(("", "[Num[4], Ws]".into(), vec![]))
    );

    assert_eq!(
        test_parse_debug(p_args, "4 ,,,5 "),
        Ok((
            "",
            "[Num[4], Ws, Comma, Comma, Comma, Num[5], Ws]".into(),
            vec!["expected expression".into(), "expected expression".into()]
        ))
    );

    assert_eq!(
        test_parse_debug(p_args, " ,, 12 hi "),
        Ok((
            "",
            "[Ws, Comma, Comma, Ws, Num[12], Ws, Ident[hi], Ws]".into(),
            vec![
                "expected expression".into(),
                "expected expression".into(),
                "expected comma".into()
            ]
        ))
    );
}

fn p_use_call(input: Span) -> ParseResult<(SubsequenctUse, Vec<SyntaxNode>)> {
    map(
        tuple((
            p_ws0,
            leaf(Kind::ParenLeft, tag("(")),
            cut(tuple((
                p_args,
                expecting(leaf(Kind::ParenRight, tag(")")), "expected closing `)`"),
            ))),
        )),
        |(ws, open, (mut args, close))| {
            let mut children = vec![];

            if !ws.empty() {
                children.push(ws);
            }
            children.push(open);
            children.append(&mut args);
            if let Some(close) = close {
                children.push(close);
            }

            (SubsequenctUse::Call, children)
        },
    )
    .parse(input)
}

fn p_factor(input: Span) -> ParseResult<SyntaxNode> {
    alt((
        p_identifier,
        p_primitive,
        p_parenthesized_expr,
        // p_block,
        // p_anonymous_function,
    ))
    .parse(input)
}

#[test]
fn test_factor() {
    assert_eq!(
        test_parse_debug(p_factor, "4 "),
        Ok((" ", "Num[4]".into(), vec![]))
    );
}

fn fold_usages<'a>(
    initial: SyntaxNode<'a>,
    usages: Vec<(SubsequenctUse, Vec<SyntaxNode<'a>>)>,
) -> SyntaxNode<'a> {
    usages.into_iter().fold(initial, |expr, (usage, nodes)| {
        let range = cover_ranges(expr.range, nodes.last().unwrap().range);
        let mut parent = SyntaxNode::new(
            match usage {
                SubsequenctUse::Index => Kind::IndexExpr,
                SubsequenctUse::AccessMember => Kind::MemberExpr,
                SubsequenctUse::Call => Kind::CallExpr,
            },
            range,
        );

        expr.collect_into(&mut parent.children);
        parent.children.extend(nodes);

        parent
    })
}

fn p_usage(i: Span) -> ParseResult<SyntaxNode> {
    let (i, initial) = p_factor(i)?;

    let (i, usages) = many0(alt((p_use_index, p_use_access_member, p_use_call))).parse(i)?;

    Ok((i, fold_usages(initial, usages)))
}

// TODO
#[allow(unused)]
pub fn p_expression(input: Span) -> ParseResult<SyntaxNode> {
    p_usage.parse(input)
}

#[test]
fn test_expr() {
    assert_eq!(
        test_parse_debug(p_expression, "4 "),
        Ok((" ", "Num[4]".into(), vec![]))
    );

    assert_eq!(
        test_parse_debug(p_expression, "hi "),
        Ok((" ", "Ident[hi]".into(), vec![]))
    );

    assert_eq!(
        test_parse_debug(p_expression, "hi.there "),
        Ok((
            " ",
            "MemberExpr[Ident[hi], Dot, Ident[there]]".into(),
            vec![]
        ))
    );

    assert_eq!(
        test_parse_debug(p_expression, "hi .  there "),
        Ok((
            " ",
            "MemberExpr[Ident[hi], Ws, Dot, Ws, Ident[there]]".into(),
            vec![]
        ))
    );

    assert_eq!(
        test_parse_debug(p_expression, "hi .  there [4] "),
        Ok((
            " ",
            "IndexExpr[MemberExpr[Ident[hi], Ws, Dot, Ws, Ident[there]], Ws, BracketLeft, Num[4], BracketRight]"
                .into(),
            vec![]
        ))
    );

    assert_eq!(
        test_parse_debug(p_expression, "hi .  there [4] (a, b ,, c) "),
        Ok((
            " ",
            "CallExpr[IndexExpr[MemberExpr[Ident[hi], Ws, Dot, Ws, Ident[there]], Ws, BracketLeft, Num[4], BracketRight], Ws, ParenLeft, Ident[a], Comma, Ws, Ident[b], Ws, Comma, Comma, Ws, Ident[c], ParenRight]"
                .into(),
            vec!["expected expression".into()]
        ))
    );

    assert_eq!(
        test_parse_debug(p_expression, "hi .  there [4] (a, b ,, c "),
        Ok((
            "",
            "CallExpr[IndexExpr[MemberExpr[Ident[hi], Ws, Dot, Ws, Ident[there]], Ws, BracketLeft, Num[4], BracketRight], Ws, ParenLeft, Ident[a], Comma, Ws, Ident[b], Ws, Comma, Comma, Ws, Ident[c], Ws]"
                .into(),
            vec!["expected expression".into(), "expected closing `)`".into()]
        ))
    );

    let node = test_parse(p_expression, "hi .  there [4] (a, b ,, c ")
        .unwrap()
        .1;

    assert_eq!(node.stringify(), "hi .  there [4] (a, b ,, c ");
}

fn p_parenthesized_expr(i: Span) -> ParseResult<SyntaxNode> {
    map(
        with_span(tuple((
            leaf(Kind::CurlyLeft, tag("(")),
            expecting(
                tuple((p_ws0, p_expression)),
                "expected expression after `(`",
            ),
            expecting(
                tuple((p_ws0, leaf(Kind::CurlyRight, tag(")")))),
                "missing `)`",
            ),
        ))),
        |(span, items)| SyntaxNode::leaf(Kind::ParenExpr, span).with_collect_children(items),
    )
    .parse(i)
}

fn test_parse<'a, R, E>(
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

fn test_parse_debug<'a, R, E>(
    parser: impl Parser<Span<'a>, R, E>,
    str: &'a str,
) -> Result<(&'a str, String, Vec<String>), nom::Err<E>>
where
    E: std::fmt::Debug,
    R: std::fmt::Debug,
{
    test_parse(map(parser, |x| format!("{:?}", x)), str).map(|(rem, res, errs)| {
        (
            rem,
            res,
            errs.into_iter().map(|err| err.1).collect::<Vec<_>>(),
        )
    })
}
