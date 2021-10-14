use crate::artefact::tokens::{Delimiter, Operator, Side, Token, Tokens};
use std::{
    iter::Peekable,
    str::{CharIndices, FromStr},
};

type Iter<'a> = Peekable<CharIndices<'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidFloat(<f64 as FromStr>::Err),
    InvalidInt(<isize as FromStr>::Err),
    UnexpectedEOF,
    InvalidIdentifier,
    InvalidCharacter(char),
}

pub type Result<T> = std::result::Result<T, Error>;

/* impl From<<f64 as FromStr>::Err> for Error {
    fn from(e: <f64 as FromStr>::Err) -> Self {
        Self::InvalidFloat(e)
    }
} */

pub fn run_lexer(code: &str) -> Result<Tokens> {
    let mut tokens = Tokens::new();
    let mut iter = code.char_indices().peekable();

    loop {
        let token = advance(code, &mut iter)?;
        let is_eof = token == Token::EOF;

        tokens.push(token);

        if is_eof {
            break;
        }
    }

    Ok(tokens)
}

fn advance(code: &str, iter: &mut Iter) -> Result<Token> {
    loop {
        dbg!(iter.peek());
        match iter.peek() {
            Some(&(_, '.')) => {
                let _ = iter.next();
                break Ok(Token::Dot);
            }
            Some(&(_, ',')) => {
                let _ = iter.next();
                break Ok(Token::Comma);
            }
            Some(&(_, '+')) => {
                let _ = iter.next();
                break Ok(Token::Operator(Operator::Add));
            }
            Some(&(_, '-')) => {
                let _ = iter.next().unwrap();
                let next = iter.peek().map(|&(_, c)| c);
                match next {
                    Some('>') => {
                        let _ = iter.next();
                        break Ok(Token::Arrow);
                    }
                    _ => {
                        break Ok(Token::Operator(Operator::Sub));
                    }
                }
            }
            Some(&(_, '*')) => {
                let _ = iter.next();
                break Ok(Token::Operator(Operator::Mul));
            }
            Some(&(_, '/')) => {
                let _ = iter.next();
                let next = iter.peek().map(|&(_, c)| c);
                match next {
                    Some('*') => {
                        let _ = iter.next();
                        break Ok(Token::BlockComment(Side::Left));
                    }
                    Some('/') => {
                        let _ = iter.next();
                        break Ok(Token::InlineComment(Side::Left));
                    }
                    _ => {
                        break Ok(Token::Operator(Operator::Div));
                    }
                }
            }
            Some(&(_, '(')) => {
                let _ = iter.next();
                break Ok(Token::Group(Delimiter::Parentheses, Side::Left));
            }
            Some(&(_, ')')) => {
                let _ = iter.next();
                break Ok(Token::Group(Delimiter::Parentheses, Side::Right));
            }
            Some(&(_, '{')) => {
                let _ = iter.next();
                break Ok(Token::Group(Delimiter::Braces, Side::Left));
            }
            Some(&(_, '}')) => {
                let _ = iter.next();
                break Ok(Token::Group(Delimiter::Braces, Side::Right));
            }
            Some(&(_, '[')) => {
                let _ = iter.next();
                break Ok(Token::Group(Delimiter::Brackets, Side::Left));
            }
            Some(&(_, ']')) => {
                let _ = iter.next();
                break Ok(Token::Group(Delimiter::Brackets, Side::Right));
            }
            Some(&(_, '"')) => {
                break Ok(Token::LitStr(
                    collect_delimited(code, iter, '"')?.to_string(),
                ));
            }
            Some(&(_, '\'')) => {
                todo!(
                    "lit char not implemented but the output was: {:?}",
                    collect_delimited(code, iter, '\'')
                );
            }
            Some(&(_, c)) => {
                // skip whitespace
                if c.is_whitespace() {
                    let _ = iter.next();
                    continue;
                }

                if c.is_digit(10) {
                    break collect_digit(code, iter);
                } else if c.is_alphabetic() {
                    break collect_ident(code, iter);
                } else {
                    break Err(Error::InvalidCharacter(c));
                }
            }
            None => {
                let _ = iter.next();
                break Ok(Token::EOF);
            }
        }
    }
}

fn collect_digit(code: &str, iter: &mut Iter) -> Result<Token> {
    let mut dot = false;
    let (first, _) = *iter.peek().unwrap();
    let mut last = first;

    loop {
        match iter.peek() {
            Some(&(i, c)) => {
                let is_digit = c.is_digit(10);
                let is_dot = c == '.';
                // let next_is_digit = next_is_digit(iter);

                // digit ends when there are no more numbers or dots
                // and if the char after dot is not a digit
                if !is_digit && !is_dot {
                    break;
                }
                /* if (!is_digit && !is_dot) || (is_dot && !next_is_digit) {
                    println!(
                        "complicated break: {} {} {}",
                        is_digit, is_dot, next_is_digit
                    );
                    break;
                } */

                // first dot means that it is a float
                // second dot ends the digit
                if dot && is_dot {
                    break;
                } else if is_dot {
                    dot = true;
                }

                let _ = iter.next();
                last = i + c.len_utf8();
            }
            None => {
                break;
            }
        }
    }

    let digit_str = &code[first..last];
    let token = if dot {
        Token::LitFloat(
            digit_str
                .parse()
                .or_else(|err| Err(Error::InvalidFloat(err)))?,
        )
    } else {
        Token::LitInt(
            digit_str
                .parse()
                .or_else(|err| Err(Error::InvalidInt(err)))?,
        )
    };

    Ok(token)
}

fn collect_delimited<'s>(code: &'s str, iter: &mut Iter, delimiter: char) -> Result<&'s str> {
    // consume the first delimiter
    let (_, first_c) = iter.next().unwrap();
    let (first, _) = *iter.peek().unwrap();

    assert!(first_c == delimiter);

    let last = match iter.find(|&(_, c)| c == delimiter) {
        Some((last, d)) => {
            assert!(d == delimiter);
            last
        }
        None => return Err(Error::UnexpectedEOF),
    };

    Ok(&code[first..last])
}

fn collect_ident(code: &str, iter: &mut Iter) -> Result<Token> {
    let (first, first_c) = iter.next().unwrap();
    let mut last = first;

    // consume the first delimiter
    if !first_c.is_alphabetic() {
        return Err(Error::InvalidIdentifier);
    }

    loop {
        match iter.peek() {
            Some((i, c)) => {
                last = *i;
                if !c.is_alphanumeric() {
                    break;
                } else {
                    let _ = iter.next();
                }
            }
            None => break,
        }
    }

    Ok(Token::Ident(code[first..last].to_string()))
}
