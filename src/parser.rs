//! Parsing of REPL input into function calls.
//!
//! Grammar:
//!
//! ```text
//! call   := ident "(" [ arg { "," arg } ] ")"
//! arg    := ident "=" expr
//! expr   := term { ("+" | "-") term }
//! term   := factor { ("*" | "/") factor }
//! factor := number | "-" factor | "(" expr ")"
//! ```

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// A parsed function call, e.g. `bi_trial(p = 0.5)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Arg>,
}

/// A named argument whose value has already been evaluated.
#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: String,
    pub value: f64,
}

/// An error raised while tokenizing or parsing input.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedChar(char),
    UnexpectedToken {
        expected: &'static str,
        found: String,
    },
    InvalidNumber(String),
    TrailingInput(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedChar(c) => write!(f, "unexpected character `{c}`"),
            Self::UnexpectedToken { expected, found } => {
                write!(f, "expected {expected}, found {found}")
            }
            Self::InvalidNumber(s) => write!(f, "invalid number `{s}`"),
            Self::TrailingInput(s) => write!(f, "unexpected trailing input starting at {s}"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Number(f64),
    LParen,
    RParen,
    Comma,
    Equals,
    Plus,
    Minus,
    Star,
    Slash,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(s) => write!(f, "`{s}`"),
            Self::Number(n) => write!(f, "`{n}`"),
            Self::LParen => f.write_str("`(`"),
            Self::RParen => f.write_str("`)`"),
            Self::Comma => f.write_str("`,`"),
            Self::Equals => f.write_str("`=`"),
            Self::Plus => f.write_str("`+`"),
            Self::Minus => f.write_str("`-`"),
            Self::Star => f.write_str("`*`"),
            Self::Slash => f.write_str("`/`"),
        }
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&c) = chars.peek() {
        let token = match c {
            c if c.is_whitespace() => {
                chars.next();
                continue;
            }
            c if c.is_ascii_alphabetic() || c == '_' => Token::Ident(take_while(&mut chars, |c| {
                c.is_ascii_alphanumeric() || c == '_'
            })),
            c if c.is_ascii_digit() || c == '.' => {
                let text = take_while(&mut chars, |c| c.is_ascii_digit() || c == '.');
                Token::Number(text.parse().map_err(|_| ParseError::InvalidNumber(text))?)
            }
            _ => {
                chars.next();
                match c {
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    ',' => Token::Comma,
                    '=' => Token::Equals,
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Star,
                    '/' => Token::Slash,
                    other => return Err(ParseError::UnexpectedChar(other)),
                }
            }
        };
        tokens.push(token);
    }

    Ok(tokens)
}

fn take_while(chars: &mut Peekable<Chars<'_>>, pred: impl Fn(char) -> bool) -> String {
    let mut out = String::new();
    while let Some(c) = chars.next_if(|&c| pred(c)) {
        out.push(c);
    }
    out
}

