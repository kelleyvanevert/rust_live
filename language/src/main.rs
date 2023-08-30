use ast::Document;
use check::check_document;
use parse::parse_document;

mod ast;
mod check;
mod parse;

pub fn parse_and_check(code: &str) -> Document {
    check_document(parse_document(code).0)
}

fn main() {
    // println!("{:?}", parse_document("play sine(440hz);"));
}
