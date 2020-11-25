#![allow(dead_code)]
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::character::complete::alphanumeric1;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::multispace0;
use nom::character::complete::one_of;
use nom::combinator::all_consuming;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::{sequence::tuple, IResult};
use nom_locate::LocatedSpan;
use nom_recursive::recursive_parser;
use nom_recursive::RecursiveInfo;

static LVAR_CHAR: char = '@';
static GVAR_CHAR: char = '$';

pub trait HasRecursiveInfo {
    fn get_recursive_info(&self) -> RecursiveInfo;
    fn set_recursive_info(self, info: RecursiveInfo) -> Self;
}

impl HasRecursiveInfo for RecursiveInfo {
    fn get_recursive_info(&self) -> RecursiveInfo {
        *self
    }

    fn set_recursive_info(self, info: RecursiveInfo) -> Self {
        info
    }
}

impl<T, U> HasRecursiveInfo for nom_locate::LocatedSpan<T, U>
where
    U: HasRecursiveInfo,
{
    fn get_recursive_info(&self) -> RecursiveInfo {
        self.extra.get_recursive_info()
    }

    fn set_recursive_info(mut self, info: RecursiveInfo) -> Self {
        self.extra = self.extra.set_recursive_info(info);
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum SyntaxKind {
    IntegerLiteral,
    FloatLiteral,
    Array,
    LocalVariable,
    GlobalVariable,
    UnaryPrefixExpr,
    BinaryExpr,

    OperatorNot,        // ~
    OperatorAnd,        // &
    OperatorOr,         // |
    OperatorXor,        // ^
    OperatorMod,        // %
    OperatorShr,        // >>
    OperatorShl,        // <<
    OperatorPlus,       // +
    OperatorMinus,      // -
    OperatorMul,        // *
    OperatorDiv,        // /
    OperatorEqual,      // =
    OperatorEqualEqual, // ==
    OperatorNotEqual,   // ~=
    OperatorAndEqual,   // &=
    OperatorOrEqual,    // |=
    OperatorXorEqual,   // ^=
    OperatorModEqual,   // %=
    OperatorShrEqual,   // >>=
    OperatorShlEqual,   // <<=
    OperatorPlusEqual,  // +=
    OperatorMinusEqual, // -=
    OperatorMulEqual,   // *=
    OperatorDivEqual,   // /=
}

#[derive(Debug, PartialEq)]
pub struct Token {
    // pub text: String,
    pub syntax_kind: SyntaxKind,
    pub start: usize,
    pub len: usize,
}

impl Token {
    fn from(s: Span, syntax_kind: SyntaxKind) -> Token {
        Self {
            start: s.get_column(),
            len: s.len(),
            // text: String::from(*s.fragment()),
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
pub struct UnaryPrefixExpr {
    pub operator: Token,
    pub operand: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub operator: Token,
    pub right: Box<Node>,
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub node: Node,
}

type Span<'a> = LocatedSpan<&'a str, RecursiveInfo>;
type R<'a, T> = IResult<Span<'a>, T>;

pub fn parse(s: &str) -> R<AST> {
    all_consuming(map(node, |node| AST { node }))(Span::from(s))
}

fn node(s: Span) -> R<Node> {
    alt((
        map(binary_expr, |e| Node::Binary(e)),
        map(unary_expr, |e| Node::Unary(e)),
        map(variable, |e| Node::Token(e)),
        map(signed_number, |e| Node::Token(e)),
    ))(s)
}

fn unary_expr(s: Span) -> R<UnaryPrefixExpr> {
    map(
        delimited(multispace0, consumed(tuple((operator, node))), multispace0),
        |(s, (operator, node))| UnaryPrefixExpr {
            operator,
            operand: Box::new(node),
            token: Token::from(s, SyntaxKind::UnaryPrefixExpr),
        },
    )(s)
}

#[recursive_parser]
fn binary_expr(s: Span) -> R<BinaryExpr> {
    map(
        consumed(tuple((
            delimited(multispace0, node, multispace0),
            delimited(multispace0, operator, multispace0),
            delimited(multispace0, node, multispace0),
        ))),
        |(s, (left, operator, right))| BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            token: Token::from(s, SyntaxKind::BinaryExpr),
        },
    )(s)
}

fn operator(s: Span) -> R<Token> {
    alt((
        alt((
            op_equal_equal,
            op_plus_equal,
            op_minus_equal,
            op_mul_equal,
            op_div_equal,
            op_not_equal,
            op_and_equal,
            op_or_equal,
            op_xor_equal,
            op_mod_equal,
            op_shr_equal,
            op_shl_equal,
        )),
        alt((
            op_not, op_and, op_or, op_xor, op_mod, op_shr, op_shl, op_plus, op_minus, op_mul,
            op_div, op_equal,
        )),
    ))(s)
}

fn op_not(s: Span) -> R<Token> {
    map(tag("~"), |s: Span| Token::from(s, SyntaxKind::OperatorNot))(s)
}

fn op_and(s: Span) -> R<Token> {
    map(tag("&"), |s: Span| Token::from(s, SyntaxKind::OperatorAnd))(s)
}

fn op_or(s: Span) -> R<Token> {
    map(tag("|"), |s: Span| Token::from(s, SyntaxKind::OperatorOr))(s)
}

fn op_xor(s: Span) -> R<Token> {
    map(tag("^"), |s: Span| Token::from(s, SyntaxKind::OperatorXor))(s)
}

fn op_mod(s: Span) -> R<Token> {
    map(tag("%"), |s: Span| Token::from(s, SyntaxKind::OperatorMod))(s)
}

fn op_shr(s: Span) -> R<Token> {
    map(tag(">>"), |s: Span| Token::from(s, SyntaxKind::OperatorShr))(s)
}

fn op_shl(s: Span) -> R<Token> {
    map(tag("<<"), |s: Span| Token::from(s, SyntaxKind::OperatorShl))(s)
}

fn op_plus(s: Span) -> R<Token> {
    map(tag("+"), |s: Span| Token::from(s, SyntaxKind::OperatorPlus))(s)
}

fn op_minus(s: Span) -> R<Token> {
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

fn op_not_equal(s: Span) -> R<Token> {
    map(tag("~="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorNotEqual)
    })(s)
}

fn op_and_equal(s: Span) -> R<Token> {
    map(tag("&="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorAndEqual)
    })(s)
}

fn op_or_equal(s: Span) -> R<Token> {
    map(tag("|="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorOrEqual)
    })(s)
}

fn op_xor_equal(s: Span) -> R<Token> {
    map(tag("^="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorXorEqual)
    })(s)
}

fn op_mod_equal(s: Span) -> R<Token> {
    map(tag("%="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorModEqual)
    })(s)
}

