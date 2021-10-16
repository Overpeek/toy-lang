use std::fmt::Display;

use crate::artefact::{
    ast::{BinaryOpNode, Node, NumberNode},
    tokens::{Delimiter, ErrorSpan, Lit, Operator, Side, Span, SpannedToken, Token, Tokens},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnexpectedToken(ErrorSpan, &'static str),
    UnexpectedEOF(ErrorSpan, &'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(span, err) => write!(f, "Unexpected token\n{}{}", span, err),
            Self::UnexpectedEOF(span, err) => write!(f, "Unexpected end of file\n{}{}", span, err),
        }
    }
}

pub struct Parser<'t> {
    tokens: &'t Tokens,
    index: usize,
}

impl<'t> Parser<'t> {
    fn new(tokens: &'t Tokens) -> Self {
        Self { tokens, index: 0 }
    }

    fn run(mut self) -> Result<Box<dyn Node>> {
        self.expr()
    }

    fn skip_token(&mut self) {
        self.index += 1;
    }

    fn peek_token(&self) -> Option<&'_ SpannedToken> {
        self.tokens.tokens.get(self.index)
    }

    fn make_error_span(&self, span: &Span) -> ErrorSpan {
        span.make_error_span(&self.tokens.code, self.tokens.source_type.clone())
    }

    fn make_eof_error_span(&self) -> ErrorSpan {
        self.make_error_span(&Span::new(
            self.tokens.code.len() - 1..self.tokens.code.len(),
        ))
    }

    fn factor(&mut self) -> Result<Box<dyn Node>> {
        match self.peek_token() {
            Some(&SpannedToken {
                value: Token::Lit(Lit::LitInt(i)),
                ..
            }) => {
                self.skip_token();
                Ok(Box::new(NumberNode::LitInt(i)))
            }
            Some(&SpannedToken {
                value: Token::Lit(Lit::LitFloat(f)),
                ..
            }) => {
                self.skip_token();
                Ok(Box::new(NumberNode::LitFloat(f)))
            }
            Some(&SpannedToken {
                value: Token::Group(Delimiter::Parentheses, Side::Left),
                ..
            }) => {
                self.skip_token();
                let expr = self.expr()?;
                match self.peek_token() {
                    Some(&SpannedToken {
                        value: Token::Group(Delimiter::Parentheses, Side::Right),
                        ..
                    }) => Ok(expr),
                    Some(SpannedToken { span, .. }) => Err(Error::UnexpectedToken(
                        self.make_error_span(span),
                        "expected ')'",
                    )),
                    None => Err(Error::UnexpectedEOF(
                        self.make_eof_error_span(),
                        "expected ')'",
                    )),
                }
            }
            Some(SpannedToken { span, .. }) => Err(Error::UnexpectedToken(
                self.make_error_span(span),
                "expected int or float",
            )),
            None => Err(Error::UnexpectedEOF(
                self.make_eof_error_span(),
                "expected int or float",
            )),
        }
    }

    fn term(&mut self) -> Result<Box<dyn Node>> {
        let mut lhs_node = self.factor()?;

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Operator(op),
                    ..
                }) if matches!(op, Operator::Mul | Operator::Div) => {
                    self.skip_token();
                    let rhs_node = self.factor()?;
                    lhs_node = Box::new(BinaryOpNode::new(op, lhs_node, rhs_node));
                }
                _ => break,
            }
        }

        Ok(lhs_node)
    }

    fn expr(&mut self) -> Result<Box<dyn Node>> {
        let mut lhs_node = self.term()?;

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Operator(op),
                    ..
                }) if matches!(op, Operator::Add | Operator::Sub) => {
                    self.skip_token();
                    let rhs_node = self.term()?;
                    lhs_node = Box::new(BinaryOpNode::new(op, lhs_node, rhs_node));
                }
                _ => break,
            }
        }

        Ok(lhs_node)
    }
}

pub fn run_parser(tokens: &Tokens) -> Result<Box<dyn Node>> {
    Parser::new(tokens).run()
}
