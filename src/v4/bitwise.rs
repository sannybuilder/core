use crate::parser::interface::*;

static OP_AND: &'static str = "BIT_AND";
static OP_OR: &'static str = "BIT_OR";
static OP_XOR: &'static str = "BIT_XOR";
static OP_NOT: &'static str = "BIT_NOT";
static OP_MOD: &'static str = "MOD";
static OP_SHR: &'static str = "BIT_SHR";
static OP_SHL: &'static str = "BIT_SHL";
static OP_AND_COMPOUND: &'static str = "BIT_AND_COMPOUND";
static OP_OR_COMPOUND: &'static str = "BIT_OR_COMPOUND";
static OP_XOR_COMPOUND: &'static str = "BIT_XOR_COMPOUND";
static OP_NOT_UNARY: &'static str = "BIT_NOT_COMPOUND";
static OP_MOD_COMPOUND: &'static str = "MOD_COMPOUND";
static OP_SHR_COMPOUND: &'static str = "BIT_SHR_COMPOUND";
static OP_SHL_COMPOUND: &'static str = "BIT_SHL_COMPOUND";

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
        Node::Variable(e) => match e {
            Variable::Local(v) => Some(&v.token),
            Variable::Global(v) => Some(&v.token),
            Variable::ArrayElement(v) => Some(&v.token),
            Variable::Indexed(v) => Some(&v.token),
        },
        _ => None,
    }
}

fn as_variable(node: &Node) -> Option<&Token> {
    let token = as_token(node)?;
    match token.syntax_kind {
        SyntaxKind::LocalVariable
        | SyntaxKind::GlobalVariable
        | SyntaxKind::ArrayElementSCR
        | SyntaxKind::IndexedVariable => Some(token),
        _ => None,
    }
}

fn get_unary_operator(expr: &UnaryPrefixExpr) -> &SyntaxKind {
    &expr.operator.syntax_kind
}

fn get_binary_operator(expr: &BinaryExpr) -> &SyntaxKind {
    &expr.operator.syntax_kind
}

fn format_unary(command_name: &str, operand: &str) -> Option<String> {
    [command_name, operand].join(" ").into()
}

fn format_binary(command_name: &str, operand1: &str, operand2: &str) -> Option<String> {
    [command_name, operand1, operand2].join(" ").into()
}

fn format_ternary(
    command_name: &str,
    operand1: &str,
    operand2: &str,
    operand3: &str,
) -> Option<String> {
    [command_name, operand1, operand2, operand3]
        .join(" ")
        .into()
}

fn token_str<'a>(s: &'a str, token: &Token) -> &'a str {
    let start = token.start - 1;
    let end = start + token.len;
    &s[start..end]
}

pub fn try_bitwise(ast: &AST, expr: &str) -> Option<String> {
    let e = ast.body.get(0)?;

    if is_unary(&e) {
        let e = as_unary(&e)?;
        if get_unary_operator(e) == &SyntaxKind::OperatorBitwiseNot {
            if let Some(var) = as_variable(e.operand.as_ref()) {
                return format_unary(OP_NOT_UNARY, token_str(expr, var));
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
                    SyntaxKind::OperatorBitwiseAndEqual => {
                        return format_binary(
                            OP_AND_COMPOUND,
                            token_str(expr, left_token),
                            token_str(expr, right_token),
                        )
                    }
                    SyntaxKind::OperatorBitwiseOrEqual => {
                        return format_binary(
                            OP_OR_COMPOUND,
                            token_str(expr, left_token),
                            token_str(expr, right_token),
                        )
                    }
                    SyntaxKind::OperatorBitwiseXorEqual => {
                        return format_binary(
                            OP_XOR_COMPOUND,
                            token_str(expr, left_token),
                            token_str(expr, right_token),
                        )
                    }
                    SyntaxKind::OperatorBitwiseModEqual => {
                        return format_binary(
                            OP_MOD_COMPOUND,
                            token_str(expr, left_token),
                            token_str(expr, right_token),
                        )
                    }
                    SyntaxKind::OperatorBitwiseShrEqual => {
                        return format_binary(
                            OP_SHR_COMPOUND,
                            token_str(expr, left_token),
                            token_str(expr, right_token),
                        )
                    }
                    SyntaxKind::OperatorBitwiseShlEqual => {
                        return format_binary(
                            OP_SHL_COMPOUND,
                            token_str(expr, left_token),
                            token_str(expr, right_token),
                        )
                    }
                    SyntaxKind::OperatorEqual => {
                        // var = ~var
                        let unary = as_unary(right)?;
                        if get_unary_operator(unary) == &SyntaxKind::OperatorBitwiseNot {
                            return format_binary(
                                OP_NOT,
                                token_str(expr, left_token),
                                token_str(expr, as_token(unary.operand.as_ref())?),
                            );
                        }
                    }
                    _ => return None,
                }
            }

            if let Some(binary_expr) = as_binary(right.as_ref()) {
                match get_binary_operator(binary_expr) {
                    SyntaxKind::OperatorBitwiseAnd => {
                        return format_ternary(
                            OP_AND,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorBitwiseOr => {
                        return format_ternary(
                            OP_OR,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorBitwiseXor => {
                        return format_ternary(
                            OP_XOR,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorBitwiseMod => {
                        return format_ternary(
                            OP_MOD,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorBitwiseShr => {
                        return format_ternary(
                            OP_SHR,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorBitwiseShl => {
                        return format_ternary(
                            OP_SHL,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    _ => return None,
                }
            }
        }
    }
    None
}
