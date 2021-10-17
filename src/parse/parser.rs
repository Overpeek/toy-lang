use std::{borrow::Cow, fmt::Display};

use crate::artefact::{
    ast::{
        AccessNode, AssignNode, BinaryOpNode, BooleanNode, IfElseNode, Node, NumberNode, ScopeNode,
        UnaryOpNode,
    },
    tokens::{
        Delimiter, ErrorSpan, Group, Keyword, Lit, Operator, Side, Span, SpannedToken, ToToken,
        Token, Tokens,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    UnexpectedToken(ErrorSpan, Token, Cow<'static, str>),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(span, token, err) => write!(f, "Unexpected token '{:?}'\n{}{}", token, span, err),
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

    fn run(mut self) -> Result<Node> {
        let result = self.scope()?;

        match self.peek_token() {
            Some(&SpannedToken {
                value: Token::EOF, ..
            })
            | None => Ok(result),
            Some(SpannedToken { span, value }) => Err(Error::UnexpectedToken(
                self.make_error_span(span),
                value.clone(),
                format!("expected 'EOF'").into(),
            )),
        }
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

    fn expect(&self, expected_token: &Token) -> Option<SpannedToken> {
        match self.peek_token() {
            Some(SpannedToken { value, .. }) if value == expected_token => None,
            Some(token) => Some(token.clone()),
            None => Some(SpannedToken::new(
                Token::EOF,
                Span::new(self.tokens.code.len() - 1..self.tokens.code.len()),
            )),
        }
    }

    fn expect_or(&self, expected_token: &Token) -> Result<()> {
        if let Some(token) = self.expect(expected_token) {
            return Err(Error::UnexpectedToken(
                self.make_error_span(&token.span),
                token.value.clone(),
                format!("expected: '{:?}'", expected_token).into(),
            ));
        } else {
            Ok(())
        }
    }

    fn factor(&mut self) -> Result<Node> {
        match self.peek_token() {
            Some(&SpannedToken {
                value: Token::Lit(Lit::LitInt(i)),
                ..
            }) => {
                self.skip_token();

                Ok(Node::NumberNode(NumberNode::LitInt(i)))
            }
            Some(&SpannedToken {
                value: Token::Lit(Lit::LitFloat(f)),
                ..
            }) => {
                self.skip_token();
                Ok(Node::NumberNode(NumberNode::LitFloat(f)))
            }
            Some(&SpannedToken {
                value: Token::Keyword(Keyword::True),
                ..
            }) => {
                self.skip_token();
                Ok(Node::BooleanNode(BooleanNode::new(true)))
            }
            Some(&SpannedToken {
                value: Token::Keyword(Keyword::False),
                ..
            }) => {
                self.skip_token();
                Ok(Node::BooleanNode(BooleanNode::new(false)))
            }
            Some(SpannedToken {
                value: Token::Ident(ident),
                ..
            }) => {
                let ident = ident.clone();
                self.skip_token();
                Ok(Node::AccessNode(AccessNode::new(ident)))
            }
            Some(&SpannedToken {
                value:
                    Token::Group(Group {
                        delimiter: Delimiter::Parentheses,
                        side: Side::Left,
                    }),
                ..
            }) => {
                self.skip_token();
                let expr = self.expr()?;

                if let Some(token) = self.expect(&Token::Group(Group {
                    delimiter: Delimiter::Parentheses,
                    side: Side::Right,
                })) {
                    return Err(Error::UnexpectedToken(
                        self.make_error_span(&token.span),
                        token.value.clone(),
                        "expected ')'".into(),
                    ));
                };
                self.skip_token();

                Ok(expr)
            }
            Some(&SpannedToken {
                value: Token::Operator(op),
                ..
            }) if matches!(op, Operator::Add | Operator::Sub) => {
                self.skip_token();
                let node = self.factor()?;
                Ok(Node::UnaryOpNode(UnaryOpNode::new(op, node)))
            }
            Some(&SpannedToken {
                value: Token::Keyword(Keyword::If),
                ..
            }) => {
                self.skip_token();

                let test = self.expr()?;

                self.expect_or(&Group::new(Delimiter::Braces, Side::Left).to_token())?;
                self.skip_token();

                let on_true = self.expr()?;

                self.expect_or(&Group::new(Delimiter::Braces, Side::Right).to_token())?;
                self.skip_token();

                self.expect_or(&Keyword::Else.to_token())?;
                self.skip_token();

                self.expect_or(&Group::new(Delimiter::Braces, Side::Left).to_token())?;
                self.skip_token();

                let on_false = self.expr()?;

                self.expect_or(&Group::new(Delimiter::Braces, Side::Right).to_token())?;
                self.skip_token();

                Ok(Node::IfElseNode(IfElseNode::new(test, on_true, on_false)))
            }
            Some(&SpannedToken {
                value: Token::Keyword(Keyword::Let),
                ..
            }) => {
                self.skip_token();

                let ident = match self.peek_token() {
                    Some(SpannedToken {
                        value: Token::Ident(ident),
                        ..
                    }) => {
                        let ident = ident.clone();
                        self.skip_token();
                        ident
                    }
                    Some(token) => {
                        return Err(Error::UnexpectedToken(
                            self.make_error_span(&token.span),
                            token.value.clone(),
                            "expected identifier".into(),
                        ))
                    }
                    None => {
                        return Err(Error::UnexpectedToken(
                            self.make_error_span(&Span::new(
                                self.tokens.code.len() - 1..self.tokens.code.len(),
                            )),
                            Token::EOF,
                            "expected identifier".into(),
                        ))
                    }
                };

                self.expect_or(&Token::Assign)?;
                self.skip_token();

                let expr = self.expr()?;

                Ok(Node::AssignNode(AssignNode::new(ident, expr)))
            }
            Some(token) => Err(Error::UnexpectedToken(
                self.make_error_span(&token.span),
                token.value.clone(),
                "expected int or float".into(),
            )),
            None => Err(Error::UnexpectedToken(
                self.make_error_span(&Span::new(0..0)),
                Token::EOF,
                "expected int or float".into(),
            )),
        }
    }

    fn term(&mut self) -> Result<Node> {
        let mut lhs_node = self.factor()?;

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Operator(op),
                    ..
                }) if matches!(op, Operator::Mul | Operator::Div) => {
                    self.skip_token();
                    let rhs_node = self.factor()?;
                    lhs_node = Node::BinaryOpNode(BinaryOpNode::new(op, lhs_node, rhs_node));
                }
                _ => break,
            }
        }

        Ok(lhs_node)
    }

    fn arith_expr(&mut self) -> Result<Node> {
        let mut lhs_node = self.term()?;

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Operator(op),
                    ..
                }) if matches!(op, Operator::Add | Operator::Sub) => {
                    self.skip_token();
                    let rhs_node = self.term()?;
                    lhs_node = Node::BinaryOpNode(BinaryOpNode::new(op, lhs_node, rhs_node));
                }
                _ => break,
            }
        }

        Ok(lhs_node)
    }

    fn expr(&mut self) -> Result<Node> {
        let mut lhs_node = self.arith_expr()?;

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Operator(op),
                    ..
                }) if matches!(
                    op,
                    Operator::Eq | Operator::Lt | Operator::Gt | Operator::Le | Operator::Ge
                ) =>
                {
                    self.skip_token();
                    let rhs_node = self.arith_expr()?;
                    lhs_node = Node::BinaryOpNode(BinaryOpNode::new(op, lhs_node, rhs_node));
                }
                _ => break,
            }
        }

        Ok(lhs_node)
    }

    fn scope(&mut self) -> Result<Node> {
        let mut scope = ScopeNode::new();
        scope.push_line(self.expr()?);

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Semicolon,
                    ..
                }) => {
                    self.skip_token();
                    scope.push_line(self.expr()?);
                }
                _ => break,
            }
        }

        Ok(Node::ScopeNode(scope))
    }
}

pub fn run_parser(tokens: &Tokens) -> Result<Node> {
    let result = Parser::new(tokens).run();

    if let Ok(result) = &result {
        log::debug!("got result {}", result);
    }

    result
}
