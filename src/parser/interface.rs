use nom::IResult;
use nom_locate::LocatedSpan;
#[derive(Debug, PartialEq)]
pub enum SyntaxKind {
    IntegerLiteral,
    FloatLiteral,
    Array,
    LocalVariable,
    GlobalVariable,
    UnaryPrefixExpr,
    BinaryExpr,

    OperatorBitwiseNot,      // ~
    OperatorBitwiseAnd,      // &
    OperatorBitwiseOr,       // |
    OperatorBitwiseXor,      // ^
    OperatorBitwiseMod,      // %
    OperatorBitwiseShr,      // >>
    OperatorBitwiseShl,      // <<
    OperatorPlus,            // +
    OperatorMinus,           // -
    OperatorMul,             // *
    OperatorDiv,             // /
    OperatorEqual,           // =
    OperatorEqualEqual,      // ==
    OperatorLessGreater,     // <>
    OperatorBitwiseNotEqual, // ~=
    OperatorBitwiseAndEqual, // &=
    OperatorBitwiseOrEqual,  // |=
    OperatorBitwiseXorEqual, // ^=
    OperatorBitwiseModEqual, // %=
    OperatorBitwiseShrEqual, // >>=
    OperatorBitwiseShlEqual, // <<=
    OperatorPlusEqual,       // +=
    OperatorMinusEqual,      // -=
    OperatorMulEqual,        // *=
    OperatorDivEqual,        // /=
    OperatorGreater,         // >
    OperatorGreaterEqual,    // >=
    OperatorLess,            // <
    OperatorLessEqual,       // <=
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub syntax_kind: SyntaxKind,
    pub start: usize,
    pub len: usize,
}

impl Token {
    pub fn from(s: Span, syntax_kind: SyntaxKind) -> Token {
        Self {
            start: s.get_column(),
            len: s.len(),
            syntax_kind,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Token(Token),
    Binary(BinaryExpr),
    Unary(UnaryPrefixExpr),
}

#[derive(Debug, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub operator: Token,
    pub right: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct UnaryPrefixExpr {
    pub operator: Token,
    pub operand: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub node: Node,
}

pub type Span<'a> = LocatedSpan<&'a str>;
pub type R<'a, T> = IResult<Span<'a>, T>;
