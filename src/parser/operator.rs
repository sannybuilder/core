use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::terminated;

use crate::parser::interface::*;

pub fn assignment(s: Span) -> R<Token> {
    alt((
        op_plus_equal,
        op_minus_equal,
        op_mul_equal,
        op_div_equal,
        op_bitwise_and_equal,
        op_bitwise_or_equal,
        op_bitwise_xor_equal,
        op_bitwise_mod_equal,
        op_bitwise_shr_equal,
        op_bitwise_shl_equal,
        op_bitwise_not_equal,
        op_equal,
    ))(s)
}

pub fn bitwise(s: Span) -> R<Token> {
    alt((
        op_bitwise_and,
        op_bitwise_or,
        op_bitwise_xor,
        op_bitwise_mod,
        op_bitwise_shr,
        op_bitwise_shl,
    ))(s)
}


pub fn equality(s: Span) -> R<Token> {
    alt((op_equal_equal, op_less_greater))(s)
}

pub fn comparison(s: Span) -> R<Token> {
    alt((op_greater_equal, op_less_equal, op_greater, op_less))(s)
}

pub fn add_sub(s: Span) -> R<Token> {
    alt((op_plus, op_minus))(s)
}

pub fn mul_div(s: Span) -> R<Token> {
    alt((op_mul, op_div))(s)
}

pub fn op_bitwise_not(s: Span) -> R<Token> {
    map(tag("~"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseNot)
    })(s)
}

fn op_bitwise_and(s: Span) -> R<Token> {
    map(tag("&"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseAnd)
    })(s)
}

fn op_bitwise_or(s: Span) -> R<Token> {
    map(tag("|"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseOr)
    })(s)
}

fn op_bitwise_xor(s: Span) -> R<Token> {
    map(tag("^"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseXor)
    })(s)
}

fn op_bitwise_mod(s: Span) -> R<Token> {
    map(tag("%"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseMod)
    })(s)
}

fn op_bitwise_shr(s: Span) -> R<Token> {
    map(tag(">>"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseShr)
    })(s)
}

fn op_bitwise_shl(s: Span) -> R<Token> {
    map(tag("<<"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseShl)
    })(s)
}

fn op_plus(s: Span) -> R<Token> {
    map(tag("+"), |s: Span| Token::from(s, SyntaxKind::OperatorPlus))(s)
}

pub fn op_minus(s: Span) -> R<Token> {
    map(tag("-"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorMinus)
    })(s)
}

fn op_mul(s: Span) -> R<Token> {
    map(tag("*"), |s: Span| Token::from(s, SyntaxKind::OperatorMul))(s)
}

fn op_div(s: Span) -> R<Token> {
    map(tag("/"), |s: Span| Token::from(s, SyntaxKind::OperatorDiv))(s)
}

fn op_greater(s: Span) -> R<Token> {
    map(tag(">"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorGreater)
    })(s)
}

fn op_greater_equal(s: Span) -> R<Token> {
    map(tag(">="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorGreaterEqual)
    })(s)
}

fn op_less(s: Span) -> R<Token> {
    map(tag("<"), |s: Span| Token::from(s, SyntaxKind::OperatorLess))(s)
}

fn op_less_equal(s: Span) -> R<Token> {
    map(tag("<="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorLessEqual)
    })(s)
}

fn op_less_greater(s: Span) -> R<Token> {
    map(tag("<>"), |s: Span| {
        Token::from(s, SyntaxKind::OperatorLessGreater)
    })(s)
}

fn op_equal(s: Span) -> R<Token> {
    map(tag("="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorEqual)
    })(s)
}

fn op_equal_equal(s: Span) -> R<Token> {
    map(tag("=="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorEqualEqual)
    })(s)
}

fn op_plus_equal(s: Span) -> R<Token> {
    map(tag("+="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorPlusEqual)
    })(s)
}

fn op_minus_equal(s: Span) -> R<Token> {
    map(tag("-="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorMinusEqual)
    })(s)
}

fn op_mul_equal(s: Span) -> R<Token> {
    map(tag("*="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorMulEqual)
    })(s)
}

fn op_div_equal(s: Span) -> R<Token> {
    map(tag("/="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorDivEqual)
    })(s)
}

fn op_bitwise_not_equal(s: Span) -> R<Token> {
    map(tag("~="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseNotEqual)
    })(s)
}

fn op_bitwise_and_equal(s: Span) -> R<Token> {
    map(tag("&="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseAndEqual)
    })(s)
}

fn op_bitwise_or_equal(s: Span) -> R<Token> {
    map(tag("|="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseOrEqual)
    })(s)
}

fn op_bitwise_xor_equal(s: Span) -> R<Token> {
    map(tag("^="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseXorEqual)
    })(s)
}

fn op_bitwise_mod_equal(s: Span) -> R<Token> {
    map(tag("%="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseModEqual)
    })(s)
}

fn op_bitwise_shr_equal(s: Span) -> R<Token> {
    map(tag(">>="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseShrEqual)
    })(s)
}

fn op_bitwise_shl_equal(s: Span) -> R<Token> {
    map(tag("<<="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorBitwiseShlEqual)
    })(s)
}

pub fn op_not(s: Span) -> R<Token> {
    map(terminated(tag("not"), multispace1), |s: Span| {
        Token::from(s, SyntaxKind::OperatorNot)
    })(s)
}
