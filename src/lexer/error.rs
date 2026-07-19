use crate::{Source, Span};

/// Implement this on your lexer's error type.
///
/// Two methods — where the error is, and what went wrong. In return you
/// get [`render`], producing the same diagnostic format as
/// `ParseError::render`.
///
/// [`render`]: LexError::render
///
/// # Example
///
/// ```rust
/// use syntia::{Span, Source, lexer::LexError};
///
/// #[derive(Debug)]
/// struct MyLexError { ch: char, span: Span }
///
/// impl LexError for MyLexError {
///     fn span(&self) -> Span { self.span }
///     fn message(&self) -> String {
///         format!("unexpected character `{}`", self.ch)
///     }
/// }
///
/// let source = Source::new("1 @ 2");
/// let err = MyLexError { ch: '@', span: Span::new(2, 3) };
/// let text = err.render(&source);
/// assert!(text.contains("unexpected character"));
/// ```
pub trait LexError {
    /// Where in the source the error occurred.
    fn span(&self) -> Span;

    /// What went wrong, without any position info — that comes from the span.
    fn message(&self) -> String;

    /// Render this error as a human-readable diagnostic string.
    ///
    /// Same output format as `ParseError::render`. syntia never prints
    /// anything — displaying the result is your call.
    fn render(&self, source: &Source<'_>) -> String {
        let message = self.message();
        crate::render::render(
            &crate::render::Diagnostic {
                span: self.span(),
                message: &message,
                found: None,
                expected: &[],
                notes: &[],
                labels: &[],
            },
            source,
        )
    }
}