struct Parser {
    tokens: std::vec::IntoIter<Token>,
    peeked: Option<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let mut tokens = tokens.into_iter();
        let peeked = tokens.next();
        Self { tokens, peeked }
    }

    fn peek(&self) -> Option<&Token> {
        self.peeked.as_ref()
    }

    fn bump(&mut self) -> Option<Token> {
        std::mem::replace(&mut self.peeked, self.tokens.next())
    }

    fn describe(token: Option<&Token>) -> String {
        token.map_or_else(|| "end of input".to_owned(), Token::to_string)
    }

    fn expect(&mut self, expected: Token, what: &'static str) -> Result<(), ParseError> {
        if self.peek() == Some(&expected) {
            self.bump();
            Ok(())
        } else {
            Err(self.unexpected(what))
        }
    }

    fn unexpected(&self, expected: &'static str) -> ParseError {
        ParseError::UnexpectedToken {
            expected,
            found: Self::describe(self.peek()),
        }
    }

    fn ident(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token::Ident(_)) => match self.bump() {
                Some(Token::Ident(name)) => Ok(name),
                _ => unreachable!("peeked token was an identifier"),
            },
            _ => Err(self.unexpected("an identifier")),
        }
    }

    fn call(&mut self) -> Result<Call, ParseError> {
        let name = self.ident()?;
        self.expect(Token::LParen, "`(`")?;

        let mut args = Vec::new();
        if self.peek() != Some(&Token::RParen) {
            loop {
                args.push(self.arg()?);
                if self.peek() == Some(&Token::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen, "`,` or `)`")?;

        match self.peek() {
            None => Ok(Call { name, args }),
            Some(token) => Err(ParseError::TrailingInput(token.to_string())),
        }
    }

    fn arg(&mut self) -> Result<Arg, ParseError> {
        let name = self.ident()?;
        self.expect(Token::Equals, "`=`")?;
        let value = self.expr()?;
        Ok(Arg { name, value })
    }

    fn expr(&mut self) -> Result<f64, ParseError> {
        let mut value = self.term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.bump();
                    value += self.term()?;
                }
                Some(Token::Minus) => {
                    self.bump();
                    value -= self.term()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn term(&mut self) -> Result<f64, ParseError> {
        let mut value = self.factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.bump();
                    value *= self.factor()?;
                }
                Some(Token::Slash) => {
                    self.bump();
                    value /= self.factor()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn factor(&mut self) -> Result<f64, ParseError> {
        match self.peek() {
            Some(Token::Number(n)) => {
                let n = *n;
                self.bump();
                Ok(n)
            }
            Some(Token::Minus) => {
                self.bump();
                Ok(-self.factor()?)
            }
            Some(Token::LParen) => {
                self.bump();
                let value = self.expr()?;
                self.expect(Token::RParen, "`)`")?;
                Ok(value)
            }
            _ => Err(self.unexpected("a number")),
        }
    }
}

/// Parses a single line of input into a [`Call`].
pub fn parse(input: &str) -> Result<Call, ParseError> {
    Parser::new(tokenize(input)?).call()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(name: &str, value: f64) -> Arg {
        Arg {
            name: name.to_owned(),
            value,
        }
    }

    #[test]
    fn parses_call_with_spaced_arg() {
        let call = parse("bi_trial(p = 0.5)").unwrap();
        assert_eq!(call.name, "bi_trial");
        assert_eq!(call.args, vec![arg("p", 0.5)]);
    }

    #[test]
    fn evaluates_arithmetic_in_args() {
        let call = parse("bi_trial(q=1/3)").unwrap();
        assert_eq!(call.args, vec![arg("q", 1.0 / 3.0)]);

        let call = parse("f(x = -(1 + 2) * 2 - 1, y=.25)").unwrap();
        assert_eq!(call.args, vec![arg("x", -7.0), arg("y", 0.25)]);
    }

    #[test]
    fn parses_call_without_args() {
        let call = parse("f()").unwrap();
        assert!(call.args.is_empty());
    }

    #[test]
    fn rejects_malformed_input() {
        assert!(matches!(
            parse("bi_trial(p=)"),
            Err(ParseError::UnexpectedToken { .. })
        ));
        assert!(matches!(
            parse("bi_trial(p=0.5"),
            Err(ParseError::UnexpectedToken { .. })
        ));
        assert!(matches!(
            parse("bi_trial(0.5)"),
            Err(ParseError::UnexpectedToken { .. })
        ));
        assert!(matches!(
            parse("bi_trial(p=1.2.3)"),
            Err(ParseError::InvalidNumber(_))
        ));
        assert!(matches!(
            parse("bi_trial(p=0.5) x"),
            Err(ParseError::TrailingInput(_))
        ));
        assert!(matches!(
            parse("bi_trial(p=0.5;"),
            Err(ParseError::UnexpectedChar(';'))
        ));
    }
}
