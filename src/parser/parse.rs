use crate::Token;
use super::{TokenStream, ParseError};

/// Implement this on AST node types that can be parsed from a token stream.
///
/// # Example
///
/// ```rust,ignore
/// struct Expr { ... }
///
/// impl Parse<MyToken> for Expr {
///     fn parse(stream: &mut TokenStream<MyToken>) -> Result<Self, ParseError> {
///         // consume tokens, build Expr, or return ParseError
///     }
/// }
///
/// // Then call it:
/// let expr = Expr::parse(&mut stream)?;
/// ```
pub trait Parse<T: Token>: Sized {
    fn parse(stream: &mut TokenStream<T>) -> Result<Self, ParseError>;
}