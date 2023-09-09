use core::f64;
use std::{
    cell::RefCell,
    f64::consts::{PI, TAU},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Loc(usize); // TODO: text offset + (col, row). For now: just text offset

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

    CurlyLe,
    CurlyRi,

    ParenExpr,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SyntaxNode<'a> {
    kind: Kind,
    span: Span<'a>,
    children: Vec<SyntaxNode<'a>>,
}

impl<'a> SyntaxNode<'a> {
    pub fn new(kind: Kind, span: Span<'a>) -> Self {
        Self {
            kind,
            span,
            children: vec![],
        }
    }

    pub fn empty(&self) -> bool {
        self.span.len() == 0
    }

    pub fn with_child(mut self, child: SyntaxNode<'a>) -> Self {
        self.add_child(child);

        self
    }

    pub fn with_children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = SyntaxNode<'a>>,
    {
        for child in children {
            self.add_child(child);
        }

        self
    }

    pub fn with_children_opt<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Option<SyntaxNode<'a>>>,
    {
        for child in children {
            self.add_child_opt(child);
        }

        self
    }

    pub fn add_child(&mut self, child: SyntaxNode<'a>) {
        if !child.empty() {
            self.children.push(child);
        }
    }

    pub fn add_child_opt(&mut self, child: Option<SyntaxNode<'a>>) {
        if let Some(child) = child {
            self.add_child(child);
        }
    }
}

impl<'a> std::fmt::Debug for SyntaxNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.span.fragment())
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

        let node = SyntaxNode::new(kind, recognized_span);

        Ok((i, node))
    }
}

fn p_ws0(input: Span) -> ParseResult<SyntaxNode> {
    map(multispace0, |span| SyntaxNode::new(Kind::Ws, span)).parse(input)
}

fn p_op(input: Span) -> ParseResult<SyntaxNode> {
    map(recognize(one_of("+-*/")), |span| {
        SyntaxNode::new(Kind::Op, span)
    })
    .parse(input)
}

fn p_bool(input: Span) -> ParseResult<SyntaxNode> {
    map(recognize(alt((tag("true"), tag("false")))), |span| {
        SyntaxNode::new(Kind::Bool, span)
    })
    .parse(input)
}

fn p_math_constant(input: Span) -> ParseResult<SyntaxNode> {
    map(recognize(alt((tag("pi"), tag("tau")))), |span| {
        SyntaxNode::new(Kind::MathConstant, span)
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
            )),
        ))),
        |span| SyntaxNode::new(Kind::Num, span),
    )
    .parse(input)
}

fn p_unit(input: Span) -> ParseResult<SyntaxNode> {
    map(
        alt((tag("min"), tag("ms"), tag("s"), tag("khz"), tag("hz"))),
        |span| SyntaxNode::new(Kind::Unit, span),
    )
    .parse(input)
}

fn p_num_or_amount(input: Span) -> ParseResult<SyntaxNode> {
    map(
        with_span(tuple((p_num, opt(tuple((p_ws0, p_unit)))))),
        |(span, (num, and_unit))| match and_unit {
            None => num,
            Some((ws, unit)) => SyntaxNode::new(Kind::Amount, span)
                .with_child(num)
                .with_child(ws)
                .with_child(unit),
        },
    )
    .parse(input)
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
        |span| SyntaxNode::new(Kind::Str, span),
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

// TODO
fn p_expression(input: Span) -> ParseResult<SyntaxNode> {
    p_primitive.parse(input)
}

fn p_parenthesized_expr(i: Span) -> ParseResult<SyntaxNode> {
    map(
        with_span(tuple((
            leaf(Kind::CurlyLe, tag("(")),
            expecting(
                tuple((p_ws0, p_expression)),
                "expected expression after `(`",
            ),
            expecting(tuple((p_ws0, leaf(Kind::CurlyRi, tag(")")))), "missing `)`"),
        ))),
        |(span, (open, expr, close))| {
            let mut node = SyntaxNode::new(Kind::ParenExpr, span);

            node.add_child(open);

            if let Some((ws, expr)) = expr {
                node.add_child(ws);
                node.add_child(expr);
            }

            if let Some((ws, close)) = close {
                node.add_child(ws);
                node.add_child(close);
            }

            node
        },
    )
    .parse(i)
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

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

    fn parse_debug<'a, R, E>(
        parser: impl Parser<Span<'a>, R, E>,
        str: &'a str,
    ) -> Result<(&'a str, String, Vec<String>), nom::Err<E>>
    where
        E: std::fmt::Debug,
        R: Debug,
    {
        parse(map(parser, debug), str).map(|(rem, res, errs)| {
            (
                rem,
                res,
                errs.into_iter().map(|err| err.1).collect::<Vec<_>>(),
            )
        })
    }

    fn debug<T: std::fmt::Debug>(x: T) -> String {
        format!("{:?}", x)
    }

    #[test]
    fn test_op() {
        assert_eq!(parse_debug(p_op, "+ 3"), Ok((" 3", "+".into(), vec![])));
    }
}
