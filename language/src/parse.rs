use core::f64;
use std::f64::consts::{PI, TAU};

use nom::{
    branch::*,
    bytes::complete::*,
    character::complete::{char, *},
    combinator::{cut, eof, map, opt, recognize, value},
    error,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};

use crate::ast::*;

#[allow(unused)]
fn op(input: &str) -> IResult<&str, Op> {
    alt((
        value(Op::Add, tag("+")),
        value(Op::Sub, tag("-")),
        value(Op::Mul, tag("*")),
        value(Op::Div, tag("*")),
    ))
    .parse(input)
}

fn boolean(input: &str) -> IResult<&str, Primitive> {
    alt((
        value(Primitive::Bool(true), tag("true")),
        value(Primitive::Bool(false), tag("false")),
    ))
    .parse(input)
}

fn math_constants(input: &str) -> IResult<&str, Primitive> {
    alt((
        value(Primitive::Float(PI), tag("pi")),
        value(Primitive::Float(TAU), tag("tau")),
    ))
    .parse(input)
}

fn integer(input: &str) -> IResult<&str, i64> {
    map(recognize(many1(alt((digit1::<&str, _>, tag("_"))))), |s| {
        s.chars()
            .filter(|&c| c != '_')
            .collect::<String>()
            .parse::<i64>()
            .unwrap()
    })
    .parse(input)
}

