#![feature(assert_matches)]
#![feature(let_chains)]

use ast::Document;
use check::check_document;
// use parse::parse_document;

mod ast;
mod check;
mod parse;
mod syntax;

// fn bool2(input: Span) -> ParseResult<SyntaxNode> {
//     map(tag("true"), |pos| SyntaxNode {
//         pos,
//         kind: SyntaxKind::BoolLiteral,
//     })
//     .parse(input)
// }

// fn bool3(input: Span) -> ParseResult<SyntaxNode> {
//     syntax_node(SyntaxKind::LiteralBool, tag("true")).parse(input)
// }

// pub fn parse_and_check(code: &str) -> Document {
//     check_document(parse_document(code).0)
// }

fn main() {
    // println!(
    //     "{:?}",
    //     bool2(Span::new_extra(
    //         "trueb".into(),
    //         ParseState(Arc::new(RefCell::new(vec![])))
    //     ))
    // );

    // println!(
    //     "{:?}",
    //     bool3(Span::new_extra(
    //         "trueb".into(),
    //         ParseState(Arc::new(RefCell::new(vec![])))
    //     ))
    //     .unwrap()
    //     .1
    // );
}
