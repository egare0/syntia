# syntia

A lightweight toolkit for building lexers and parsers in Rust.

syntia doesn't pick a grammar format or parser algorithm for you. It gives you
the building blocks and gets out of the way.

## Design

**Tokens hold [`Span`]s, not strings.** A `Span` is a pair of byte offsets.
When you need the text, ask the source: `source.slice(token.span())`. Tokens
stay `Copy` and allocation-free. Use [`Spanned<T>`] to attach a span to any
AST node without wrapping each type individually.

**Errors are values.** [`ParseError`] has a `render` method that returns a
diagnostic string — syntia never touches stderr.

**Lexing and parsing are separate.** The `Token` trait connects them. The
`lexer` and `parser` features are independent — use only what you need, and
bring your own lexer if you prefer.

## Features

| Feature  | Contents                                              | Default |
|----------|-------------------------------------------------------|---------|
| `lexer`  | `Cursor` with helpers, `Lex` trait                    | yes     |
| `parser` | `TokenStream` with combinators, `ParseError`, `Parse` | yes     |

```toml
[dependencies]
syntia = "0.1"
```
```toml
# Or pick what you need:
syntia = { version = "0.1", default-features = false, features = ["parser"] }
```

## Quick start

```rust
use syntia::{Span, Source, Spanned, Token};
use syntia::lexer::{Cursor, Lex};
use syntia::parser::{Parse, ParseError, TokenStream};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind { Number, Plus, Eof }

#[derive(Debug, Clone, Copy)]
struct Tok { kind: Kind, span: Span }

impl Token for Tok {
    fn span(&self) -> Span { self.span }
    fn is_eof(&self) -> bool { self.kind == Kind::Eof }
}

// Implement Lex — use Cursor helpers to avoid boilerplate.
struct MyLexer;

impl Lex for MyLexer {
    type Token = Tok;
    type Error = ();

    fn lex(&mut self, source: &str) -> Result<Vec<Tok>, Vec<()>> {
        let mut cursor = Cursor::new(source);
        let mut tokens = Vec::new();

        loop {
            cursor.eat_whitespace();
            if cursor.is_at_end() {
                tokens.push(Tok { kind: Kind::Eof, span: Span::point(cursor.offset()) });
                break;
            }
            match cursor.peek() {
                Some('+') => {
                    let start = cursor.offset();
                    cursor.advance();
                    tokens.push(Tok { kind: Kind::Plus, span: cursor.span_from(start) });
                }
                _ => {
                    let span = cursor.eat_digits(10).unwrap_or_else(|| {
                        let s = cursor.offset(); cursor.advance(); cursor.span_from(s)
                    });
                    tokens.push(Tok { kind: Kind::Number, span });
                }
            }
        }
        Ok(tokens)
    }
}

// Implement Parse for our nodes
impl Parse<Tok> for Spanned<i64> {
    fn parse(stream: &mut TokenStream<Tok>) -> Result<Self, ParseError> {
        let tok = stream.expect(|t| t.kind == Kind::Number, "number")?;
        Ok(Spanned::new(0, tok.span)) // Value extraction logic goes here
    }
}

fn main() {
    // Parse — expect takes a label, "expected" is added automatically.
    let src = Source::new("1 + 2");
    let tokens = MyLexer.lex(src.as_str()).unwrap();
    let mut stream = TokenStream::new(tokens);

    let lhs = Spanned::<i64>::parse(&mut stream)?;
    stream.expect(|t| t.kind == Kind::Plus, "`+`").unwrap();
    let rhs = Spanned::<i64>::parse(&mut stream)?;

    // Pair the result with the span it came from.
    let result = Spanned::new(3_i64, lhs.span.merge(rhs.span));
    assert_eq!(src.slice(result.span), "1 + 2");
}
```

See the [crate documentation](https://docs.rs/syntia) and the
`examples/arithmetic.rs` for a complete walkthrough.

## License

MIT