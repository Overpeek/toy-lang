use crate::artefact::tokens::{Delimiter, ErrorSpan, Group, Keyword, LitChar, LitFloat, LitInt, LitStr, Operator, Side, SourceType, Span, ToToken, Token, Tokens};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnexpectedEOF(ErrorSpan, &'static str),
    InvalidIdentifier(ErrorSpan, &'static str),
    InvalidCharacter(ErrorSpan, char),

    InvalidLitFloat(ErrorSpan, <f64 as FromStr>::Err),
    InvalidLitInt(ErrorSpan, <isize as FromStr>::Err),
    InvalidLitChar(ErrorSpan, &'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEOF(span, err) => write!(f, "Unexpected end of file\n{}{}", span, err),
            Self::InvalidIdentifier(span, err) => write!(f, "Invalid identifier\n{}{}", span, err),
            Self::InvalidCharacter(span, c) => write!(f, "Invalid character {}\n{}", c, span),
            Self::InvalidLitFloat(span, err) => write!(f, "Invalid float literal: {}\n{}", err, span),
            Self::InvalidLitInt(span, err) => write!(f, "Invalid int literal: {}\n{}", err, span),
            Self::InvalidLitChar(span, err) => write!(f, "Invalid char literal: {}\n{}", err, span),
        }
    }
}

struct LexerPosition {
    index: usize,

    row: usize,
    row_index: usize,
    column: usize,
}

impl LexerPosition {
    fn new() -> Self {
        Self {
            index: 0,
            row: 0,
            row_index: 0,
            column: 0,
        }
    }

    fn new_line(&mut self) {
        self.index += 1;
        self.row += 1;
        self.row_index = self.index;
        self.column = 0;
    }

    fn advance(&mut self, n: usize) {
        self.index += n;
        self.column += n;
    }
}

struct Lexer {
    position: LexerPosition,
    tokens: Tokens,
}

impl Lexer {
    fn new(code: &str, source_type: SourceType) -> Self {
        Self {
            position: LexerPosition::new(),
            tokens: Tokens::new(code, source_type),
        }
    }

    fn run(mut self) -> Result<Tokens> {
        while self.advance()? {}
        Ok(self.tokens)
    }

    fn advance(&mut self) -> Result<bool> {
        self.process()?;
        Ok((0..self.tokens.code.len()).contains(&self.position.index))
    }

    fn get_prev(&self) -> Option<char> {
        if self.position.index == 0 {
            None
        } else {
            Some(self.tokens.code[self.position.index - 1])
        }
    }

    fn get_this(&self) -> char {
        self.tokens.code[self.position.index]
    }

    fn get_next(&self) -> Option<char> {
        self.tokens.code.get(self.position.index + 1).map(|c| *c)
    }

    fn get_chars(&self) -> (Option<char>, char, Option<char>) {
        (self.get_prev(), self.get_this(), self.get_next())
    }

    fn make_span(&self, len: usize) -> Span {
        Span::new(self.position.index..self.position.index + len)
    }

    fn make_error_span(&self, len: usize) -> ErrorSpan {
        self.make_span(len)
            .make_error_span(&self.tokens.code, self.tokens.source_type.clone())
    }

