/// A range of bytes in the source text.
///
/// `start` and `end` are byte offsets, not character indices. To get the
/// actual text a span covers, use [`Source::slice`].
///
/// [`Source::slice`]: crate::Source::slice
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "Span: start ({start}) > end ({end})");
        Self { start, end }
    }

    /// A zero-width span at a single byte offset.
    ///
    /// Useful for EOF tokens or point diagnostics where there's no actual range.
    #[inline]
    #[must_use]
    pub fn point(offset: usize) -> Self {
        Self { start: offset, end: offset }
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extend this span to also cover `other`, returning the union of both.
    #[inline]
    #[must_use]
    pub fn merge(self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Whether a byte offset falls within this span (start-inclusive, end-exclusive).
    #[inline]
    #[must_use]
    pub fn contains_offset(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A position in source text with line and column numbers.
///
/// Both `line` and `col` are 1-indexed. Compute this from a byte offset
/// via [`Source::pos_of`] — only do so for error reporting, not in hot paths.
///
/// [`Source::pos_of`]: crate::Source::pos_of
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub offset: usize,
    pub line: u32,
    pub col: u32,
}

impl Pos {
    #[inline]
    #[must_use]
    pub fn new(offset: usize, line: u32, col: u32) -> Self {
        Self { offset, line, col }
    }

    /// The start of a source file: offset 0, line 1, column 1.
    #[inline]
    #[must_use]
    pub fn start() -> Self {
        Self { offset: 0, line: 1, col: 1 }
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}