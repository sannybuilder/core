use nom::IResult;
use nom_locate::LocatedSpan;
#[derive(Debug, PartialEq)]
pub enum SyntaxKind {
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    HexadecimalLiteral,
    StringLiteral,
    ArrayElementSCR,
    IndexedVariable,
    LocalVariable,
    GlobalVariable,
    AdmaVariable,
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
    pub fn get_text<'a>(&self, s: &'a str) -> &'a str {
        let start = self.start - 1;
        let end = start + self.len;
        &s[start..end]
    }
}

#[derive(Debug, PartialEq)]
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
}

impl Node {
    pub fn is_variable(&self) -> bool {
        self.as_variable().is_some()
    }

    pub fn as_variable(&self) -> Option<&Variable> {
        match self {
            Node::Variable(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_token(&self) -> &Token {
        match self {
            Node::Literal(e) => e,
            Node::Unary(e) => &e.get_token(),
            Node::Variable(e) => match e {
                Variable::Local(v) => &v.token,
                Variable::Global(v) => &v.token,
                Variable::ArrayElement(v) => &v.token,
                Variable::Indexed(v) => &v.token,
                Variable::Adma(v) => &v.token,
            },
            Node::Binary(b) => &b.token,
            Node::ConstDeclaration(d) => &d.token,
        }
    }

    pub fn is_literal(&self) -> bool {
        self.as_literal().is_some()
    }

    pub fn as_literal(&self) -> Option<&Token> {
        match self {
            Node::Literal(e) => Some(e),
            Node::Unary(e) => e.get_operand().as_literal(),
            _ => None,
        }
    }
    pub fn get_text<'a>(&self, s: &'a str) -> &'a str {
        self.as_token().get_text(s)
    }
}

#[derive(Debug, PartialEq)]
pub enum Variable {
    Global(SingleVariable),
    Local(SingleVariable),
    Indexed(IndexedVariable),
    ArrayElement(ArrayElementSCR),
    Adma(SingleVariable),
}

impl Variable {
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
    pub fn is_integer(&self) -> bool {
        match self {
            Variable::Global(v) => v.is_integer(),
            Variable::Local(v) => v.is_integer(),
            Variable::Indexed(v) => v.var.is_integer(),
            Variable::ArrayElement(v) => v._type == VariableType::Int,
            Variable::Adma(v) => v.is_integer(),
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            Variable::Global(v) => v.is_float(),
            Variable::Local(v) => v.is_float(),
            Variable::Indexed(v) => v.var.is_float(),
            Variable::ArrayElement(v) => v._type == VariableType::Float,
            Variable::Adma(v) => v.is_float(),
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            Variable::Global(v) => v.is_string(),
            Variable::Local(v) => v.is_string(),
            Variable::Indexed(v) => v.var.is_string(),
            Variable::ArrayElement(v) => {
                v._type == VariableType::ShortString || v._type == VariableType::LongString
            }
            Variable::Adma(v) => v.is_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UnaryPrefixExpr {
    operator: Token,
    operand: Box<Node>,
    token: Token,
}

impl UnaryPrefixExpr {
    pub fn new(operator: Token, operand: Box<Node>, token: Token) -> Self {
        Self {
            operator,
            operand,
            token,
        }
    }
    pub fn get_operator(&self) -> &SyntaxKind {
        &self.operator.syntax_kind
    }
    pub fn get_operand(&self) -> &Node {
        &self.operand
    }
    pub fn get_token(&self) -> &Token {
        &self.token
    }
}

#[derive(Debug, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub operator: Token,
    pub right: Box<Node>,
    pub token: Token,
}

impl BinaryExpr {
    pub fn get_operator(&self) -> &SyntaxKind {
        &self.operator.syntax_kind
    }
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
    /// identifier portion of the variable (10, var)
    pub name: Token,
    /// variable token including the identifier and optional type (10@s, v$var)
    pub token: Token,
    /// variable type (i,f,s,v, or unknown)
    pub _type: VariableType,
}

impl SingleVariable {
    pub fn is_integer(&self) -> bool {
        self._type == VariableType::Int
    }
    pub fn is_float(&self) -> bool {
        self._type == VariableType::Float
    }
    pub fn is_string(&self) -> bool {
        self._type == VariableType::ShortString || self._type == VariableType::LongString
    }
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
