use nom::branch::alt;
use nom::character::complete::alpha1;
use nom::character::complete::alphanumeric1;
use nom::character::complete::char;
use nom::character::complete::digit1;
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
use nom::{bytes::complete::tag, character::complete::space0};
use nom::{sequence::tuple, IResult};
use nom_locate::LocatedSpan;

static LVAR_CHAR: char = '@';
static GVAR_CHAR: char = '$';

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

type Span<'a> = LocatedSpan<&'a str>;
type R<'a, T> = IResult<Span<'a>, T>;

pub fn parse(s: &str) -> R<AST> {
    all_consuming(map(expression, |node| AST { node }))(Span::from(s))
}

fn expression(s: Span) -> R<Node> {
    ws(assignment)(s)
}

fn map_binary(span: Span, left: Node, op: Option<(Token, Node)>) -> Node {
    match op {
        Some((operator, right)) => Node::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            token: Token::from(span, SyntaxKind::BinaryExpr),
        }),
        _ => left,
    }
}

fn assignment(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            equality,
            opt(tuple((ws(assignment_operator), equality))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn equality(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            bitwise,
            opt(tuple((ws(alt((op_equal_equal, op_less_greater))), bitwise))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn bitwise(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            comparison,
            opt(tuple((ws(bitwise_operator), comparison))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn comparison(s: Span) -> R<Node> {
    map(
        consumed(tuple((term, opt(tuple((ws(comparison_operator), term)))))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn term(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            factor,
            opt(tuple((ws(alt((op_plus, op_minus))), factor))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn factor(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            unary,
            opt(tuple((ws(alt((op_mul, op_div))), unary))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn unary(s: Span) -> R<Node> {
    alt((
        map(
            consumed(tuple((alt((op_minus, op_bitwise_not)), unary))),
            |(span, (operator, right))| {
                Node::Unary(UnaryPrefixExpr {
                    operator,
                    operand: Box::new(right),
                    token: Token::from(span, SyntaxKind::UnaryPrefixExpr),
                })
            },
        ),
        map(alt((variable, number)), |token| Node::Token(token)),
    ))(s)
}

fn assignment_operator(s: Span) -> R<Token> {
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

fn bitwise_operator(s: Span) -> R<Token> {
    alt((
        op_bitwise_and,
        op_bitwise_or,
        op_bitwise_xor,
        op_bitwise_mod,
        op_bitwise_shr,
        op_bitwise_shl,
    ))(s)
}

fn comparison_operator(s: Span) -> R<Token> {
    alt((op_greater_equal, op_greater, op_less, op_less_equal))(s)
}

fn op_bitwise_not(s: Span) -> R<Token> {
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

fn number(s: Span) -> R<Token> {
    alt((
        map(float_span, |s| Token::from(s, SyntaxKind::FloatLiteral)),
        map(decimal_span, |s| Token::from(s, SyntaxKind::IntegerLiteral)),
    ))(s)
}

fn array(s: Span) -> R<Token> {
    alt((array_typed, array_indexed))(s)
}

fn variable_type_char(s: Span) -> R<Option<char>> {
    opt(one_of("sv"))(s)
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
fn identifier(s: Span) -> R<Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(s)
}

// any combination of letters, digits and underscore
fn identifier_any_span(s: Span) -> R<Span> {
    recognize(many1(alt((alphanumeric1, tag("_")))))(s)
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
    recognize(terminated(
        digit1,
        tuple((char(LVAR_CHAR), variable_type_char)),
    ))(s)
}

fn global_var_span(s: Span) -> R<Span> {
    recognize(preceded(
        tuple((variable_type_char, char(GVAR_CHAR))),
        identifier_any_span,
    ))(s)
}

// whitespace wrapper
fn ws<'a, F: 'a, O, E: nom::error::ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(space0, inner, space0)
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
                    syntax_kind: SyntaxKind::OperatorBitwiseNot,
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
fn variables() {
    let (_, ast) = parse("0@").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::LocalVariable,
                start: 1,
                len: 2,
            })
        }
    );
    let (_, ast) = parse("0@s").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::LocalVariable,
                start: 1,
                len: 3,
            })
        }
    );
    let (_, ast) = parse("0@v").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::LocalVariable,
                start: 1,
                len: 3,
            })
        }
    );
    let (_, ast) = parse("$var").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::GlobalVariable,
                start: 1,
                len: 4,
            })
        }
    );
    let (_, ast) = parse("s$var").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::GlobalVariable,
                start: 1,
                len: 5,
            })
        }
    );
    let (_, ast) = parse("v$var").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::GlobalVariable,
                start: 1,
                len: 5,
            })
        }
    );
    let (_, ast) = parse("$1").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::GlobalVariable,
                start: 1,
                len: 2,
            })
        }
    );
    let (_, ast) = parse("s$1").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::GlobalVariable,
                start: 1,
                len: 3,
            })
        }
    );
    let (_, ast) = parse("v$1").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::GlobalVariable,
                start: 1,
                len: 3,
            })
        }
    );

    let (_, ast) = parse("$var($index,10i)").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::Array,
                start: 1,
                len: 16,
            })
        }
    );

    let (_, ast) = parse("$1($2,10f)").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::Array,
                start: 1,
                len: 10,
            })
        }
    );
    let (_, ast) = parse("$1(11@,10s)").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::Array,
                start: 1,
                len: 11,
            })
        }
    );
    let (_, ast) = parse("$var[1]").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::Array,
                start: 1,
                len: 7,
            })
        }
    );
    let (_, ast) = parse("$var[0@]").unwrap();
    assert_eq!(
        ast,
        AST {
            node: Node::Token(Token {
                syntax_kind: SyntaxKind::Array,
                start: 1,
                len: 8,
            })
        }
    );
}
