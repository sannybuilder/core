use crate::parser::interface::*;

use crate::parser::binary;

pub fn expression(s: Span) -> R<Node> {
    binary::assignment(s)
}
