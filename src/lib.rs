//! syntia — a lightweight toolkit for building parsers.

mod span;
pub use span::{Span, Pos};

mod source;
pub use source::Source;

mod token;
pub use token::Token;

/// Lexer utilities: [`Cursor`] and the [`Lex`] trait.
///
/// Enable with the `lexer` feature (on by default).
///
/// [`Cursor`]: lexer::Cursor
/// [`Lex`]: lexer::Lex
#[cfg(feature = "lexer")]
pub mod lexer;