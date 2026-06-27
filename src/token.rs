use crate::Span;

/// The interface every token type must satisfy.
///
/// Implement this on your token struct to connect it with [`TokenStream`].
/// syntia never defines what a "token kind" looks like — that's yours to decide.
///
/// [`TokenStream`]: crate::parser::TokenStream
///
/// # Example
///
/// ```rust
/// use syntia::{Span, Token};
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// enum MyKind { Plus, Minus, Eof }
///
/// #[derive(Debug, Clone, Copy)]
/// struct MyToken { kind: MyKind, span: Span }
///
/// impl Token for MyToken {
///     fn span(&self) -> Span { self.span }
///     fn is_eof(&self) -> bool { self.kind == MyKind::Eof }
/// }
/// ```
pub trait Token {
    fn span(&self) -> Span;
    fn is_eof(&self) -> bool;
}