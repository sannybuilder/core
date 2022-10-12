use crate::parser::interface::*;

pub fn is_unary(node: &Node) -> bool {
    as_unary(node).is_some()
}

pub fn as_unary(node: &Node) -> Option<&UnaryPrefixExpr> {
    match node {
        Node::Unary(e) => Some(e),
        _ => None,
    }
}

pub fn is_binary(node: &Node) -> bool {
    as_binary(&node).is_some()
}

pub fn as_binary(node: &Node) -> Option<&BinaryExpr> {
    match node {
        Node::Binary(e) => Some(e),
        _ => None,
    }
}

pub fn is_variable(node: &Node) -> bool {
    as_variable(node).is_some()
}

pub fn as_variable(node: &Node) -> Option<&Variable> {
    match node {
        Node::Variable(e) => Some(e),
        _ => None,
    }
}

pub fn is_token(node: &Node) -> bool {
    as_token(&node).is_some()
}

pub fn as_token(node: &Node) -> Option<&Token> {
    match node {
        Node::Literal(e) => Some(e),
        Node::Unary(e) => Some(&e.token),
        Node::Variable(e) => match e {
            Variable::Local(v) => Some(&v.token),
            Variable::Global(v) => Some(&v.token),
            Variable::ArrayElement(v) => Some(&v.token),
            Variable::Indexed(v) => Some(&v.token),
            Variable::Adma(v) => Some(&v.token),
        },
        _ => None,
    }
}

pub fn format_unary(command_name: &str, operand: &str) -> Option<String> {
    [command_name, operand].join(" ").into()
}

pub fn format_binary(command_name: &str, operand1: &str, operand2: &str) -> Option<String> {
    [command_name, operand1, operand2].join(" ").into()
}

pub fn format_ternary(
    command_name: &str,
    operand1: &str,
    operand2: &str,
    operand3: &str,
) -> Option<String> {
    [command_name, operand1, operand2, operand3]
        .join(" ")
        .into()
}

pub fn token_str<'a>(s: &'a str, token: &Token) -> &'a str {
    let start = token.start - 1;
    let end = start + token.len;
    &s[start..end]
}
