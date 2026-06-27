//! syntia — a lightweight toolkit for building parsers.

mod span;
pub use span::{Span, Pos};

mod source;
pub use source::Source;

mod token;
pub use token::Token;