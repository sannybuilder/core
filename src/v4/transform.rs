use super::{game::Game, helpers::*};
use crate::parser::interface::{Node, SyntaxKind};

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

pub fn try_transform(e: &Node, expr: &str, game: Game, not: bool) -> Option<String> {
    match e {
        Node::Unary(e) => {
            match e.get_operator() {
                SyntaxKind::OperatorBitwiseNot => {
                    if e.get_operand().is_variable() {
                        // ~var
                        return format_unary(OP_NOT_UNARY, e.get_operand().get_text(expr));
                    }
                }
                SyntaxKind::OperatorNot => {
                    return try_transform(e.get_operand(), expr, game, !not);
                }
                _ => {
                    unreachable!()
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
                        _ => {}
                    }
                }
                _ if (left.is_variable() || left.is_literal())
                    && (right.is_variable() || right.is_literal()) =>
                {
                    let op = e.get_operator();
                    match op {
                        SyntaxKind::OperatorLess => {
                            // < :: not >=
                            return transform_binary(
                                left,
                                right,
                                &SyntaxKind::OperatorGreaterEqual,
                                &game,
                                expr,
                                !not,
                            );
                        }
                        SyntaxKind::OperatorLessEqual => {
                            // <= :: not >
                            return transform_binary(
                                left,
                                right,
                                &SyntaxKind::OperatorGreater,
                                &game,
                                expr,
                                !not,
                            );
                        }
                        SyntaxKind::OperatorNotEqual => {
                            // <> :: not ==
                            return transform_binary(
                                left,
                                right,
                                &SyntaxKind::OperatorEqualEqual,
                                &game,
                                expr,
                                !not,
                            );
                        }
                        _ => {
                            return transform_binary(
                                left,
                                right,
                                e.get_operator(),
                                &game,
                                expr,
                                not,
                            )
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    None
}

fn transform_binary(
    left: &Node,
    right: &Node,
    op: &SyntaxKind,
    game: &Game,
    expr: &str,
    not: bool,
) -> Option<String> {
    macro_rules! binary {
        ($op:expr) => {
            Some(format!(
                "{} {} {}",
                $op,
                left.get_text(expr),
                right.get_text(expr)
            ))
        };
    }
    macro_rules! g {
        ( vcs:$vcs:expr; else $e:expr) => {{
            let mut op = (if game == &Game::vcs { $vcs } else { $e });
            if !not {
                op |= 0x8000
            };
            format!("{:04X}:", op).as_str()
        }};
    }
    match op {
        // var &= var
        SyntaxKind::OperatorBitwiseAndEqual if left.is_variable() => {
            return binary!(OP_AND_COMPOUND)
        }
        // var |= var
        SyntaxKind::OperatorBitwiseOrEqual if left.is_variable() => return binary!(OP_OR_COMPOUND),
        // var ^= var
        SyntaxKind::OperatorBitwiseXorEqual if left.is_variable() => {
            return binary!(OP_XOR_COMPOUND)
        }
        // var %= var
        SyntaxKind::OperatorBitwiseModEqual if left.is_variable() => {
            return binary!(OP_MOD_COMPOUND)
        }
        // var >>= var
        SyntaxKind::OperatorBitwiseShrEqual if left.is_variable() => {
            return binary!(OP_SHR_COMPOUND)
        }
        // var <<= var
        SyntaxKind::OperatorBitwiseShlEqual if left.is_variable() => {
            return binary!(OP_SHL_COMPOUND)
        }

        SyntaxKind::OperatorEqual if left.is_variable() && right.is_literal() => {
            let left_var = left.as_variable()?;
            let literal = right.as_literal()?;
            if left_var.is_global() {
                // var = int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0004; else 0x0004));
                }
                // var = float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0005; else 0x0005));
                }
            }
            if left_var.is_local() {
                // lvar = int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0004; else 0x0006));
                }
                // lvar = float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0005; else 0x0007));
                }
            }
        }

        SyntaxKind::OperatorEqual if left.is_variable() && right.is_variable() => {
            let left_var = left.as_variable()?;
            let right_var = right.as_variable()?;

            if left_var.is_global() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0035; else 0x0084));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0036; else 0x0086));
                }
            }

            if left_var.is_local() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0035; else 0x0085));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0036; else 0x0087));
                }
            }

            if left_var.is_global() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0035; else 0x008A));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0036; else 0x0088));
                }
            }

            if left_var.is_local() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0035; else 0x008B));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0036; else 0x0089));
                }
            }
        }
        SyntaxKind::OperatorPlusEqual if left.is_variable() && right.is_literal() => {
            let left_var = left.as_variable()?;
            let literal = right.as_literal()?;

            if left_var.is_global() {
                // var += int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0007; else 0x0008));
                }
                // var += float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0008; else 0x0009));
                }
            }
            if left_var.is_local() {
                // lvar += int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0007; else 0x000A));
                }
                // lvar += float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0008; else 0x000B));
                }
            }
        }
        SyntaxKind::OperatorPlusEqual if left.is_variable() && right.is_variable() => {
            let left_var = left.as_variable()?;
            let right_var = right.as_variable()?;

            if left_var.is_global() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0029; else 0x0058));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002A; else 0x0059));
                }
            }

            if left_var.is_local() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0029; else 0x005A));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002A; else 0x005B));
                }
            }

            if left_var.is_global() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0029; else 0x005E));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002A; else 0x005F));
                }
            }

            if left_var.is_local() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0029; else 0x005C));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002A; else 0x005D));
                }
            }
        }
        SyntaxKind::OperatorMinusEqual if left.is_variable() && right.is_literal() => {
            let left_var = left.as_variable()?;
            let literal = right.as_literal()?;

            if left_var.is_global() {
                // var -= int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0009; else 0x000C));
                }
                // var -= float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x000A; else 0x000D));
                }
            }
            if left_var.is_local() {
                // lvar -= int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0009; else 0x000E));
                }
                // lvar -= float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x000A; else 0x000F));
                }
            }
        }
        SyntaxKind::OperatorMinusEqual if left.is_variable() && right.is_variable() => {
            let left_var = left.as_variable()?;
            let right_var = right.as_variable()?;

            if left_var.is_global() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002B; else 0x0060));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002C; else 0x0061));
                }
            }

            if left_var.is_local() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002B; else 0x0062));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002C; else 0x0063));
                }
            }

            if left_var.is_global() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002B; else 0x0066));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002C; else 0x0067));
                }
            }

            if left_var.is_local() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002B; else 0x0064));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002C; else 0x0065));
                }
            }
        }
        SyntaxKind::OperatorMulEqual if left.is_variable() && right.is_literal() => {
            let left_var = left.as_variable()?;
            let literal = right.as_literal()?;

            if left_var.is_global() {
                // var *= int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x000B; else 0x0010));
                }
                // var *= float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x000C; else 0x0011));
                }
            }
            if left_var.is_local() {
                // lvar *= int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x000B; else 0x0012));
                }
                // lvar *= float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x000C; else 0x0013));
                }
            }
        }
        SyntaxKind::OperatorMulEqual if left.is_variable() && right.is_variable() => {
            let left_var = left.as_variable()?;
            let right_var = right.as_variable()?;

            if left_var.is_global() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002D; else 0x0068));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002E; else 0x0069));
                }
            }

            if left_var.is_local() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002D; else 0x006A));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002E; else 0x006B));
                }
            }

            if left_var.is_global() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002D; else 0x006E));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002E; else 0x006F));
                }
            }

            if left_var.is_local() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002D; else 0x006C));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x002E; else 0x006D));
                }
            }
        }
        SyntaxKind::OperatorDivEqual if left.is_variable() && right.is_literal() => {
            let left_var = left.as_variable()?;
            let literal = right.as_literal()?;

            if left_var.is_global() {
                // var /= int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x000D; else 0x0014));
                }
                // var /= float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x000E; else 0x0015));
                }
            }
            if left_var.is_local() {
                // lvar /= int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x000D; else 0x0016));
                }
                // lvar /= float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x000E; else 0x0017));
                }
            }
        }
        SyntaxKind::OperatorDivEqual if left.is_variable() && right.is_variable() => {
            let left_var = left.as_variable()?;
            let right_var = right.as_variable()?;

            if left_var.is_global() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002F; else 0x0070));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0030; else 0x0071));
                }
            }

            if left_var.is_local() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002F; else 0x0072));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0030; else 0x0073));
                }
            }

            if left_var.is_global() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002F; else 0x0074));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0030; else 0x0075));
                }
            }

            if left_var.is_local() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x002F; else 0x0076));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0030; else 0x0077));
                }
            }
        }
        SyntaxKind::OperatorGreater if left.is_variable() && right.is_literal() => {
            let left_var = left.as_variable()?;
            let literal = right.as_literal()?;

            if left_var.is_global() {
                // var > int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x000F; else 0x0018));
                }
                // var > float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0012; else 0x0020));
                }
            }
            if left_var.is_local() {
                // lvar > int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x000F; else 0x0019));
                }
                // lvar > float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0012; else 0x0021));
                }
            }
        }
        SyntaxKind::OperatorGreater if left.is_variable() && right.is_variable() => {
            let left_var = left.as_variable()?;
            let right_var = right.as_variable()?;

            if left_var.is_global() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0011; else 0x001C));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0014; else 0x0024));
                }
            }

            if left_var.is_local() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0011; else 0x001D));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0014; else 0x0025));
                }
            }

            if left_var.is_global() && right_var.is_local() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0011; else 0x001E));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0014; else 0x0026));
                }
            }

            if left_var.is_local() && right_var.is_global() {
                if left_var.is_integer() && right_var.is_integer() {
                    return binary!(g!(vcs: 0x0011; else 0x001F));
                }
                if left_var.is_float() && right_var.is_float() {
                    return binary!(g!(vcs: 0x0014; else 0x0027));
                }
            }
        }
        // literal > var
        SyntaxKind::OperatorGreater if left.is_literal() && right.is_variable() => {
            let literal = left.as_literal()?;
            let right_var = right.as_variable()?;

            if right_var.is_global() {
                // var > int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0010; else 0x001A));
                }
                // var > float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0013; else 0x0022));
                }
            }
            if right_var.is_local() {
                // lvar > int
                if literal.is_integer() {
                    return binary!(g!(vcs: 0x0010; else 0x001B));
                }
                // lvar > float
                if literal.is_float() {
                    return binary!(g!(vcs: 0x0013; else 0x0023));
                }
            }
        }
        _ => {}
    }

    None
}
