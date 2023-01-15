use super::{game::Game, helpers::*};
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

static OP_INT_ADD: &'static str = "INT_ADD";
static OP_INT_SUB: &'static str = "INT_SUB";
static OP_INT_MUL: &'static str = "INT_MUL";
static OP_INT_DIV: &'static str = "INT_DIV";

pub fn try_tranform(ast: &AST, expr: &str, game: Game) -> Option<String> {
    macro_rules! g {
        ( vcs: ($vcs:expr) else $e:expr) => {
            if game == Game::vcs {
                $vcs
            } else {
                $e
            }
        };
    }
    let e = ast.body.get(0)?;

    match e {
        Node::Unary(e) => {
            if e.get_operator() == &SyntaxKind::OperatorBitwiseNot {
                if is_variable(&e.operand) {
                    // ~var
                    return format_unary(OP_NOT_UNARY, token_str(expr, as_token(&e.operand)?));
                }
            }
        }
        Node::Binary(e) => {
            let left = &e.left;
            let right = &e.right;

            if !is_variable(left) {
                // todo: 1 > var ?
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
                Node::Binary(binary_expr) => {
                    macro_rules! ternary {
                        ($op:expr) => {
                            format_ternary(
                                $op,
                                token_str(expr, left_token),
                                token_str(expr, as_token(&binary_expr.left)?),
                                token_str(expr, as_token(&binary_expr.right)?),
                            )
                        };
                    }
                    match binary_expr.get_operator() {
                        // var = var & var
                        SyntaxKind::OperatorBitwiseAnd => return ternary!(OP_AND),
                        // var = var | var
                        SyntaxKind::OperatorBitwiseOr => return ternary!(OP_OR),
                        // var = var ^ var
                        SyntaxKind::OperatorBitwiseXor => return ternary!(OP_XOR),
                        // var = var % var
                        SyntaxKind::OperatorBitwiseMod => return ternary!(OP_MOD),
                        // var = var >> var
                        SyntaxKind::OperatorBitwiseShr => return ternary!(OP_SHR),
                        // var = var << var
                        SyntaxKind::OperatorBitwiseShl => return ternary!(OP_SHL),
                        // var = var + var
                        SyntaxKind::OperatorPlus => return ternary!(OP_INT_ADD),
                        // var = var - var
                        SyntaxKind::OperatorMinus => return ternary!(OP_INT_SUB),
                        // var = var * var
                        SyntaxKind::OperatorMul => return ternary!(OP_INT_MUL),
                        // var = var / var
                        SyntaxKind::OperatorDiv => return ternary!(OP_INT_DIV),
                        _ => return None,
                    }
                }
                _ => {
                    let right_token = as_token(&right)?;
                    macro_rules! binary {
                        ($op:expr) => {
                            format_binary(
                                $op,
                                token_str(expr, left_token),
                                token_str(expr, right_token),
                            )
                        };
                    }
                    match e.get_operator() {
                        // var &= var
                        SyntaxKind::OperatorBitwiseAndEqual => return binary!(OP_AND_COMPOUND),
                        // var |= var
                        SyntaxKind::OperatorBitwiseOrEqual => return binary!(OP_OR_COMPOUND),
                        // var ^= var
                        SyntaxKind::OperatorBitwiseXorEqual => return binary!(OP_XOR_COMPOUND),
                        // var %= var
                        SyntaxKind::OperatorBitwiseModEqual => return binary!(OP_MOD_COMPOUND),
                        // var >>= var
                        SyntaxKind::OperatorBitwiseShrEqual => return binary!(OP_SHR_COMPOUND),
                        // var <<= var
                        SyntaxKind::OperatorBitwiseShlEqual => return binary!(OP_SHL_COMPOUND),

                        SyntaxKind::OperatorEqual => {
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                // var = int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: ("0004:") else "0004:"));
                                }
                                // var = float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: ("0005:") else "0005:"));
                                }
                                // lvar = int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: ("0004:") else "0006:"));
                                }
                                // lvar = float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: ("0005:") else "0007:"));
                                }
                                _ => {}
                            }
                        }
                        SyntaxKind::OperatorPlusEqual => {
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                // var += int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: ("0007:") else "0008:"));
                                }
                                // var += float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: ("0008:") else "0009:"));
                                }
                                // lvar += int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: ("0007:") else "000A:"));
                                }
                                // lvar += float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: ("0008:") else "000B:"));
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
