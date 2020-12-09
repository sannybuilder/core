use nom::IResult;
use nom_locate::LocatedSpan;
#[derive(Debug, PartialEq)]
pub enum SyntaxKind {
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    ArrayElementSCR,
    IndexedVariable,
    LocalVariable,
    GlobalVariable,
    UnaryPrefixExpr,
    BinaryExpr,
    ConstDeclaration,
    ConstInitialization,

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
    Variable(Variable),
    Binary(BinaryExpr),
    Unary(UnaryPrefixExpr),
    ConstDeclaration(ConstDeclaration),
}

#[derive(Debug, PartialEq)]
pub enum Variable {
    Global(SingleVariable),
    Local(SingleVariable),
    Indexed(IndexedVariable),
    ArrayElement(ArrayElementSCR),
    // ADMA
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
pub struct ArrayElementSCR {
    pub array_var: Box<Variable>,
    pub index_var: Box<Variable>,
    pub _type: VariableType,
    pub len: Token,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct IndexedVariable {
    pub var: Box<Variable>,
    pub index: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct SingleVariable {
    pub name: Token,
    pub token: Token,
    pub _type: VariableType,
}

#[derive(Debug, PartialEq)]
pub struct ConstDeclaration {
    pub items: Vec<ConstInitialization>,
    pub token: Token,
}
#[derive(Debug, PartialEq)]
pub struct ConstInitialization {
    pub name: Token,
    pub value: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub enum VariableType {
    Unknown,
    Int,         // i
    Float,       // f
    ShortString, // s
    LongString,  // v
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub body: Vec<Node>,
}

pub type Span<'a> = LocatedSpan<&'a str>;
pub type R<'a, T> = IResult<Span<'a>, T>;