    #[rustfmt::skip]
    fn process(&mut self) -> Result<()> {
        let chars = self.get_chars();

        let advance = match chars {
            // skip all whitespaces
            (_, '\n', _) =>         { self.position.new_line(); 0 }
            (_, c, _) if c.is_whitespace()
                =>                  { 1 },

            (_, '-', Some('>')) =>  { self.n_symbol_token(Token::Arrow, 2) },
            (_, '/', Some('/')) =>  { self.inline_comment()? },
            (_, '/', Some('*')) =>  { self.block_comment()? },

            (_, '.', _) =>          { self.n_symbol_token(Token::Dot, 1) },
            (_, ',', _) =>          { self.n_symbol_token(Token::Comma, 1) },
            (_, ':', _) =>          { self.n_symbol_token(Token::Colon, 1) },
            (_, ';', _) =>          { self.n_symbol_token(Token::Semicolon, 1) },

            (_, '>', Some('=')) =>  { self.n_symbol_token(Operator::Ge.to_token(), 2) },
            (_, '>', _) =>          { self.n_symbol_token(Operator::Gt.to_token(), 1) },
            (_, '<', Some('=')) =>  { self.n_symbol_token(Operator::Le.to_token(), 2) },
            (_, '<', _) =>          { self.n_symbol_token(Operator::Lt.to_token(), 1) },
            (_, '=', Some('=')) =>  { self.n_symbol_token(Operator::Eq.to_token(), 2) },
            (_, '=', _) =>          { self.n_symbol_token(Token::Assign, 1) },

            (_, '+', _) =>          { self.n_symbol_token(Operator::Add.to_token(), 1) },
            (_, '-', _) =>          { self.n_symbol_token(Operator::Sub.to_token(), 1) },
            (_, '*', _) =>          { self.n_symbol_token(Operator::Mul.to_token(), 1) },
            (_, '/', _) =>          { self.n_symbol_token(Operator::Div.to_token(), 1) },

            (_, '(', _) =>          { self.n_symbol_token(Group::new(Delimiter::Parentheses, Side::Left).to_token(), 1) },
            (_, ')', _) =>          { self.n_symbol_token(Group::new(Delimiter::Parentheses, Side::Right).to_token(), 1) },
            (_, '{', _) =>          { self.n_symbol_token(Group::new(Delimiter::Braces, Side::Left).to_token(), 1) },
            (_, '}', _) =>          { self.n_symbol_token(Group::new(Delimiter::Braces, Side::Right).to_token(), 1) },
            (_, '[', _) =>          { self.n_symbol_token(Group::new(Delimiter::Brackets, Side::Left).to_token(), 1) },
            (_, ']', _) =>          { self.n_symbol_token(Group::new(Delimiter::Brackets, Side::Right).to_token(), 1) },

            (_, '\"', _) =>         { self.lit_str()? },
            (_, '\'', _) =>         { self.lit_char()? },
            (_, '0'..='9', _) =>    { self.lit_num()? },

            (_, 'a'..='z' | 'A'..='Z', _) =>    { self.identifier()? },
            
            other => return Err(Error::InvalidCharacter(self.make_error_span(1), other.1)),
        };
        self.position.advance(advance);

        Ok(())
    }

    fn identifier(&mut self) -> Result<usize> {
        let mut offset = 0;
        loop {
            let c = match self.tokens.code.get(self.position.index + offset) {
                Some(&c) => c,
                None => break,
            };

            if c.is_alphanumeric() || c == '_' {
                offset += 1;
            } else {
                break;
            }

        };

        let span = Span::new(self.position.index..self.position.index + offset);
        let s = self.tokens.code[span.range()].into_iter().collect::<String>();

        // keyword matching
        let token = match s.as_str() {
            "fn" => Keyword::Fn.to_token(),
            "let" => Keyword::Let.to_token(),
            "if" => Keyword::If.to_token(),
            "else" => Keyword::Else.to_token(),
            "true" => Keyword::True.to_token(),
            "false" => Keyword::False.to_token(),
            _ => Token::Ident(s),
        };

        self.tokens.tokens.push(token.to_spanned(span));

        Ok(offset)
    }

    fn n_symbol_token(&mut self, token: Token, n: usize) -> usize {
        self.tokens.tokens.push(token.to_spanned(self.make_span(n)));
        n
    }

    fn block_comment(&self) -> Result<usize> {
        let mut last = '\0';
        let last = match self.find_next(self.position.index, |&(_, &c)| {
            let result = last == '*' && c == '/';
            last = c;
            result
        }) {
            Some(last) => last,
            None => {
                return Err(Error::UnexpectedEOF(
                    self.make_error_span(1),
                    "while waiting for the tailing */",
                ))
            }
        };

        Ok(last + 1)
    }

