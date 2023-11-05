use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::multi::many1;

pub mod interface;
use interface::*;

mod binary;
mod declaration;
mod expression;
mod helpers;
mod literal;
mod operator;
mod statement;
mod unary;
mod variable;

pub fn parse(s: &str) -> R<AST> {
    all_consuming(map(many1(declaration::declaration), |body| AST { body }))(Span::from(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    #[test]
    fn test_with_ws0() {
        let (_, ast) = parse("0@ =/*123*/256").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 1,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            syntax_kind: SyntaxKind::LocalVariable,
                            start: 1,
                            len: 2,
                        }
                    }))),
                    operator: Token {
                        syntax_kind: SyntaxKind::OperatorEqual,
                        start: 4,
                        len: 1
                    },
                    right: Box::new(Node::Literal(Token {
                        start: 12,
                        len: 3,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    })),
                    token: Token {
                        start: 1,
                        len: 14,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );
    }

    #[test]
    fn test_with_ws1() {
        let (_, ast) = parse("1{123}").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Literal(Token {
                    start: 1,
                    len: 1,
                    syntax_kind: SyntaxKind::IntegerLiteral
                })]
            }
        );
    }

    #[test]
    fn test_with_ws2() {
        // does not support directives yet
        assert!(parse("1{$123}").is_err());
    }

    #[test]
    fn test_hex() {
        let (_, ast) = parse("0x100").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Literal(Token {
                    start: 1,
                    len: 5,
                    syntax_kind: SyntaxKind::IntegerLiteral
                })]
            }
        );
    }

    #[test]
    fn test_label() {
        let (_, ast) = parse("@label123").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Literal(Token {
                    start: 1,
                    len: 9,
                    syntax_kind: SyntaxKind::LabelLiteral
                })]
            }
        );
    }

    #[test]
    fn test_binary() {
        let (_, ast) = parse("0b101").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Literal(Token {
                    start: 1,
                    len: 5,
                    syntax_kind: SyntaxKind::IntegerLiteral
                })]
            }
        );
    }

}