fn op_shr_equal(s: Span) -> R<Token> {
    map(tag(">>="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorShrEqual)
    })(s)
}

fn op_shl_equal(s: Span) -> R<Token> {
    map(tag("<<="), |s: Span| {
        Token::from(s, SyntaxKind::OperatorShlEqual)
    })(s)
}

fn signed_int(s: Span) -> R<Token> {
    map(
        recognize(pair(opt(one_of("-+")), decimal_span)),
        |s: Span| Token::from(s, SyntaxKind::IntegerLiteral),
    )(s)
}

fn signed_float(s: Span) -> R<Token> {
    map(recognize(pair(opt(one_of("-+")), float_span)), |s: Span| {
        Token::from(s, SyntaxKind::FloatLiteral)
    })(s)
}

fn signed_number(s: Span) -> R<Token> {
    alt((signed_float, signed_int))(s)
}

fn array(s: Span) -> R<Token> {
    alt((array_typed, array_indexed))(s)
}

fn array_typed(s: Span) -> R<Token> {
    map(
        recognize(tuple((
            variable_span,
            delimited(
                char('('),
                tuple((variable_span, char(','), digit1, opt(one_of("ifvs")))),
                char(')'),
            ),
        ))),
        |s: Span| Token::from(s, SyntaxKind::Array),
    )(s)
}

fn array_indexed(s: Span) -> R<Token> {
    map(
        recognize(tuple((
            variable_span,
            delimited(char('['), alt((variable_span, decimal_span)), char(']')),
        ))),
        |s: Span| Token::from(s, SyntaxKind::Array),
    )(s)
}

