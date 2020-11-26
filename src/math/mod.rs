use ast::{BinaryExpr, UnaryPrefixExpr};

use crate::ast;
use crate::ast::Node;
use crate::ast::SyntaxKind;
use crate::ast::Token;

static OP_AND: u16 = 0x0B10;
static OP_OR: u16 = 0x0B11;
static OP_XOR: u16 = 0x0B12;
static OP_NOT: u16 = 0x0B13;
static OP_MOD: u16 = 0x0B14;
static OP_SHR: u16 = 0x0B15;
static OP_SHL: u16 = 0x0B16;
static OP_AND_COMPOUND: u16 = 0x0B17;
static OP_OR_COMPOUND: u16 = 0x0B18;
static OP_XOR_COMPOUND: u16 = 0x0B19;
static OP_NOT_UNARY: u16 = 0x0B1A;
static OP_MOD_COMPOUND: u16 = 0x0B1B;
static OP_SHR_COMPOUND: u16 = 0x0B1C;
static OP_SHL_COMPOUND: u16 = 0x0B1D;

fn is_unary(node: &Node) -> bool {
    as_unary(node).is_some()
}

fn as_unary(node: &Node) -> Option<&UnaryPrefixExpr> {
    match node {
        Node::Unary(e) => Some(e),
        _ => None,
    }
}

fn is_binary(node: &Node) -> bool {
    as_binary(&node).is_some()
}

fn as_binary(node: &Node) -> Option<&BinaryExpr> {
    match node {
        Node::Binary(e) => Some(e),
        _ => None,
    }
}

fn is_token(node: &Node) -> bool {
    as_token(&node).is_some()
}

fn as_token(node: &Node) -> Option<&Token> {
    match node {
        Node::Token(e) => Some(e),
        Node::Unary(e) => Some(&e.token),
        _ => None,
    }
}

fn as_variable(node: &Node) -> Option<&Token> {
    let token = as_token(node)?;
    match token.syntax_kind {
        SyntaxKind::LocalVariable | SyntaxKind::GlobalVariable | SyntaxKind::Array => Some(token),
        _ => None,
    }
}

fn get_unary_operator(expr: &UnaryPrefixExpr) -> &SyntaxKind {
    &expr.operator.syntax_kind
}

fn get_binary_operator(expr: &BinaryExpr) -> &SyntaxKind {
    &expr.operator.syntax_kind
}

fn format_unary(opcode: u16, operand: &String) -> Option<String> {
    Some(format!("{:04X}: {}", opcode, operand))
}

fn format_binary(opcode: u16, operand1: &String, operand2: &String) -> Option<String> {
    format_unary(opcode, &format!("{} {}", operand1, operand2))
}

fn format_ternary(
    opcode: u16,
    operand1: &String,
    operand2: &String,
    operand3: &String,
) -> Option<String> {
    format_unary(opcode, &format!("{} {} {}", operand1, operand2, operand3))
}

fn token_str<'a>(s: &'a str, token: &Token) -> &'a str {
    let start = token.start - 1;
    let end = start + token.len;
    &s[start..end]
}

