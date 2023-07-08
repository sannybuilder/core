use crate::{legacy_ini::OpcodeTable, namespaces::namespaces::OpId, parser::interface::*};

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

pub fn format_unary(op: OpId, dest_var: &str) -> Option<String> {
    format!("{:04X}: {dest_var}", op).into()
}

pub fn format_binary(
    op: OpId,
    dest_var: &str,
    operand: &str,
    legacy_ini: &OpcodeTable,
) -> Option<String> {
    let param_count = legacy_ini.get_params_count(op);
    // get the index of the last parameter
    let var_index = legacy_ini.get_param_real_index(op, (param_count - 1) as usize);
    if var_index + 1 == param_count {
        // if variable is the last parameter, then this is SCR mode
        format_binary_no_reorder(op, operand, dest_var)
    } else {
        format_binary_no_reorder(op, dest_var, operand)
    }
}

pub fn format_binary_no_reorder(op: OpId, operand1: &str, operand2: &str) -> Option<String> {
    format!("{:04X}: {operand1} {operand2}", op).into()
}

pub fn format_ternary(
    op: OpId,
    dest_var: &str,
    operand1: &str,
    operand2: &str,
    legacy_ini: &OpcodeTable,
) -> Option<String> {
    let param_count = legacy_ini.get_params_count(op);
    // get the index of the last parameter
    let var_index = legacy_ini.get_param_real_index(op, (param_count - 1) as usize);

    if var_index + 1 == param_count {
        // if variable is the last parameter, then this is SCR mode
        format_ternary_no_reorder(op, operand1, operand2, dest_var)
    } else {
        format_ternary_no_reorder(op, dest_var, operand1, operand2)
    }
}

pub fn format_ternary_no_reorder(
    op: OpId,
    operand1: &str,
    operand2: &str,
    operand3: &str,
) -> Option<String> {
    format!("{:04X}: {operand1} {operand2} {operand3}", op).into()
}

pub fn token_str<'a>(s: &'a str, token: &Token) -> &'a str {
    let start = token.start - 1;
    let end = start + token.len;
    &s[start..end]
}
