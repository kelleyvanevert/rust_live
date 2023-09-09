#![feature(assert_matches)]
#![feature(let_chains)]
#![feature(box_patterns)]

pub mod ast;
mod check;
mod parse;
mod parse_v2;

pub use parse::parse_document;
