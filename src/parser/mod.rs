use nom::combinator::all_consuming;
use nom::combinator::map;

pub mod interface;
use interface::*;

mod binary;
mod helpers;
mod literal;
mod operator;
mod unary;
mod variable;

pub fn parse(s: &str) -> R<AST> {
    all_consuming(map(helpers::ws(binary::assignment), |node| AST { node }))(Span::from(s))
}
