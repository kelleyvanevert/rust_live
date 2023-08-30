use ast::Document;
use check::check_document;
use parse::parse_document;

mod ast;
mod check;
mod parse;

pub fn parse_and_check(code: &str) -> Option<Document> {
    parse_document(code.into()).map(check_document)
}

fn main() {
    // println!("{:?}", parse_document("play sine(440hz);"));
}
