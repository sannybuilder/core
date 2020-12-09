use crate::parser::interface::*;

use crate::parser::binary;
use crate::parser::helpers;

pub fn expression(s: Span) -> R<Node> {
    helpers::line(binary::assignment)(s)
}
