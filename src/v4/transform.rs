use super::helpers::*;
use crate::parser::interface::{Node, SyntaxKind, AST};

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

static OP_SET_VAR_INT: &'static str = "SET_VAR_INT";
static OP_SET_VAR_FLOAT: &'static str = "SET_VAR_FLOAT";
static OP_SET_LVAR_INT: &'static str = "SET_LVAR_INT";
static OP_SET_LVAR_FLOAT: &'static str = "SET_LVAR_FLOAT";

static OP_INT_ADD: &'static str = "INT_ADD";
static OP_INT_SUB: &'static str = "INT_SUB";
static OP_INT_MUL: &'static str = "INT_MUL";
static OP_INT_DIV: &'static str = "INT_DIV";

pub fn try_tranform(ast: &AST, expr: &str) -> Option<String> {
    let e = ast.body.get(0)?;

    match e {
        Node::Unary(e) => {
            if e.get_operator() == &SyntaxKind::OperatorBitwiseNot {
                if is_variable(&e.operand) {
                    return format_unary(OP_NOT_UNARY, token_str(expr, as_token(&e.operand)?));
                }
            }
        }
        Node::Binary(e) => {
            let left = &e.left;
            let right = &e.right;

            if !is_variable(left) {
                return None;
            }

            let left_token = as_token(&left)?;

            match right.as_ref() {
                Node::Unary(unary)
                    if e.get_operator() == &SyntaxKind::OperatorEqual
                        && unary.get_operator() == &SyntaxKind::OperatorBitwiseNot =>
                {
                    // var = ~var
                    return format_binary(
                        OP_NOT,
                        token_str(expr, left_token),
                        token_str(expr, as_token(&unary.operand)?),
                    );
                }
                Node::Binary(binary_expr) => match binary_expr.get_operator() {
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
                    SyntaxKind::OperatorPlus => {
                        return format_ternary(
                            OP_INT_ADD,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorMinus => {
                        return format_ternary(
                            OP_INT_SUB,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorMul => {
                        return format_ternary(
                            OP_INT_MUL,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    SyntaxKind::OperatorDiv => {
                        return format_ternary(
                            OP_INT_DIV,
                            token_str(expr, left_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                        )
                    }
                    _ => return None,
                },
                _ => {
                    let right_token = as_token(&right)?;
                    match e.get_operator() {
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
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    // var = int
                                    return format_binary(
                                        OP_SET_VAR_INT,
                                        token_str(expr, left_token),
                                        token_str(expr, right_token),
                                    );
                                }
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    // var = float
                                    return format_binary(
                                        OP_SET_VAR_FLOAT,
                                        token_str(expr, left_token),
                                        token_str(expr, right_token),
                                    );
                                }
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    // lvar = int
                                    return format_binary(
                                        OP_SET_LVAR_INT,
                                        token_str(expr, left_token),
                                        token_str(expr, right_token),
                                    );
                                }
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    // lvar = float
                                    return format_binary(
                                        OP_SET_LVAR_FLOAT,
                                        token_str(expr, left_token),
                                        token_str(expr, right_token),
                                    );
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }

    None
}
