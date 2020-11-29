use crate::parser::interface::*;

use crate::parser::binary;
use crate::parser::helpers;

pub fn statement(s: Span) -> R<Node> {
    helpers::ws(binary::assignment)(s)
}
