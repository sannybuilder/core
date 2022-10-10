use crate::parser::interface::*;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::map;
use nom::multi::many1;
use nom::sequence::{delimited, separated_pair};
use nom::{branch::alt, combinator::consumed};
use nom::{character::complete::multispace0, sequence::terminated};

use crate::parser::expression;
use crate::parser::helpers;
use crate::parser::literal;
use crate::parser::statement;

pub fn declaration(s: Span) -> R<Node> {
    terminated(alt((statement::statement, const_declaration)), multispace0)(s)
}

pub fn const_declaration(s: Span) -> R<Node> {
    map(
        consumed(delimited(
            helpers::line(tag_no_case("const")),
            map(
                many1(helpers::line(consumed(separated_pair(
                    literal::identifier,
                    helpers::ws(tag("=")),
                    expression::expression,
                )))),
                |v: Vec<(Span, (Token, Node))>| {
                    v.into_iter()
                        .map(|(span, (name, value))| ConstInitialization {
                            name,
                            value: Box::new(value),
                            token: Token::from(span, SyntaxKind::ConstInitialization),
                        })
                        .collect()
                },
            ),
            helpers::line(tag_no_case("end")),
        )),
        |(span, items)| {
            Node::ConstDeclaration(ConstDeclaration {
                items,
                token: Token::from(span, SyntaxKind::ConstDeclaration),
            })
        },
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_const_declaration() {
        let (_, node) = const_declaration(Span::from(
            r#"
const
    x = 1
    y = 2.0
end"#,
        ))
        .unwrap();
        assert_eq!(
            node,
            Node::ConstDeclaration(ConstDeclaration {
                items: vec![
                    ConstInitialization {
                        name: Token {
                            start: 5,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        value: Box::new(Node::Literal(Token {
                            start: 9,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        })),
                        token: Token {
                            start: 5,
                            len: 5,
                            syntax_kind: SyntaxKind::ConstInitialization
                        }
                    },
                    ConstInitialization {
                        name: Token {
                            start: 5,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        value: Box::new(Node::Literal(Token {
                            start: 9,
                            len: 3,
                            syntax_kind: SyntaxKind::FloatLiteral
                        })),
                        token: Token {
                            start: 5,
                            len: 7,
                            syntax_kind: SyntaxKind::ConstInitialization
                        }
                    }
                ],
                token: Token {
                    start: 1,
                    len: 32,
                    syntax_kind: SyntaxKind::ConstDeclaration
                }
            })
        )
    }
}
