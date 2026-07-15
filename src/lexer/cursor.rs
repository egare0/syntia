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
/// // eat_while returns the span covering everything consumed
/// let word_span = cursor.eat_while(|c| c.is_alphabetic()); // covers "hello"
/// cursor.eat_whitespace();
/// let num_span = cursor.eat_digits(10).unwrap();            // covers "123"
/// ```
pub struct Cursor<'src> {
    source: &'src str,
    offset: usize,
    line: u32,
    col: u32,
}

impl<'src> Cursor<'src> {
    #[must_use]
    pub fn new(source: &'src str) -> Self {
        Self { source, offset: 0, line: 1, col: 1 }
    }

    /// The current byte offset into the source.
    #[inline]
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The current position with line and column numbers.
    #[inline]
    #[must_use]
    pub fn pos(&self) -> Pos {
        Pos::new(self.offset, self.line, self.col)
    }

    /// Peek at the next character without consuming it.
    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    /// Peek `n` characters ahead without consuming (0 = same as `peek`).
    ///
    /// Worst case O(n), but pure-ASCII input takes a cheap byte-scan path
    /// instead of full UTF-8 decoding.
    #[must_use]
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        let rest = &self.source.as_bytes()[self.offset..];

        // Fast path: if everything up to and including byte n in ASCII,
        // byte n is exactly character n.
        if let Some(window) = rest.get(..=n) && window.is_ascii() {
            return Some(window[n] as char);
        }

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
    #[must_use]
    pub fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    /// Build a span from `start` to the current offset
    #[inline]
    #[must_use]
    pub fn span_from(&self, start: usize) -> Span {
        Span::new(start, self.offset)
    }

    /// Consume characters while `pred` returns `true`, returning the span covered.
    ///
    /// If nothing matches, returns an empty span at the current offset.
    pub fn eat_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Span {
        let start = self.offset;

        while self.peek().is_some_and(&mut pred) {
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

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Skip whitespace (Unicode `White_Space`, which includes space, tab,
    /// `\r`, and `\n`).
    pub fn eat_whitespace(&mut self) -> Span {
        self.eat_while(char::is_whitespace)
    }

    /// Consume an identifier: starts with a letter or `_`, followed by
    /// letters, digits, or `_`.
    ///
    /// Uses Unicode alphabetic/alphanumeric rules, so non-ASCII letters
    /// are accepted. Returns `None` if the cursor isn't at the start of
    /// a valid identifier.
    pub fn eat_ident(&mut self) -> Option<Span> {
        if !self.peek().is_some_and(|c| c.is_alphabetic() || c == '_') {
            return None;
        }

        Some(self.eat_while(|c| c.is_alphanumeric() || c == '_'))
    }

    /// Consume digits in the given radix (2, 8, 10, or 16).
    ///
    /// Returns `None` if the cursor isn't at a digit valid for `radix`.
    /// Does not handle prefixes like `0x` or `0b` — consume those yourself
    /// before calling this.
    pub fn eat_digits(&mut self, radix: u32) -> Option<Span> {
        if !self.peek().is_some_and(|c| c.is_digit(radix)) {
            return None;
        }

        Some(self.eat_while(|c| c.is_digit(radix)))
    }

    /// Consume a `//` line comment through to (but not including) the newline.
    ///
    /// Returns `None` if the next two characters aren't `//`.
    pub fn eat_line_comment(&mut self) -> Option<Span> {
        if self.peek() != Some('/') || self.peek_nth(1) != Some('/') {
            return None;
        }

        let start = self.offset;
        self.advance();
        self.advance();
        self.eat_while(|c| c != '\n');
        Some(self.span_from(start))
    }

    /// Consumes a quoted string delimited by `delim`.
    ///
    /// Handles backslash escape sequences — the character after `\` is
    /// consumed without being checked.
    ///
    /// Returns:
    /// - `None` if the cursor isn't at `delim` (not a quoted string here).
    /// - `Some(Ok(span))` if the string was fully consumed, including both delimiters.
    /// - `Some(Err(span))` if the string started but was never closed
    ///   (reached EOF). The span covers everything from the opening delimiter.
    pub fn eat_quoted(&mut self, delim: char) -> Option<Result<Span, Span>> {
        if self.peek() != Some(delim) {
            return None;
        }

        let start = self.offset;
        self.advance(); // opening delimiter

        loop {
            match self.advance() {
                None => return Some(Err(self.span_from(start))), // unterminated
                Some('\\') => { self.advance(); }, // skip escaped char
                Some(c) if c == delim => break,
                _ => {}
            }
        }

        Some(Ok(self.span_from(start)))
    }
}