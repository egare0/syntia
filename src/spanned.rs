use crate::Span;

/// Pairs any value with the source span it came from.
///
/// Use this to carry span information through your AST without building a
/// custom wrapper for each node type.
///
/// # Example
/// ```rust
/// use syntia::{Span, Spanned};
///
/// // Your AST node
/// #[derive(Debug)]
/// struct Ident(String);
///
/// let node = Spanned::new(Ident("foo".into()), Span::new(0, 3));
/// println!("{:?}", node.span); // Span { start: 0, end: 3 }
/// println!("{}", node.node.0); // foo
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    /// Apply a function to the inner value, keeping the span.
    ///
    /// Useful for transforming nodes during lowering passes without
    /// losing source location.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned { node: f(self.node), span: self.span }
    }

    /// Borrow the inner value, keeping the span.
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned { node: &self.node, span: self.span }
    }
}