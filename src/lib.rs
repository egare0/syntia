//! syntia is a lightweight toolkit for building lexers and parsers in Rust.
//!
//! # The three ideas
//!
//! **Phase separation.** Lexing and parsing are distinct steps, connected by
//! the [`Token`] trait. The `lexer` and `parser` features are independent —
//! use only one if that's all you need.
//!
//! **Spans, not strings.** Tokens hold byte offsets ([`Span`]), not `&str` or
//! `String`. To get the text a token covers, ask the source:
//!
//! ```rust
//! # use syntia::{Source, Span};
//! # let source = Source::new("hello");
//! # let span = Span::new(0, 5);
//! let text = source.slice(span); // &str, no allocation
//! ```
//!
//! Keeping tokens free of string data means they stay `Copy` and cheap to
//! pass around. Wrap them in [`Spanned<T>`] when you need to attach a span
//! to an AST node.
//!
//! **Errors as values.** [`ParseError`] is a plain struct. Call [`render`] to
//! get a diagnostic string — syntia never writes to stderr.
//!
//! # Feature flags
//!
//! | Feature   | Contents                                           | Default |
//! |-----------|----------------------------------------------------|---------|
//! | `lexer`   | [`Cursor`] and helpers, [`Lex`] trait              | yes     |
//! | `parser`  | [`TokenStream`] with combinators, [`ParseError`],  | yes     |
//! |           | [`Parse`] trait                                    |         |
//!
//! Enable only `parser` if you're bringing your own lexer:
//!
//! ```toml
//! syntia = { version = "0.1", default-features = false, features = ["parser"] }
//! ```
//!
//! # Quick start
//!
//! ```rust
//! use syntia::{Span, Source, Spanned, Token};
//!
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! enum Kind { Number, Plus, Eof }
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Tok { kind: Kind, span: Span }
//!
//! impl Token for Tok {
//!     fn span(&self) -> Span { self.span }
//!     fn is_eof(&self) -> bool { self.kind == Kind::Eof }
//! }
//!
//! let src = Source::new("1 + 2");
//! let tokens = vec![
//!     Tok { kind: Kind::Number, span: Span::new(0, 1) },
//!     Tok { kind: Kind::Plus,   span: Span::new(2, 3) },
//!     Tok { kind: Kind::Number, span: Span::new(4, 5) },
//!     Tok { kind: Kind::Eof,    span: Span::point(5)  },
//! ];
//!
//! # #[cfg(feature = "parser")]
//! # {
//! use syntia::parser::TokenStream;
//!
//! let mut stream = TokenStream::new(tokens);
//!
//! let lhs = stream.advance();
//! stream.expect(|t: &Tok| t.kind == Kind::Plus, "`+`").unwrap();
//! let rhs = stream.advance();
//!
//! // Attach the full expression span to the computed result.
//! let result = Spanned::new(3_i64, lhs.span().merge(rhs.span()));
//!
//! assert_eq!(result.node, 3);
//! assert_eq!(src.slice(result.span), "1 + 2");
//! # }
//! ```
//!
//! [`render`]: parser::ParseError::render
//! [`Cursor`]: lexer::Cursor
//! [`Lex`]: lexer::Lex
//! [`TokenStream`]: parser::TokenStream
//! [`ParseError`]: parser::ParseError
//! [`Parse`]: parser::Parse

mod span;
pub use span::{Span, Pos};

mod source;
pub use source::Source;

mod token;
pub use token::Token;

mod spanned;
pub use spanned::Spanned;

/// Lexer utilities.
///
/// [`Cursor`] walks source text character by character, tracking byte offset,
/// line, and column. It comes with helpers for common lexer patterns —
/// [`eat_ident`], [`eat_digits`], [`eat_whitespace`], [`eat_line_comment`],
/// and [`eat_quoted`] — so you're not rewriting the same `eat_while` calls
/// for every lexer you build.
///
/// Implement [`Lex`] on your lexer struct to connect it with the rest of
/// the library.
///
/// Enable with the `lexer` Cargo feature (on by default).
///
/// [`Cursor`]: lexer::Cursor
/// [`Lex`]: lexer::Lex
/// [`eat_ident`]: lexer::Cursor::eat_ident
/// [`eat_digits`]: lexer::Cursor::eat_digits
/// [`eat_whitespace`]: lexer::Cursor::eat_whitespace
/// [`eat_line_comment`]: lexer::Cursor::eat_line_comment
/// [`eat_quoted`]: lexer::Cursor::eat_quoted
#[cfg(feature = "lexer")]
pub mod lexer;

/// Parser utilities.
///
/// [`TokenStream`] wraps the token vec from your lexer. Beyond basic
/// peek/advance, it provides [`many`], [`optional`], and [`separated_by`]
/// for common parse patterns, and [`try_parse`] for explicit backtracking.
///
/// [`expect`] takes a label describing what was expected — "expected" is
/// prepended automatically, so pass `` "`fn`" `` rather than
/// `"expected `fn`"`.
///
/// [`Parse`] is the trait for AST nodes. [`ParseError`] carries span and
/// context; call [`render`] for a human-readable diagnostic string.
///
/// Enable with the `parser` Cargo feature (on by default). Does not depend
/// on the `lexer` feature — you can bring your own lexer.
///
/// [`TokenStream`]: parser::TokenStream
/// [`many`]: parser::TokenStream::many
/// [`optional`]: parser::TokenStream::optional
/// [`separated_by`]: parser::TokenStream::separated_by
/// [`try_parse`]: parser::TokenStream::try_parse
/// [`expect`]: parser::TokenStream::expect
/// [`Parse`]: parser::Parse
/// [`ParseError`]: parser::ParseError
/// [`render`]: parser::ParseError::render
#[cfg(feature = "parser")]
pub mod parser;

#[cfg(any(feature = "lexer", feature = "parser"))]
mod render;