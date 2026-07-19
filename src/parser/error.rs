use crate::{Source, Span};

/// A parse error with span, message, and optional context.
///
/// Build one with [`new`], chain context methods, and call [`render`] when
/// you want a human-readable diagnostic string.
///
/// syntia never prints anything. Rendering is your call.
///
/// [`new`]: ParseError::new
/// [`render`]: ParseError::render
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
    /// What was expected at this position (can be multiple).
    pub expected: Vec<String>,
    /// What was actually found, if known.
    pub found: Option<String>,
    /// Extra notes to show below the diagnostic.
    pub notes: Vec<String>,
}

impl ParseError {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            expected: Vec::new(),
            found: None,
            notes: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected.push(expected.into());
        self
    }

    #[must_use]
    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Render this error as a human-readable diagnostic string.
    ///
    /// Pass the original source text to get line context and underlines.
    /// The output format is similar to rustc's error output.
    ///
    /// # Note on multi-line spans
    ///
    /// If `self.span` crosses a line boundary, the underline is clipped to
    /// the first line. This is a known limitation — good enough for v0.1.
    ///
    /// # Example output
    ///
    /// ```text
    /// error: expected `;`
    ///   --> 3:10
    ///    |
    ///  3 | let x = 5 }
    ///    |           ^ found `}`
    ///    |
    ///    = expected `;`
    /// ```
    #[must_use]
    pub fn render(&self, source: &Source<'_>) -> String {
        crate::render::render(
            crate::render::Diagnostic {
                span: self.span,
                message: &self.message,
                found: self.found.as_deref(),
                expected: &self.expected,
                notes: &self.notes,
            },
            source,
        )
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}