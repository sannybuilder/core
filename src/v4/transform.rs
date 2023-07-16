use super::helpers::*;
use crate::{
    dictionary::dictionary_num_by_str::DictNumByStr,
    legacy_ini::OpcodeTable,
    namespaces::namespaces::Namespaces,
    parser::interface::{Node, SyntaxKind, AST},
};

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

static OP_ADD_TIMED_VAL_TO_FLOAT_VAR: &'static str = "ADD_TIMED_VAL_TO_FLOAT_VAR";
static OP_ADD_TIMED_VAL_TO_FLOAT_LVAR: &'static str = "ADD_TIMED_VAL_TO_FLOAT_LVAR";
static OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR: &'static str = "ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR";
static OP_ADD_TIMED_FLOAT_LVAR_TO_FLOAT_LVAR: &'static str = "ADD_TIMED_FLOAT_LVAR_TO_FLOAT_LVAR";
static OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_LVAR: &'static str = "ADD_TIMED_FLOAT_VAR_TO_FLOAT_LVAR";
static OP_ADD_TIMED_FLOAT_LVAR_TO_FLOAT_VAR: &'static str = "ADD_TIMED_FLOAT_LVAR_TO_FLOAT_VAR";
static OP_SUB_TIMED_VAL_FROM_FLOAT_VAR: &'static str = "SUB_TIMED_VAL_FROM_FLOAT_VAR";
static OP_SUB_TIMED_VAL_FROM_FLOAT_LVAR: &'static str = "SUB_TIMED_VAL_FROM_FLOAT_LVAR";
static OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR: &'static str = "SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR";
static OP_SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_LVAR: &'static str =
    "SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_LVAR";
static OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_LVAR: &'static str = "SUB_TIMED_FLOAT_VAR_FROM_FLOAT_LVAR";
static OP_SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_VAR: &'static str = "SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_VAR";

static OP_CSET_VAR_INT_TO_VAR_FLOAT: &'static str = "CSET_VAR_INT_TO_VAR_FLOAT";
static OP_CSET_VAR_FLOAT_TO_VAR_INT: &'static str = "CSET_VAR_FLOAT_TO_VAR_INT";
static OP_CSET_LVAR_INT_TO_VAR_FLOAT: &'static str = "CSET_LVAR_INT_TO_VAR_FLOAT";
static OP_CSET_LVAR_FLOAT_TO_VAR_INT: &'static str = "CSET_LVAR_FLOAT_TO_VAR_INT";

static OP_CSET_VAR_INT_TO_LVAR_FLOAT: &'static str = "CSET_VAR_INT_TO_LVAR_FLOAT";
static OP_CSET_VAR_FLOAT_TO_LVAR_INT: &'static str = "CSET_VAR_FLOAT_TO_LVAR_INT";
static OP_CSET_LVAR_INT_TO_LVAR_FLOAT: &'static str = "CSET_LVAR_INT_TO_LVAR_FLOAT";
static OP_CSET_LVAR_FLOAT_TO_LVAR_INT: &'static str = "CSET_LVAR_FLOAT_TO_LVAR_INT";

