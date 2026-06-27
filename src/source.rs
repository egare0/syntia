use crate::{Pos, Span};

/// A wrapper around a source string.
///
/// Holds the text being parsed and provides span-aware utilities. Passed
/// around by reference — no allocation. The `'src` lifetime ties all the
/// spans and text slices back to the original string.
pub struct Source<'src> {
    inner: &'src str,
}

impl<'src> Source<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { inner: src }
    }

    #[inline]
    pub fn as_str(&self) -> &'src str {
        self.inner
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The text covered by `span`.
    ///
    /// Panics if the span is out of range. In practice spans always come from
    /// this same source, so this shouldn't happen.
    #[inline]
    pub fn slice(&self, span: Span) -> &'src str {
        &self.inner[span.start..span.end]
    }

    /// Compute the line/column position of a byte offset.
    ///
    /// O(n) — only call this when rendering error messages.
    pub fn pos_of(&self, offset: usize) -> Pos {
        let mut line = 1u32;
        let mut col = 1u32;

        for (i, ch) in self.inner.char_indices() {
            if i >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        Pos::new(offset, line, col)
    }

    /// The text of a given line number (1-indexed).
    ///
    /// Returns `None` if out of range.
    pub fn line_text(&self, line: u32) -> Option<&'src str> {
        self.inner.lines().nth((line - 1) as usize)
    }

    /// Create a [`Cursor`] pointing to the start of this source.
    ///
    /// [`Cursor`]: crate::lexer::Cursor
    #[cfg(feature = "lexer")]
    pub fn cursor(&self) -> crate::lexer::Cursor<'src> {
        crate::lexer::Cursor::new(self.inner)
    }
}