fn variable(s: Span) -> R<Token> {
    alt((array, local_var, global_var))(s)
}

fn local_var(s: Span) -> R<Token> {
    map(local_var_span, |s: Span| {
        Token::from(s, SyntaxKind::LocalVariable)
    })(s)
}

fn global_var(s: Span) -> R<Token> {
    map(global_var_span, |s: Span| {
        Token::from(s, SyntaxKind::GlobalVariable)
    })(s)
}

// combination of letters, digits and underscore, not starting with a digit
fn identifier(s: Span) -> R<String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: Span| String::from(*s.fragment()),
    )(s)
}

// any combination of letters, digits and underscore
fn identifier_any(s: Span) -> R<String> {
    map(
        recognize(many1(alt((alphanumeric1, tag("_"))))),
        |s: Span| String::from(*s.fragment()),
    )(s)
}

fn decimal_span(s: Span) -> R<Span> {
    recognize(digit1)(s)
}

fn float_span(s: Span) -> R<Span> {
    alt((
        // Case one: .42
        recognize(tuple((
            char('.'),
            decimal_span,
            opt(tuple((one_of("eE"), opt(one_of("+-")), decimal_span))),
        ))), // Case two: 42e42 and 42.42e42
        recognize(tuple((
            decimal_span,
            opt(preceded(char('.'), decimal_span)),
            one_of("eE"),
            opt(one_of("+-")),
            decimal_span,
        ))), // Case three: 42. and 42.42
        recognize(tuple((decimal_span, char('.'), opt(decimal_span)))),
    ))(s)
}

fn variable_span(s: Span) -> R<Span> {
    alt((local_var_span, global_var_span))(s)
}

fn local_var_span(s: Span) -> R<Span> {
    recognize(terminated(digit1, char(LVAR_CHAR)))(s)
}

fn global_var_span(s: Span) -> R<Span> {
    recognize(preceded(char(GVAR_CHAR), identifier_any))(s)
}

#[test]
fn test2() {
    let (_, ast) = parse("  ~1@  ").unwrap();

    assert_eq!(
        ast,
        AST {
            node: Node::Unary(UnaryPrefixExpr {
                operator: Token {
                    start: 3,
                    len: 1,
                    syntax_kind: SyntaxKind::OperatorNot,
                    // text: String::from("~")
                },
                operand: Box::new(Node::Token(Token {
                    start: 4,
                    len: 2,
                    // text: String::from("1@"),
                    syntax_kind: SyntaxKind::LocalVariable,
                })),
                token: Token {
                    // text: String::from("~1@"),
                    start: 3,
                    len: 3,
                    syntax_kind: SyntaxKind::UnaryPrefixExpr,
                },
            })
        }
    )
}

#[test]
fn test3() {
    let (_, ast) = parse("0@ += $_t_e_s_t").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Binary(BinaryExpr {
                left: Box::new(Node::Token(Token {
                    start: 1,
                    len: 2,
                    // text: String::from("0@"),
                    syntax_kind: SyntaxKind::LocalVariable,
                })),
                operator: Token {
                    start: 4,
                    len: 2,
                    // text: String::from("+="),
                    syntax_kind: SyntaxKind::OperatorPlusEqual
                },
                right: Box::new(Node::Token(Token {
                    start: 7,
                    len: 9,
                    // text: String::from("$_t_e_s_t"),
                    syntax_kind: SyntaxKind::GlobalVariable
                })),
                token: Token {
                    start: 1,
                    len: 15,
                    // text: String::from("0@ += $_t_e_s_t"),
                    syntax_kind: SyntaxKind::BinaryExpr
                }
            })
        }
    )
}

#[test]
fn test4() {
    assert!(parse("0@ += 1").is_ok());

    assert!(parse("0@ += 1.0").is_ok());

    assert!(parse("0@ += -1").is_ok());

    assert!(parse("0@ += -100.12").is_ok());
}

#[test]
fn test5() {
    println!("{:#?}", parse("0@ = ~1@"));
    assert!(parse("0@ = $a(0@,1i)").is_ok());
}