// not amazingly written, but, well, works for now ;)
fn numeric_primitive(input: &str) -> IResult<&str, Primitive> {
    map(
        tuple((
            opt(alt((char::<&str, error::Error<&str>>('+'), char('-')))),
            alt((
                map(
                    tuple((integer, opt(recognize(pair(char('.'), opt(digit1)))))),
                    |(int, rest)| match rest {
                        None => Primitive::Int(int as i64),
                        Some(rest) => Primitive::Float(int as f64 + rest.parse::<f64>().unwrap()),
                    },
                ),
                map(recognize(tuple((char('.'), digit1))), |s: &str| {
                    Primitive::Float(s.parse::<f64>().unwrap())
                }),
            )),
            opt(preceded(
                multispace0,
                map(
                    alt((tag("min"), tag("ms"), tag("s"), tag("khz"), tag("hz"))),
                    Unit::from,
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

fn parse_str(input: &str) -> IResult<&str, String> {
    escaped_transform(alphanumeric1, '\\', one_of("\"\n\\")).parse(input)
}

fn string(input: &str) -> IResult<&str, Primitive> {
    map(
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
        Primitive::Str,
    )
    .parse(input)
}

pub fn parse_primitive(input: &str) -> IResult<&str, Primitive> {
    alt((boolean, numeric_primitive, math_constants, string)).parse(input)
}

fn parenthesized_expr(i: &str) -> IResult<&str, Expr> {
    delimited(
        tag("("),
        map(parse_expression, |e| Expr::Paren(Box::new(e))),
        tag(")"),
    )
    .parse(i)
}

fn parse_block(input: &str) -> IResult<&str, Block> {
    delimited(
        tag("{"),
        map(
            tuple((
                multispace0,
                many0(terminated(parse_statement, multispace0)),
                opt(terminated(parse_expression, multispace0)),
            )),
            |(_, stmts, expr)| Block {
                stmts,
                expr: expr.map(Box::new),
            },
        ),
        tag("}"),
    )
    .parse(input)
}

fn access_or_call(input: &str) -> IResult<&str, Expr> {
    map(
        pair(
            identifier,
            opt(tuple((
                multispace0,
                tag("("),
                multispace0,
                separated_list0(
                    tuple((multispace0, tag(","), multispace0)),
                    parse_expression,
                ),
                multispace0,
                tag(")"),
            ))),
        ),
        |(id, rest)| match rest {
            None => Expr::Var(id),
            Some((_, _, _, args, _, _)) => Expr::Call(CallExpr { id, args }),
        },
    )
    .parse(input)
}

fn factor(i: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            access_or_call,
            map(parse_primitive, Expr::Prim),
            parenthesized_expr,
            map(parse_block, |block| Expr::Block(Box::new(block))),
            map(parse_anonymous_function, |fun| {
                Expr::AnonymousFn(Box::new(fun))
            }),
        )),
        multispace0,
    )
    .parse(i)
}

fn fold_exprs(initial: Expr, remainder: Vec<(Op, Expr)>) -> Expr {
    remainder.into_iter().fold(initial, |acc, pair| {
        let (oper, expr) = pair;
        match oper {
            Op::Add => Expr::Add(Box::new(acc), Box::new(expr)),
            Op::Sub => Expr::Sub(Box::new(acc), Box::new(expr)),
            Op::Mul => Expr::Mul(Box::new(acc), Box::new(expr)),
            Op::Div => Expr::Div(Box::new(acc), Box::new(expr)),
        }
    })
}

fn term(i: &str) -> IResult<&str, Expr> {
    let (i, initial) = factor(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, mul) = preceded(tag("*"), factor).parse(i)?;
            Ok((i, (Op::Mul, mul)))
        },
        |i| {
            let (i, div) = preceded(tag("/"), factor).parse(i)?;
            Ok((i, (Op::Div, div)))
        },
    )))
    .parse(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

pub fn parse_expression(i: &str) -> IResult<&str, Expr> {
    let (i, initial) = term(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, add) = preceded(tag("+"), term).parse(i)?;
            Ok((i, (Op::Add, add)))
        },
        |i| {
            let (i, sub) = preceded(tag("-"), term).parse(i)?;
            Ok((i, (Op::Sub, sub)))
        },
    )))
    .parse(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

fn identifier(input: &str) -> IResult<&str, Identifier> {
    map(
        recognize(tuple((
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        ))),
        |str: &str| Identifier(str.into()),
    )
    .parse(input)
}

fn parse_param(input: &str) -> IResult<&str, Param> {
    map(
        pair(opt(terminated(identifier, multispace1)), identifier),
        |(ty, name)| Param { ty, name },
    )
    .parse(input)
}

fn parse_anonymous_function(input: &str) -> IResult<&str, AnonymousFn> {
    map(
        preceded(
            pair(tag("|"), space0),
            cut(tuple((
                separated_list0(tuple((multispace0, tag(","), multispace0)), parse_param),
                multispace0,
                opt(tag("|")),
                multispace0,
                parse_expression,
            ))),
        ),
        |(params, _, _, _, body)| AnonymousFn {
            params: ParamList(params),
            body: Box::new(body),
        },
    )
    .parse(input)
}

fn parse_function_declaration(input: &str) -> IResult<&str, FnDecl> {
    map(
        preceded(
            pair(tag("fn"), space1),
            cut(tuple((
                identifier,
                multispace0,
                tag("("),
                multispace0,
                separated_list0(tuple((multispace0, tag(","), multispace0)), parse_param),
                multispace0,
                opt(tag(",")),
                multispace0,
                tag(")"),
                multispace0,
                parse_block,
            ))),
        ),
        |(name, _, _, _, params, _, _, _, _, _, body)| FnDecl {
            name,
            params: ParamList(params),
            body: Box::new(body),
        },
    )
    .parse(input)
}

fn parse_item(input: &str) -> IResult<&str, Item> {
    alt((
        map(parse_function_declaration, |fndecl| {
            Item::FnDecl(Box::new(fndecl))
        }),
        // others to come..
    ))
    .parse(input)
}

pub fn parse_statement(input: &str) -> IResult<&str, Stmt> {
    alt((
        map(tag(";"), |_| Stmt::Skip),
        map(
            preceded(
                pair(tag("return"), space1),
                cut(tuple((parse_expression, tag(";")))),
            ),
            |(expr, _)| Stmt::Return(Box::new(expr)),
        ),
        map(
            preceded(
                pair(tag("play"), space1),
                cut(tuple((parse_expression, tag(";")))),
            ),
            |(expr, _)| Stmt::Play(Box::new(expr)),
        ),
        map(
            preceded(
                pair(tag("let"), space1),
                cut(tuple((
                    identifier,
                    multispace0,
                    tag("="),
                    multispace0,
                    parse_expression,
                    tag(";"),
                ))),
            ),
            |(id, _, _, _, expr, _)| Stmt::Let((id, Box::new(expr))),
        ),
        map(parse_item, |item| Stmt::Item(Box::new(item))),
        map(terminated(parse_expression, tag(";")), |expr| {
            Stmt::Expr(Box::new(expr))
        }),
    ))
    .parse(input)
}

pub fn parse_document(input: &str) -> Option<Document> {
    terminated(
        map(
            tuple((multispace0, many0(terminated(parse_statement, multispace0)))),
            |(_, stmts)| Document { stmts },
        ),
        eof,
    )
    .parse(input)
    .ok()
    .map(|p| p.1)
}

#[test]
fn test_primitives() {
    assert_eq!(parse_primitive("true!"), Ok(("!", Primitive::Bool(true))));
    assert_eq!(parse_primitive("3.14!"), Ok(("!", Primitive::Float(3.14))));
    assert_eq!(
        parse_primitive("440hz!"),
        Ok(("!", Primitive::Quantity((440.0, "hz".into()))))
    );
    assert_eq!(
        parse_primitive("40 khz!"),
        Ok(("!", Primitive::Quantity((40.0, "khz".into()))))
    );
    assert_eq!(
        parse_primitive("-40 khz!"),
        Ok(("!", Primitive::Quantity((-40.0, "khz".into()))))
    );
    assert_eq!(parse_primitive("40!"), Ok(("!", Primitive::Int(40))));
    assert_eq!(parse_primitive("-0!"), Ok(("!", Primitive::Int(0))));
    assert_eq!(parse_primitive("-0.0!"), Ok(("!", Primitive::Float(0.0))));
    assert_eq!(parse_primitive("-.0!"), Ok(("!", Primitive::Float(-0.0))));
    assert_eq!(
        parse_primitive("-.023!"),
        Ok(("!", Primitive::Float(-0.023)))
    );
    assert_eq!(
        parse_primitive("40_000!"),
        Ok(("!", Primitive::Int(40_000)))
    );
    assert_eq!(
        parse_primitive(r#""hello"!"#),
        Ok(("!", Primitive::Str("hello".into())))
    );
    assert_eq!(
        parse_primitive("\"he\\\"llo\"!"),
        Ok(("!", Primitive::Str("he\"llo".into())))
    );
    assert_eq!("456".parse::<f64>(), Ok(456.0));
}

#[test]
fn test_factor() {
    assert_eq!(factor("  3  "), Ok(("", Expr::Prim(Primitive::Int(3)))));
    assert_eq!(
        factor("  3  ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "3".into()))
    );
}

#[test]
fn test_term() {
    assert_eq!(
        term(" 3 *  5   ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "(3 * 5)".into()))
    );

    assert_eq!(
        term(" 3 *  5hz   ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "(3 * 5hz)".into()))
    );
}

#[test]
fn test_expr() {
    assert_eq!(
        parse_expression(" 1 + 2 *  3 ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "(1 + (2 * 3))".into()))
    );
    assert_eq!(
        parse_expression(" 1 + 2 hz *  3 / 4 - 5 ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "((1 + ((2hz * 3) / 4)) - 5)".into()))
    );
    assert_eq!(
        parse_expression(" 72 / 2 / 3 ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "((72 / 2) / 3)".into()))
    );
}

#[test]
fn test_parens() {
    assert_eq!(
        parse_expression(" ( 1.2s + (2) ) *  3 ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "(((1.2s + (2))) * 3)".into()))
    );
}

#[test]
fn test_block_expr() {
    assert_eq!(
        parse_expression(" ( 1.2s + { let x = 2; 5; x + 1 } ) *  3 ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "(((1.2s + { let x = 2; 5; (x + 1) })) * 3)".into()))
    );
    assert_eq!(
        parse_expression(" ( 1.2s + { let x = 2; 5; x + 1; } ) *  3 ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", "(((1.2s + { let x = 2; 5; (x + 1); })) * 3)".into()))
    );
}

#[test]
fn test_fn_expr() {
    assert_eq!(
        parse_param("osc s, ").map(|(i, x)| (i, format!("{:?}", x))),
        Ok((", ", "osc s".into()))
    );
    assert_eq!(
        parse_expression("|osc s| s + 5hz?!").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("?!", "|osc s| (s + 5hz)".into()))
    );
    assert_eq!(
        parse_expression("|osc s| { s + 5hz }?!").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("?!", "|osc s| { (s + 5hz) }".into()))
    );
    assert_eq!(
        parse_statement("let x = |osc s| { s + 5hz };?!").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("?!", "let x = |osc s| { (s + 5hz) };".into()))
    );
}

#[test]
fn test_stmts() {
    assert_eq!(
        parse_statement("return 26; }").map(|(i, x)| (i, format!("{:?}", x))),
        Ok((" }", "return 26;".into()))
    );
    assert_eq!(
        parse_statement("let x= (26 * 1hz); }").map(|(i, x)| (i, format!("{:?}", x))),
        Ok((" }", "let x = ((26 * 1hz));".into()))
    );
    assert_eq!(
        parse_statement("fn add( int x, wave bla) { 5 }?").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("?", "fn add(int x, wave bla) { 5 }".into()))
    );
    assert_eq!(
        parse_statement("fn add( int x, wave bla, ) { 5 }?").map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("?", "fn add(int x, wave bla) { 5 }".into()))
    );
}

#[test]
fn test_all_together() {
    assert_eq!(
        parse_document(
            "fn beat_reverb(int a, int b) { let bla = 5hz; a + b }
        
        let audio = midi_in * bla * bla(4, 6) ; play audio;  "
        )
        .map(|x| format!("{:?}", x)),
        Some(
            "fn beat_reverb(int a, int b) { let bla = 5hz; (a + b) }

let audio = ((midi_in * bla) * bla(4, 6));

play audio;"
                .into()
        )
    );
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

*/
