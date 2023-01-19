use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::multi::many1;

pub mod interface;
use interface::*;
use whitespace::mws;

mod binary;
mod declaration;
mod expression;
mod literal;
mod operator;
mod statement;
mod string;
mod unary;
mod variable;
mod whitespace;

pub fn parse(s: &str) -> R<AST> {
    all_consuming(map(many1(mws(declaration::declaration)), |body| AST {
        body,
    }))(Span::from(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    #[test]
    fn test_with_ws0() {
        let (_, ast) = parse("0@ = /*123*/ 256").unwrap();
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
                    right: Box::new(Node::Literal(Literal::Int(IntLiteral {
                        value: 256,
                        token: Token {
                            start: 14,
                            len: 3,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 16,
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
                body: vec![Node::Literal(Literal::Int(IntLiteral {
                    value: 1,
                    token: Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    }
                })),]
            }
        );
    }

    #[test]
    fn test_with_ws2() {
        // does not support directives yet
        assert!(parse("1{$123}").is_err());
    }

    #[test]
    fn test_with_ws3() {
        let (_, ast) = parse("/**//* */ 5").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Literal(Literal::Int(IntLiteral {
                    value: 5,
                    token: Token {
                        start: 11,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    }
                })),]
            }
        );
    }
}
