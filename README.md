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
diagnostic string — syntia never touches stderr. Lex errors work the same
way: implement `LexError` on your error type and get `render` for free, in
the same format.

**Lexing and parsing are separate.** The `Token` trait connects them. The
`lexer` and `parser` features are independent — use only what you need, and
bring your own lexer if you prefer.

## Features

| Feature  | Contents                                              | Default |
|----------|-------------------------------------------------------|---------|
| `lexer`  | `Cursor` with helpers, `Lex` and `LexError` traits    | yes     |
| `parser` | `TokenStream` with combinators, `ParseError`, `Parse` | yes     |

```toml
[dependencies]
syntia = "0.2"

# Or pick what you need:
syntia = { version = "0.2", default-features = false, features = ["parser"] }
```

## Quick start

```rust
use syntia::{Span, Source, Spanned, Token};
use syntia::lexer::{Cursor, Lex, LexError};
use syntia::parser::TokenStream;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind { Number, Plus, Eof }

#[derive(Debug, Clone, Copy)]
struct Tok { kind: Kind, span: Span }

impl Token for Tok {
    fn span(&self) -> Span { self.span }
    fn is_eof(&self) -> bool { self.kind == Kind::Eof }
}

// Lex errors carry a span and a message; rendering comes for free.
#[derive(Debug)]
struct MyLexError { span: Span }

impl LexError for MyLexError {
    fn span(&self) -> Span { self.span }
    fn message(&self) -> String { "unexpected character".into() }
}

// Implement Lex — use Cursor helpers to avoid boilerplate.
struct MyLexer;

impl Lex for MyLexer {
    type Token = Tok;
    type Error = MyLexError;

    fn lex(&mut self, source: &str) -> Result<Vec<Tok>, Vec<MyLexError>> {
        let mut cursor = Cursor::new(source);
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            cursor.eat_whitespace();
            if cursor.is_at_end() {
                tokens.push(Tok { kind: Kind::Eof, span: Span::point(cursor.offset()) });
                break;
            }
            if cursor.eat_if('+') {
                let end = cursor.offset();
                tokens.push(Tok { kind: Kind::Plus, span: Span::new(end - 1, end) });
            } else if let Some(span) = cursor.eat_digits(10) {
                tokens.push(Tok { kind: Kind::Number, span });
            } else {
                let start = cursor.offset();
                cursor.advance();
                errors.push(MyLexError { span: cursor.span_from(start) });
            }
        }

        if errors.is_empty() { Ok(tokens) } else { Err(errors) }
    }
}

// Parse — expect takes a label, "expected" is added automatically.
let src = Source::new("1 + 2");
let tokens = MyLexer.lex(src.as_str()).unwrap();
let mut stream = TokenStream::new(tokens);

// Tokens are Copy — dereference to end the borrow on the stream.
let lhs = *stream.advance();
stream.expect(|t: &Tok| t.kind == Kind::Plus, "`+`").unwrap();
let rhs = *stream.advance();

// Pair the result with the span it came from.
let result = Spanned::new(3_i64, lhs.span().merge(rhs.span()));
assert_eq!(src.slice(result.span), "1 + 2");
```

See the [crate documentation](https://docs.rs/syntia) and
`examples/arithmetic.rs` for a complete walkthrough.

## License

MIT