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
        ( vcs: $vcs:expr; else $e:expr) => {
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
                if e.get_operand().is_variable() {
                    // ~var
                    return format_unary(OP_NOT_UNARY, e.get_operand().get_text(expr));
                }
            }
        }
        Node::Binary(e) => {
            let left = &e.left;
            let right = &e.right;

            match right.as_ref() {
                Node::Unary(unary)
                    if left.is_variable()
                        && e.get_operator() == &SyntaxKind::OperatorEqual
                        && unary.get_operator() == &SyntaxKind::OperatorBitwiseNot =>
                {
                    // var = ~var
                    return format_binary(
                        OP_NOT,
                        left.get_text(expr),
                        unary.get_operand().get_text(expr),
                    );
                }
                Node::Binary(binary_expr) if left.is_variable() => {
                    macro_rules! ternary {
                        ($op:expr) => {
                            format_ternary(
                                $op,
                                left.get_text(expr),
                                binary_expr.left.get_text(expr),
                                binary_expr.right.get_text(expr),
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
                _ if left.is_variable() && (right.is_variable() || right.is_literal()) => {
                    macro_rules! binary {
                        ($op:expr) => {
                            format_binary($op, left.get_text(expr), right.get_text(expr))
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

                        SyntaxKind::OperatorEqual if right.is_literal() => {
                            let left_var = left.as_variable()?;
                            let literal = right.as_literal()?;
                            match literal.syntax_kind {
                                // var = int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "0004:"; else "0004:"));
                                }
                                // var = float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "0005:"; else "0005:"));
                                }
                                // lvar = int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "0004:"; else "0006:"));
                                }
                                // lvar = float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "0005:"; else "0007:"));
                                }
                                _ => {}
                            }
                        }
                        SyntaxKind::OperatorEqual if right.is_variable() => {
                            let left_var = left.as_variable()?;
                            let right_var = right.as_variable()?;

                            if left_var.is_global() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0035:"; else "0084:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0036:"; else "0086:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0035:"; else "0085:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0036:"; else "0087:"));
                                }
                            }

                            if left_var.is_global() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0035:"; else "008A:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0036:"; else "0088:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0035:"; else "008B:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0036:"; else "0089:"));
                                }
                            }
                        }
                        SyntaxKind::OperatorPlusEqual if right.is_literal() => {
                            let left_var = left.as_variable()?;
                            let literal = right.as_literal()?;
                            match literal.syntax_kind {
                                // var += int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "0007:"; else "0008:"));
                                }
                                // var += float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "0008:"; else "0009:"));
                                }
                                // lvar += int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "0007:"; else "000A:"));
                                }
                                // lvar += float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "0008:"; else "000B:"));
                                }
                                _ => {}
                            }
                        }

                        SyntaxKind::OperatorPlusEqual if right.is_variable() => {
                            let left_var = left.as_variable()?;
                            let right_var = right.as_variable()?;

                            if left_var.is_global() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0029:"; else "0058:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002A:"; else "0059:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0029:"; else "005A:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002A:"; else "005B:"));
                                }
                            }

                            if left_var.is_global() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0029:"; else "005E:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002A:"; else "005F:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "0029:"; else "005C:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002A:"; else "005D:"));
                                }
                            }
                        }
                        SyntaxKind::OperatorMinusEqual if right.is_literal() => {
                            let left_var = left.as_variable()?;
                            let literal = right.as_literal()?;
                            match literal.syntax_kind {
                                // var -= int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "0009:"; else "000C:"));
                                }
                                // var -= float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "000A:"; else "000D:"));
                                }
                                // lvar -= int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "0009:"; else "000E:"));
                                }
                                // lvar -= float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "000A:"; else "000F:"));
                                }
                                _ => {}
                            }
                        }
                        SyntaxKind::OperatorMinusEqual if right.is_variable() => {
                            let left_var = left.as_variable()?;
                            let right_var = right.as_variable()?;

                            if left_var.is_global() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002B:"; else "0060:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002C:"; else "0061:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002B:"; else "0062:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002C:"; else "0063:"));
                                }
                            }

                            if left_var.is_global() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002B:"; else "0066:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002C:"; else "0067:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002B:"; else "0064:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002C:"; else "0065:"));
                                }
                            }
                        }
                        SyntaxKind::OperatorMulEqual if right.is_literal() => {
                            let left_var = left.as_variable()?;
                            let literal = right.as_literal()?;
                            match literal.syntax_kind {
                                // var *= int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "000B:"; else "0010:"));
                                }
                                // var *= float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "000C:"; else "0011:"));
                                }
                                // lvar *= int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "000B:"; else "0012:"));
                                }
                                // lvar *= float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "000C:"; else "0013:"));
                                }
                                _ => {}
                            }
                        }
                        SyntaxKind::OperatorMulEqual if right.is_variable() => {
                            let left_var = left.as_variable()?;
                            let right_var = right.as_variable()?;

                            if left_var.is_global() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002D:"; else "0068:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002E:"; else "0069:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002D:"; else "006A:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002E:"; else "006B:"));
                                }
                            }

                            if left_var.is_global() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002D:"; else "006E:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002E:"; else "006F:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002D:"; else "006C:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "002E:"; else "006D:"));
                                }
                            }
                        }
                        SyntaxKind::OperatorDivEqual if right.is_literal() => {
                            let left_var = left.as_variable()?;
                            let literal = right.as_literal()?;
                            match literal.syntax_kind {
                                // var /= int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "000D:"; else "0014:"));
                                }
                                // var /= float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "000E:"; else "0015:"));
                                }
                                // lvar /= int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "000D:"; else "0016:"));
                                }
                                // lvar /= float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "000E:"; else "0017:"));
                                }
                                _ => {}
                            }
                        }
                        SyntaxKind::OperatorDivEqual if right.is_variable() => {
                            let left_var = left.as_variable()?;
                            let right_var = right.as_variable()?;

                            if left_var.is_global() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002F:"; else "0070:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0030:"; else "0071:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002F:"; else "0072:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0030:"; else "0073:"));
                                }
                            }

                            if left_var.is_global() && right_var.is_local() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002F:"; else "0074:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0030:"; else "0075:"));
                                }
                            }

                            if left_var.is_local() && right_var.is_global() {
                                if left_var.is_integer() && right_var.is_integer() {
                                    return binary!(g!(vcs: "002F:"; else "0076:"));
                                }
                                if left_var.is_float() && right_var.is_float() {
                                    return binary!(g!(vcs: "0030:"; else "0077:"));
                                }
                            }
                        }
                        SyntaxKind::OperatorGreater if right.is_literal() => {
                            let left_var = left.as_variable()?;
                            let literal = right.as_literal()?;
                            match literal.syntax_kind {
                                // var > int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "000F:"; else "0018:"));
                                }
                                // var > float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    return binary!(g!(vcs: "0012:"; else "0020:"));
                                }
                                // lvar > int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "000F:"; else "0019:"));
                                }
                                // lvar > float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    return binary!(g!(vcs: "0012:"; else "0021:"));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    // left is not a variable
                }
            }
        }
        _ => {}
    }

    None
}
