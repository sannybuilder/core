use nom::IResult;
use nom_locate::LocatedSpan;
#[derive(Debug, PartialEq, Clone)]
pub enum SyntaxKind {
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    LabelLiteral,
    ArrayElementSCR,
    IndexedVariable,
    LocalVariable,
    GlobalVariable,
    AdmaVariable,
    UnaryPrefixExpr,
    BinaryExpr,
    ConstDeclaration,
    ConstInitialization,
    FunctionSignature,

    OperatorBitwiseNot,            // ~
    OperatorBitwiseAnd,            // &
    OperatorBitwiseOr,             // |
    OperatorBitwiseXor,            // ^
    OperatorBitwiseMod,            // %
    OperatorBitwiseShr,            // >>
    OperatorBitwiseShl,            // <<
    OperatorPlus,                  // +
    OperatorMinus,                 // -
    OperatorMul,                   // *
    OperatorDiv,                   // /
    OperatorEqual,                 // =
    OperatorEqualEqual,            // ==
    OperatorLessGreater,           // <>
    OperatorBitwiseNotEqual,       // ~=
    OperatorBitwiseAndEqual,       // &=
    OperatorBitwiseOrEqual,        // |=
    OperatorBitwiseXorEqual,       // ^=
    OperatorBitwiseModEqual,       // %=
    OperatorBitwiseShrEqual,       // >>=
    OperatorBitwiseShlEqual,       // <<=
    OperatorPlusEqual,             // +=
    OperatorMinusEqual,            // -=
    OperatorMulEqual,              // *=
    OperatorDivEqual,              // /=
    OperatorGreater,               // >
    OperatorGreaterEqual,          // >=
    OperatorLess,                  // <
    OperatorLessEqual,             // <=
    OperatorTimedAdditionEqual,    // +=@
    OperatorTimedSubtractionEqual, // -=@
    OperatorCastEqual,             // =#
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    /// Integer or Float literal
    Literal(Token),
    /// Global or local variable or array element
    Variable(Variable),
    /// Binary expression
    Binary(BinaryExpr),
    /// Unary expression, e.g. `~var`
    Unary(UnaryPrefixExpr),
    ConstDeclaration(ConstDeclaration),
    FunctionSignature(FunctionSignature),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    Global(SingleVariable),
    Local(SingleVariable),
    Indexed(IndexedVariable),
    ArrayElement(ArrayElementSCR),
    Adma(SingleVariable),
}

impl Variable {
    pub fn get_var_name(&self) -> &Token {
        match self {
            Variable::Indexed(v) => &v.var.get_var_name(),
            Variable::ArrayElement(v) => &v.array_var.get_var_name(),
            Variable::Local(v) | Variable::Global(v) | Variable::Adma(v) => &v.token,
        }
    }

    pub fn is_global(&self) -> bool {
        match self {
            Variable::Global(_) | Variable::Adma(_) => true,
            Variable::Indexed(v) if v.var.is_global() => true,
            Variable::ArrayElement(v) if v.array_var.is_global() => true,
            _ => false,
        }
    }

    pub fn is_local(&self) -> bool {
        match self {
            Variable::Local(_) => true,
            Variable::Indexed(v) if v.var.is_local() => true,
            Variable::ArrayElement(v) if v.array_var.is_local() => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub operator: Token,
    pub right: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryPrefixExpr {
    pub operator: Token,
    pub operand: Box<Node>,
    pub token: Token,
}

impl UnaryPrefixExpr {
    pub fn get_operator(&self) -> &SyntaxKind {
        &self.operator.syntax_kind
    }
}

impl BinaryExpr {
    pub fn get_operator(&self) -> &SyntaxKind {
        &self.operator.syntax_kind
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayElementSCR {
    pub array_var: Box<Variable>,
    pub index_var: Box<Variable>,
    pub _type: VariableType,
    pub len: Token,
    pub token: Token,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IndexedVariable {
    pub var: Box<Variable>,
    pub index: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SingleVariable {
    /// identifier portion of the variable (10, var)
    pub name: Token,
    /// variable token including the identifier and optional type (10@s, v$var)
    pub token: Token,
    /// variable type (i,f,s,v, or unknown)
    pub _type: VariableType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConstDeclaration {
    pub items: Vec<ConstInitialization>,
    pub token: Token,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ConstInitialization {
    pub name: Token,
    pub value: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature {
    pub name: Token,
    pub parameters: Vec<FunctionParameter>,
    pub return_types: Vec<FunctionReturnType>,
    pub cc: FunctionCC,
    pub address: Option<Token>,
    pub token: Token,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionCC {
    Local, 
    Cdecl,
    Stdcall,
    Thiscall,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParameter {
    pub name: Option<Token>,
    pub _type: Token,
    pub token: Token,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionReturnType {
    pub _type: Token,
    pub token: Token,
}

pub type Span<'a> = LocatedSpan<&'a str>;
pub type R<'a, T> = IResult<Span<'a>, T>;
