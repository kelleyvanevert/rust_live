#![feature(assert_matches)]
#![feature(let_chains)]
#![feature(box_patterns)]

pub mod ast;
mod check;
mod parse;

pub use parse::parse_document;