pub fn try_tranform(
    ast: &AST,
    expr: &str,
    ns: &Namespaces,
    legacy_ini: &OpcodeTable,
    var_types: &DictNumByStr,
) -> Option<String> {
    let e = ast.body.get(0)?;

    return match e {
        Node::Unary(e) => {
            if e.get_operator() == &SyntaxKind::OperatorBitwiseNot {
                if is_variable(&e.operand) {
                    // ~var
                    let op_id = *ns.get_opcode_by_command_name(OP_NOT_UNARY)?;
                    return format_unary(op_id, token_str(expr, as_token(&e.operand)?));
                }
            }
            None
        }
        Node::Binary(e) => {
            let left = &e.left;
            let right = &e.right;

            if !is_variable(left) {
                return None;
            }

            let dest_var_token = as_token(&left)?;

            match right.as_ref() {
                Node::Unary(unary)
                    if e.get_operator() == &SyntaxKind::OperatorEqual
                        && unary.get_operator() == &SyntaxKind::OperatorBitwiseNot =>
                {
                    // var = ~var
                    return format_binary(
                        *ns.get_opcode_by_command_name(OP_NOT)?,
                        token_str(expr, dest_var_token),
                        token_str(expr, as_token(&unary.operand)?),
                        legacy_ini,
                    );
                }
                Node::Binary(binary_expr) => {
                    let op = |op| {
                        format_ternary(
                            *ns.get_opcode_by_command_name(op)?,
                            token_str(expr, dest_var_token),
                            token_str(expr, as_token(&binary_expr.left)?),
                            token_str(expr, as_token(&binary_expr.right)?),
                            legacy_ini,
                        )
                    };
                    match binary_expr.get_operator() {
                        SyntaxKind::OperatorBitwiseAnd => op(OP_AND),
                        SyntaxKind::OperatorBitwiseOr => op(OP_OR),
                        SyntaxKind::OperatorBitwiseXor => op(OP_XOR),
                        SyntaxKind::OperatorBitwiseMod => op(OP_MOD),
                        SyntaxKind::OperatorBitwiseShr => op(OP_SHR),
                        SyntaxKind::OperatorBitwiseShl => op(OP_SHL),
                        SyntaxKind::OperatorPlus => op(OP_INT_ADD),
                        SyntaxKind::OperatorMinus => op(OP_INT_SUB),
                        SyntaxKind::OperatorMul => op(OP_INT_MUL),
                        SyntaxKind::OperatorDiv => op(OP_INT_DIV),
                        _ => None,
                    }
                }
                _ => {
                    let right_token = as_token(&right)?;

                    let op = |op| {
                        format_binary_no_reorder(
                            *ns.get_opcode_by_command_name(op)?,
                            token_str(expr, dest_var_token),
                            token_str(expr, right_token),
                        )
                    };
                    match e.get_operator() {
                        SyntaxKind::OperatorBitwiseAndEqual => op(OP_AND_COMPOUND),
                        SyntaxKind::OperatorBitwiseOrEqual => op(OP_OR_COMPOUND),
                        SyntaxKind::OperatorBitwiseXorEqual => op(OP_XOR_COMPOUND),
                        SyntaxKind::OperatorBitwiseModEqual => op(OP_MOD_COMPOUND),
                        SyntaxKind::OperatorBitwiseShrEqual => op(OP_SHR_COMPOUND),
                        SyntaxKind::OperatorBitwiseShlEqual => op(OP_SHL_COMPOUND),
                        SyntaxKind::OperatorTimedAdditionEqual => {
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    // var +=@ float
                                    return op(OP_ADD_TIMED_VAL_TO_FLOAT_VAR);
                                }
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    // lvar +=@ float
                                    return op(OP_ADD_TIMED_VAL_TO_FLOAT_LVAR)
                                        .or(op(OP_ADD_TIMED_VAL_TO_FLOAT_VAR)); // vcs
                                }
                                SyntaxKind::GlobalVariable if left_var.is_global() => {
                                    // var +=@ var
                                    return op(OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR);
                                }
                                SyntaxKind::LocalVariable if left_var.is_local() => {
                                    // lvar +=@ lvar
                                    return op(OP_ADD_TIMED_FLOAT_LVAR_TO_FLOAT_LVAR)
                                        .or(op(OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR));
                                    // vcs
                                }
                                SyntaxKind::LocalVariable if left_var.is_global() => {
                                    // var +=@ lvar
                                    return op(OP_ADD_TIMED_FLOAT_LVAR_TO_FLOAT_VAR)
                                        .or(op(OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR));
                                    // vcs
                                }
                                SyntaxKind::GlobalVariable if left_var.is_local() => {
                                    // lvar +=@ var
                                    return op(OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_LVAR)
                                        .or(op(OP_ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR));
                                    // vcs
                                }
                                _ => None,
                            }
                        }
                        SyntaxKind::OperatorTimedSubtractionEqual => {
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    // var -=@ float
                                    return op(OP_SUB_TIMED_VAL_FROM_FLOAT_VAR);
                                }
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    // lvar -=@ float
                                    return op(OP_SUB_TIMED_VAL_FROM_FLOAT_LVAR)
                                        .or(op(OP_SUB_TIMED_VAL_FROM_FLOAT_VAR));
                                }
                                SyntaxKind::GlobalVariable if left_var.is_global() => {
                                    // var -=@ var
                                    return op(OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR);
                                }
                                SyntaxKind::LocalVariable if left_var.is_local() => {
                                    // lvar -=@ lvar
                                    return op(OP_SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_LVAR)
                                        .or(op(OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR));
                                }
                                SyntaxKind::LocalVariable if left_var.is_global() => {
                                    // var -=@ lvar
                                    return op(OP_SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_VAR)
                                        .or(op(OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR));
                                }
                                SyntaxKind::GlobalVariable if left_var.is_local() => {
                                    // lvar -=@ var
                                    return op(OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_LVAR)
                                        .or(op(OP_SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR));
                                }
                                _ => None,
                            }
                        }
                        SyntaxKind::OperatorCastEqual => {
                            // requires type info
                            if !is_variable(right) {
                                return None;
                            }
                            let v1 = token_str(expr, dest_var_token);
                            let v2 = token_str(expr, right_token);
                            let t1 = *var_types.map.get(v1)?;
                            let t2 = *var_types.map.get(v2)?;

                            use crate::utils::compiler_const::*;
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                SyntaxKind::GlobalVariable
                                    if left_var.is_global()
                                        && t1 == TOKEN_INT
                                        && t2 == TOKEN_FLOAT =>
                                {
                                    // var =# var // int = float
                                    return op(OP_CSET_VAR_INT_TO_VAR_FLOAT);
                                }
                                SyntaxKind::GlobalVariable
                                    if left_var.is_global()
                                        && t1 == TOKEN_FLOAT
                                        && t2 == TOKEN_INT =>
                                {
                                    op(OP_CSET_VAR_FLOAT_TO_VAR_INT)
                                }
                                SyntaxKind::LocalVariable
                                    if left_var.is_local()
                                        && t1 == TOKEN_INT
                                        && t2 == TOKEN_FLOAT =>
                                {
                                    // lvar =# lvar // int = float
                                    return op(OP_CSET_LVAR_INT_TO_LVAR_FLOAT)
                                        .or(op(OP_CSET_VAR_INT_TO_VAR_FLOAT)); // vcs
                                }
                                SyntaxKind::LocalVariable
                                    if left_var.is_local()
                                        && t1 == TOKEN_FLOAT
                                        && t2 == TOKEN_INT =>
                                {
                                    // lvar =# lvar // float = int
                                    return op(OP_CSET_LVAR_FLOAT_TO_LVAR_INT)
                                        .or(op(OP_CSET_VAR_FLOAT_TO_VAR_INT)); // vcs
                                }
                                SyntaxKind::GlobalVariable
                                    if left_var.is_local()
                                        && t1 == TOKEN_INT
                                        && t2 == TOKEN_FLOAT =>
                                {
                                    // lvar =# var // int = float
                                    return op(OP_CSET_LVAR_INT_TO_VAR_FLOAT)
                                        .or(op(OP_CSET_VAR_INT_TO_VAR_FLOAT)); // vcs
                                }
                                SyntaxKind::GlobalVariable
                                    if left_var.is_local()
                                        && t1 == TOKEN_FLOAT
                                        && t2 == TOKEN_INT =>
                                {
                                    // lvar =# var // float = int
                                    return op(OP_CSET_LVAR_FLOAT_TO_VAR_INT)
                                        .or(op(OP_CSET_VAR_FLOAT_TO_VAR_INT)); // vcs
                                }
                                SyntaxKind::LocalVariable
                                    if left_var.is_global()
                                        && t1 == TOKEN_INT
                                        && t2 == TOKEN_FLOAT =>
                                {
                                    // var =# lvar // int = float
                                    return op(OP_CSET_VAR_INT_TO_LVAR_FLOAT)
                                        .or(op(OP_CSET_VAR_INT_TO_VAR_FLOAT)); // vcs
                                }
                                SyntaxKind::LocalVariable
                                    if left_var.is_global()
                                        && t1 == TOKEN_FLOAT
                                        && t2 == TOKEN_INT =>
                                {
                                    // var =# lvar // float = int
                                    return op(OP_CSET_VAR_FLOAT_TO_LVAR_INT)
                                        .or(op(OP_CSET_VAR_FLOAT_TO_VAR_INT)); // vcs
                                }

                                _ => None,
                            }
                        }
                        SyntaxKind::OperatorEqual => {
                            let left_var = as_variable(left)?;
                            match right_token.syntax_kind {
                                // var = int
                                SyntaxKind::IntegerLiteral if left_var.is_global() => {
                                    op(OP_SET_VAR_INT)
                                }
                                // var = float
                                SyntaxKind::FloatLiteral if left_var.is_global() => {
                                    op(OP_SET_VAR_FLOAT)
                                }
                                // lvar = int
                                SyntaxKind::IntegerLiteral if left_var.is_local() => {
                                    op(OP_SET_LVAR_INT).or(op(OP_SET_VAR_INT))
                                }
                                // lvar = float
                                SyntaxKind::FloatLiteral if left_var.is_local() => {
                                    op(OP_SET_LVAR_FLOAT).or(op(OP_SET_VAR_FLOAT))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
            }
        }
        _ => None,
    };
}
