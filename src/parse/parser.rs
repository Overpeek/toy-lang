use std::{borrow::Cow, fmt::Display};

use crate::artefact::{
    ast::{
        AccessNode, AssignNode, BinaryOpNode, BooleanNode, FnNode, IfElseNode, Node, ScopeNode,
        UnaryOpNode,
    },
    tokens::{
        Delimiter, ErrorSpan, Group, Keyword, Operator, Side, Span, SpannedToken, ToToken, Token,
        Tokens,
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
            Self::UnexpectedToken(span, token, err) => write!(f, "Unexpected token '{}'\n{}{}", token, span, err),
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
        let result = self.func()?;

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

    fn make_eof_span(&self) -> Span {
        Span::new(self.tokens.code.len()..self.tokens.code.len() + 1)
    }

    fn expect_or_span(&self, expected_token: &Token) -> Option<SpannedToken> {
        match self.peek_token() {
            Some(SpannedToken { value, .. }) if value == expected_token => None,
            Some(token) => Some(token.clone()),
            None => Some(SpannedToken::new(Token::EOF, self.make_eof_span())),
        }
    }

    fn expect(&self, expected_token: &Token) -> Result<()> {
        if let Some(token) = self.expect_or_span(expected_token) {
            return Err(Error::UnexpectedToken(
                self.make_error_span(&token.span),
                token.value.clone(),
                format!("expected: '{}'", expected_token).into(),
            ));
        } else {
            Ok(())
        }
    }

    fn parse(&mut self, expected_token: &Token) -> Result<()> {
        self.expect(expected_token)?;
        self.skip_token();
        Ok(())
    }

    fn ident(&mut self) -> Result<String> {
        match self.peek_token() {
            Some(SpannedToken {
                value: Token::Ident(ident),
                ..
            }) => {
                let ident = ident.clone();
                self.skip_token();
                Ok(ident)
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
                    self.make_error_span(&self.make_eof_span()),
                    Token::EOF,
                    "expected identifier".into(),
                ))
            }
        }
    }

    fn factor(&mut self) -> Result<Node> {
        match self.peek_token() {
            Some(SpannedToken {
                value: Token::Lit(lit),
                ..
            }) => {
                let lit = lit.clone().into();
                self.skip_token();
                Ok(Node::LitNode(lit))
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

                self.parse(&Group::new(Delimiter::Parentheses, Side::Right).to_token())?;

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

                self.parse(&Group::new(Delimiter::Braces, Side::Left).to_token())?;

                let on_true = self.expr()?;

                self.parse(&Group::new(Delimiter::Braces, Side::Right).to_token())?;
                self.parse(&Keyword::Else.to_token())?;
                self.parse(&Group::new(Delimiter::Braces, Side::Left).to_token())?;

                let on_false = self.expr()?;

                self.parse(&Group::new(Delimiter::Braces, Side::Right).to_token())?;

                Ok(Node::IfElseNode(IfElseNode::new(test, on_true, on_false)))
            }
            Some(&SpannedToken {
                value: Token::Keyword(Keyword::Let),
                ..
            }) => {
                self.skip_token();

                let ident = self.ident()?;

                self.parse(&Token::Assign)?;

                let expr = self.expr()?;

                Ok(Node::AssignNode(AssignNode::new(ident, expr)))
            }
            Some(token) => Err(Error::UnexpectedToken(
                self.make_error_span(&token.span),
                token.value.clone(),
                "expected int or float".into(),
            )),
            None => Err(Error::UnexpectedToken(
                self.make_error_span(&self.make_eof_span()),
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

    fn scope(&mut self) -> Result<ScopeNode> {
        let mut scope = ScopeNode::new();
        let line = self.expr()?;
        log::debug!("parsed line: {}", line);
        scope.push_line(line);

        loop {
            match self.peek_token() {
                Some(&SpannedToken {
                    value: Token::Semicolon,
                    ..
                }) => {
                    self.skip_token();
                    let line = self.expr()?;
                    log::debug!("parsed line: {}", line);
                    scope.push_line(line);
                }
                _ => break,
            }
        }

        Ok(scope)
    }

    fn func(&mut self) -> Result<Node> {
        self.parse(&Keyword::Fn.to_token())?;

        let name = self.ident()?;

        self.parse(&Group::new(Delimiter::Parentheses, Side::Left).to_token())?;
        self.parse(&Group::new(Delimiter::Parentheses, Side::Right).to_token())?;

        self.parse(&Group::new(Delimiter::Braces, Side::Left).to_token())?;
        let body = self.scope()?;
        self.parse(&Group::new(Delimiter::Braces, Side::Right).to_token())?;

        Ok(Node::FnNode(FnNode::new(name, body)))
    }
}

pub fn run_parser(tokens: &Tokens) -> Result<Node> {
    let result = Parser::new(tokens).run();

    if let Ok(result) = &result {
        log::debug!("got result {}", result);
    }

    result
}