    fn inline_comment(&self) -> Result<usize> {
        let mut last = self.position.index;
        let last = match self.find_next(last, |&(i, &c)| {
            last = i;
            c == '\n'
        }) {
            Some(last) => last,
            None => last + 1,
        };

        println!("{}", last);

        Ok(last)
    }

    fn find_next<P>(&self, after: usize, pred: P) -> Option<usize>
    where
        P: FnMut(&(usize, &char)) -> bool,
    {
        self.tokens.code[after..]
            .iter()
            .enumerate()
            .find(pred)
            .map(|(i, _)| i)
    }

    fn lit_str(&mut self) -> Result<usize> {
        let first = self.position.index;

        let last = match self.find_next(first + 1, |&(_, &c)| c == '\"') {
            Some(last) => last,
            None => {
                return Err(Error::UnexpectedEOF(
                    self.make_error_span(1),
                    "while waiting for the tailing \"",
                ))
            }
        };

        // TODO: escapes

        let span = Span::new(first + 1..first + 1 + last);
        self.tokens.tokens.push(
            LitStr::new(
                self.tokens.code[span.range()]
                    .into_iter()
                    .collect::<String>(),
            )
            .to_spanned_token(span),
        );

        Ok(last + 3)
    }

    fn lit_char(&mut self) -> Result<usize> {
        let first = self.position.index;
        let last = match self.tokens.code[first + 1..]
            .iter()
            .enumerate()
            .find(|&(_, &c)| c == '\'')
        {
            Some((last, _)) => last,
            None => {
                return Err(Error::UnexpectedEOF(
                    self.make_error_span(1),
                    "while waiting for the tailing '",
                ))
            }
        };

        if last != 1 {
            return Err(Error::InvalidLitChar(
                self.make_error_span(last),
                "a char has to have exactly one codepoint",
            ));
        }

        // TODO: escapes

        let span = Span::new(first + 1..first + 2);
        self.tokens
            .tokens
            .push(LitChar::new(self.tokens.code[span.range().start]).to_spanned_token(span));

        Ok(last + 3)
    }

    fn parse_radix(&mut self) -> (u32, usize) {
        match self.get_chars() {
            (_, '0', Some('x')) => (16, 2),
            (_, '0', Some('o')) => (8, 2),
            (_, '0', Some('b')) => (2, 2),
            _ => (10, 0),
        }
    }

    fn lit_num(&mut self) -> Result<usize> {
        let (radix, mut offset) = self.parse_radix();
        let mut dot = false;

        loop {
            let c = match self.tokens.code.get(self.position.index + offset) {
                Some(&c) => c,
                None => break,
            };
            let is_dot = c == '.';
            let is_digit = c.is_digit(radix);

            // digit ends when there are no more numbers or dots
            // and if the char after dot is not a digit
            if !is_dot && !is_digit {
                break;
            }

            // first dot means that it is a float
            // second dot ends the digit
            if dot && is_dot {
                break;
            } else if is_dot {
                dot = true;
            }

            offset += 1;
        }

        let span = Span::new(self.position.index..self.position.index + offset);
        let digit_str = self.tokens.code[span.range()]
            .into_iter()
            .collect::<String>();

        self.tokens.tokens.push(if dot {
            LitFloat::new(
                digit_str.parse().or_else(|err| {
                    Err(Error::InvalidLitFloat(self.make_error_span(offset), err))
                })?,
            )
            .to_spanned_token(span)
        } else {
            LitInt::new(
                digit_str
                    .parse()
                    .or_else(|err| Err(Error::InvalidLitInt(self.make_error_span(offset), err)))?,
            )
            .to_spanned_token(span)
        });

        Ok(offset)
    }
}

pub fn run_lexer(code: &str, source_type: SourceType) -> Result<Tokens> {
    let result = Lexer::new(code, source_type).run();

    if let Ok(result) = &result {
        log::debug!("got result {:?}", result.tokens);
    }

    result
}
