use crate::{Pos, Span};

/// A character-by-character cursor over source text.
///
/// The main tool for writing a lexer. Records byte offset, line and column
/// as you advance, and produce [`Span`]s for the ranges you consume.
///
/// # Example
///
/// ```rust
/// use syntia::{Source, lexer::Cursor};
///
/// let source = Source::new("hello 123");
/// let mut cursor = source.cursor();
///
/// let start = cursor.offset();
/// let word_span = cursor.eat_while(|c| c.is_alphabetic());
/// ```
pub struct Cursor<'src> {
    source: &'src str,
    offset: usize,
    line: u32,
    col: u32,
}

impl<'src> Cursor<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, offset: 0, line: 1, col: 1 }
    }

    /// The current byte offset into the source.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The current position with line and column numbers.
    #[inline]
    pub fn pos(&self) -> Pos {
        Pos::new(self.offset, self.line, self.col)
    }

    /// Peek at the next character without consuming it.
    #[inline]
    pub fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    /// Peek `n` characters ahead without consuming (0 = same as `peek`).
    ///
    /// O(n)  — scans forward through UTF-8 bytes to find the nth character.
    /// Fine for small lookaheads, don't call in a tight loop with large n.
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        self.source[self.offset..].chars().nth(n)
    }

    /// Consume and return the next character, updating position tracking.
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.source[self.offset..].chars().next()?;
        self.offset += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        Some(ch)
    }

    /// Whether the cursor is at the end of the source.
    #[inline]
    pub fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    /// Build a span from `start` to the current offset
    #[inline]
    pub fn span_from(&self, start: usize) -> Span {
        Span::new(start, self.offset)
    }

    /// Consume characters while `pred` returns `true`, returning the span covered.
    ///
    /// If nothing matches, returns an empty span at the current offset.
    pub fn eat_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Span {
        let start = self.offset;

        while self.peek().is_some_and(|c| pred(c)) {
            self.advance();
        }

        self.span_from(start)
    }

    /// Consume the next character only if it equals `expected`.
    /// Returns whether it was consumed.
    #[inline]
    pub fn eat_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
}