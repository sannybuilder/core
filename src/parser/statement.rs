use crate::parser::interface::*;

use crate::parser::expression;

pub fn statement(s: Span) -> R<Node> {
    expression::expression(s)
}
