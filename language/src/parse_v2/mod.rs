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
