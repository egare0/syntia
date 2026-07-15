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

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected.push(expected.into());
        self
    }

    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }

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
    pub fn render(&self, source: &Source<'_>) -> String {
        let start = source.pos_of(self.span.start);
        let line_text = source.line_text(start.line).unwrap_or("");
        let line_num_str = start.line.to_string();
        // Gutter width = width of the line number string, minimum 2 chars.
        let gutter = line_num_str.len().max(2);
        let pad = " ".repeat(gutter);

        let mut out = String::new();

        // error: <message>
        out.push_str(&format!("error: {}\n", self.message));

        // --> <line>:<col>
        out.push_str(&format!("{pad}--> {}:{}\n", start.line, start.col));

        // gutter separator
        out.push_str(&format!("{pad} |\n"));

        // line number + source line
        out.push_str(&format!("{:>gutter$} | {line_text}\n", start.line));

        // underline
        let col0 = (start.col as usize).saturating_sub(1); // 0-indexed column

        // Count in characters, not bytes, so the underline stays aligned when
        // the line contains multibyte characters.
        let span_chars = source.slice(self.span).split('\n').next().map_or(0, |s| s.chars().count());
        let available = line_text.chars().count().saturating_sub(col0);
        let hat_count = span_chars.min(available).max(1);
        let hats = "^".repeat(hat_count);

        let label = match &self.found {
            Some(f) => format!(" found `{f}`"),
            None => String::new(),
        };

        out.push_str(&format!("{pad} | {}{hats}{label}\n", " ".repeat(col0)));

        // closing separator
        out.push_str(&format!("{pad} |\n"));

        // expected hints
        if !self.expected.is_empty() {
            let joined = self.expected.join("`, `");
            out.push_str(&format!("{pad} = expected `{joined}`\n"));
        }

        // notes
        for note in &self.notes {
            out.push_str(&format!("{pad} = note: {note}\n"));
        }

        out
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}