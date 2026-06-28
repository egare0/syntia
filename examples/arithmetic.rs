//! Minimal arithmetic evaluator built with syntia.
//!
//! Parses and evaluates expressions like `1 + 2 - 3`.
//! Run with: cargo run --example arithmetic

use syntia::{Source, Span, Spanned, Token};
use syntia::lexer::{Lex, Cursor};
use syntia::parser::{ParseError, TokenStream, Parse};

// ── token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenKind {
    Number(i64),
    Plus,
    Minus,
    Eof
}

#[derive(Debug, Clone, Copy)]
struct Tok {
    kind: TokenKind,
    span: Span,
}

impl Token for Tok {
    fn span(&self) -> Span {
        self.span
    }
    fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

// ── lexer ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct LexError {
    ch: char,
    span: Span,
}

struct ArithLexer;

impl Lex for ArithLexer {
    type Token = Tok;
    type Error = LexError;

    fn lex(&mut self, input: &str) -> Result<Vec<Tok>, Vec<LexError>> {
        let source = Source::new(input);
        let mut cursor = Cursor::new(input);
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            cursor.eat_whitespace();

            if cursor.is_at_end() {
                tokens.push(Tok {
                    kind: TokenKind::Eof,
                    span: Span::point(cursor.offset())
                });
                break;
            }

            match cursor.peek() {
                Some('+') => {
                    let start = cursor.offset();
                    cursor.advance();
                    tokens.push(Tok {
                        kind: TokenKind::Plus,
                        span: cursor.span_from(start)
                    });
                }
                Some('-') => {
                    let start = cursor.offset();
                    cursor.advance();
                    tokens.push(Tok {
                        kind: TokenKind::Minus,
                        span: cursor.span_from(start)
                    });
                }
                Some(c) if c.is_ascii_digit() => {
                    let span = cursor.eat_digits(10).unwrap();

                    let val_str = source.slice(span);
                    let val = val_str.parse::<i64>().unwrap();

                    tokens.push(Tok {
                        kind: TokenKind::Number(val),
                        span
                    });
                }
                Some(c) => {
                    let start = cursor.offset();
                    cursor.advance();
                    errors.push(LexError {
                        ch: c,
                        span: cursor.span_from(start)
                    });
                }
                None => unreachable!()
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }
}

// ── ast ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Op {
    Add,
    Sub
}

#[derive(Debug)]
struct Expr {
    first: Spanned<i64>,
    rest: Vec<(Op, Spanned<i64>)>,
}

impl Expr {
    fn eval(&self) -> i64 {
        self.rest.iter().fold(self.first.node, |acc, (op, term)| match op {
            Op::Add => acc + term.node,
            Op::Sub => acc - term.node,
        })
    }
}

// ── parser ────────────────────────────────────────────────────────────────────

impl Parse<Tok> for Spanned<i64> {
    fn parse(stream: &mut TokenStream<Tok>) -> Result<Spanned<i64>, ParseError> {
        let tok = stream.expect(|t| matches!(t.kind, TokenKind::Number(_)), "a number")?;

        if let TokenKind::Number(val) = tok.kind {
            Ok(Spanned::new(val, tok.span))
        } else {
            unreachable!()
        }
    }
}

impl Parse<Tok> for Expr {
    fn parse(stream: &mut TokenStream<Tok>) -> Result<Expr, ParseError> {
        let first = Spanned::<i64>::parse(stream)?;
        let mut rest = Vec::new();

        loop {
            let op = match stream.peek().kind {
                TokenKind::Plus => {
                    stream.advance();
                    Op::Add
                }
                TokenKind::Minus => {
                    stream.advance();
                    Op::Sub
                }
                _ => break
            };

            rest.push((op, Spanned::<i64>::parse(stream)?));
        }

        Ok(Expr {
            first,
            rest
        })
    }
}

// ── entry ─────────────────────────────────────────────────────────────────────

fn eval(input: &str) -> Result<i64, String> {
    let source = Source::new(input);

    let tokens = ArithLexer.lex(input).map_err(|errs| {
        errs.iter()
            .map(|e| format!("unexpected '{}' at {}", e.ch, e.span))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let mut stream = TokenStream::new(tokens);
    let expr = Expr::parse(&mut stream).map_err(|e| e.render(&source))?;

    if !stream.is_at_end() {
        return Err(ParseError::new(stream.peek().span(), "unexpected token").with_note("expected end of input").render(&source))
    }

    Ok(expr.eval())
}

fn main() {
    let cases = ["1 + 2", "10 - 3 + 5", "42", "1 + ", "1 @ 2"];

    for input in cases {
        print!("{input:12} => ");

        match eval(input) {
            Ok(n) => println!("{n}"),
            Err(e) => println!("\n{e}")
        }
    }
}