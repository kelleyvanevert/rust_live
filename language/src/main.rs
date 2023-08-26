use crate::parse::parse_document;

pub mod parse;

fn main() {
    println!("{:?}", parse_document("play sine(440hz);"));
}