pub fn to_command(expr: &str) -> Option<String> {
    let e = ast::parse(expr).ok()?.1.node;

    if is_unary(&e) {
        let e = as_unary(&e)?;
        if get_unary_operator(e) == &SyntaxKind::OperatorNot {
            if let Some(var) = as_variable(e.operand.as_ref()) {
                return format_unary(OP_NOT_UNARY, &String::from(token_str(expr, var)));
            }
        }
    } else if is_binary(&e) {
        let e = as_binary(&e)?;
        let left = &e.left;
        let right = &e.right;
        let op = &e.operator;

        if let Some(left_token) = as_token(left.as_ref()) {
            if let Some(right_token) = as_token(right.as_ref()) {
                match op.syntax_kind {
                    SyntaxKind::OperatorAndEqual => {
                        return format_binary(
                            OP_AND_COMPOUND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, right_token)),
                        )
                    }
                    SyntaxKind::OperatorOrEqual => {
                        return format_binary(
                            OP_OR_COMPOUND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, right_token)),
                        )
                    }
                    SyntaxKind::OperatorXorEqual => {
                        return format_binary(
                            OP_XOR_COMPOUND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, right_token)),
                        )
                    }
                    SyntaxKind::OperatorModEqual => {
                        return format_binary(
                            OP_MOD_COMPOUND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, right_token)),
                        )
                    }
                    SyntaxKind::OperatorShrEqual => {
                        return format_binary(
                            OP_SHR_COMPOUND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, right_token)),
                        )
                    }
                    SyntaxKind::OperatorShlEqual => {
                        return format_binary(
                            OP_SHL_COMPOUND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, right_token)),
                        )
                    }
                    SyntaxKind::OperatorEqual => {
                        // var = ~var
                        let unary = as_unary(right)?;
                        if get_unary_operator(unary) == &SyntaxKind::OperatorNot {
                            return format_binary(
                                OP_NOT,
                                &String::from(token_str(expr, left_token)),
                                &String::from(token_str(expr, as_token(unary.operand.as_ref())?)),
                            );
                        }
                    }
                    _ => return None,
                }
            }

            if let Some(binary_expr) = as_binary(right.as_ref()) {
                match get_binary_operator(binary_expr) {
                    SyntaxKind::OperatorAnd => {
                        return format_ternary(
                            OP_AND,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, as_token(&binary_expr.left)?)),
                            &String::from(token_str(expr, as_token(&binary_expr.right)?)),
                        )
                    }
                    SyntaxKind::OperatorOr => {
                        return format_ternary(
                            OP_OR,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, as_token(&binary_expr.left)?)),
                            &String::from(token_str(expr, as_token(&binary_expr.right)?)),
                        )
                    }
                    SyntaxKind::OperatorXor => {
                        return format_ternary(
                            OP_XOR,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, as_token(&binary_expr.left)?)),
                            &String::from(token_str(expr, as_token(&binary_expr.right)?)),
                        )
                    }
                    SyntaxKind::OperatorMod => {
                        return format_ternary(
                            OP_MOD,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, as_token(&binary_expr.left)?)),
                            &String::from(token_str(expr, as_token(&binary_expr.right)?)),
                        )
                    }
                    SyntaxKind::OperatorShr => {
                        return format_ternary(
                            OP_SHR,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, as_token(&binary_expr.left)?)),
                            &String::from(token_str(expr, as_token(&binary_expr.right)?)),
                        )
                    }
                    SyntaxKind::OperatorShl => {
                        return format_ternary(
                            OP_SHL,
                            &String::from(token_str(expr, left_token)),
                            &String::from(token_str(expr, as_token(&binary_expr.left)?)),
                            &String::from(token_str(expr, as_token(&binary_expr.right)?)),
                        )
                    }
                    _ => return None,
                }
            }
        }
    }
    None
}

#[test]

fn test_unary() {
    assert_eq!(to_command("~0@"), Some(String::from("0B1A: 0@")));
    assert_eq!(to_command("~$var"), Some(String::from("0B1A: $var")));
    assert_eq!(
        to_command("~10@($_,1i)"),
        Some(String::from("0B1A: 10@($_,1i)"))
    );
    assert_eq!(
        to_command("~$0101(1000@,12f)"),
        Some(String::from("0B1A: $0101(1000@,12f)"))
    );
}

#[test]
fn test_binary() {
    assert_eq!(to_command("0@ &= 1@"), Some(String::from("0B17: 0@ 1@")));
    assert_eq!(to_command("0@ &= 100"), Some(String::from("0B17: 0@ 100")));
    assert_eq!(
        to_command("0@ &= 42.01"),
        Some(String::from("0B17: 0@ 42.01"))
    );

    assert_eq!(to_command("0@ &= -1"), Some(String::from("0B17: 0@ -1")));
    assert_eq!(to_command("0@ |= 1@"), Some(String::from("0B18: 0@ 1@")));
    assert_eq!(to_command("0@ ^= 1@"), Some(String::from("0B19: 0@ 1@")));
    assert_eq!(to_command("0@ %= 1@"), Some(String::from("0B1B: 0@ 1@")));
    assert_eq!(to_command("0@ >>= 1@"), Some(String::from("0B1C: 0@ 1@")));
    assert_eq!(to_command("0@ <<= 1@"), Some(String::from("0B1D: 0@ 1@")));
}

#[test]
fn test_ternary() {
    println!("{:#?}", ast::parse("0@ = -1 & 1@"));
    assert_eq!(
        to_command("0@ = -1 & 1@"),
        Some(String::from("0B10: 0@ -1 1@"))
    );
    assert_eq!(
        to_command("0@ = 1 | 1@"),
        Some(String::from("0B11: 0@ 1 1@"))
    );
    assert_eq!(
        to_command("0@ = 1 ^ 1@"),
        Some(String::from("0B12: 0@ 1 1@"))
    );
    assert_eq!(
        to_command("0@ = 1 % 1@"),
        Some(String::from("0B14: 0@ 1 1@"))
    );
    assert_eq!(
        to_command("0@ = 1 >> 1@"),
        Some(String::from("0B15: 0@ 1 1@"))
    );
    assert_eq!(
        to_command("0@ = 1 << 1@"),
        Some(String::from("0B16: 0@ 1 1@"))
    );
}

#[test]
fn test_not() {
    assert_eq!(to_command("0@ = ~1@"), Some(String::from("0B13: 0@ 1@")));
    assert_eq!(to_command("~0@"), Some(String::from("0B1A: 0@")));
}
