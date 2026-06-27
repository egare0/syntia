use crate::Token;
use super::error::ParseError;

/// A peekable, restartable stream of tokens.
///
/// Feed it the output of your lexer and use it in your parser to consume
/// tokens one by one.
///
/// # Invariant
///
/// The `tokens` vec must be non-empty and with an EOF sentinel (a token
/// where [`Token::is_eof`] returns `true`). `Lex` implementations are expected
/// to uphold this. Violating it causes panics in `peek` and `advance`.
pub struct TokenStream<T> {
    tokens: Vec<T>,
    pos: usize
}

impl<T: Token> TokenStream<T> {
    /// Create a stream from a vec of tokens.
    ///
    /// The vec must be non-empty and end with an EOF sentinel.
    pub fn new(tokens: Vec<T>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// The next token without consuming it.
    ///
    /// Once the stream reaches EOF, this always returns the EOF token.
    #[inline]
    pub fn peek(&self) -> &T {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    /// Look `n` tokens ahead without consuming (0 is the same as `peek`).
    #[inline]
    pub fn peek_nth(&self, n: usize) -> &T {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    /// Consume and return the next token.
    ///
    /// Once the stream is exhausted, returns the EOF token on every call.
    pub fn advance(&mut self) -> &T {
        let pos = self.pos.min(self.tokens.len() - 1);

        if self.pos < self.tokens.len() {
            self.pos += 1;
        }

        &self.tokens[pos]
    }

    /// Whether the next token is an EOF sentinel.
    #[inline]
    pub fn is_at_end(&self) -> bool {
        self.peek().is_eof()
    }

    /// Save the current position for potential backtracking.
    #[inline]
    pub fn checkpoint(&self) -> usize {
        self.pos
    }

    /// Restore the stream to a previously saved checkpoint.
    #[inline]
    pub fn restore(&mut self, checkpoint: usize) {
        self.pos = checkpoint;
    }

    /// Try a parse closure; if it returns `Err`, automatically backtrack to
    /// where the stream was before the call.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = stream.try_parse(|s| {
    ///     let a = s.advance();
    ///     s.expect(|t| t.kind == Kind::Plus, "expected `+`")?;
    ///     let b = s.advance();
    ///     Ok((a, b))
    /// });
    /// ```
    pub fn try_parse<R, E>(&mut self, f: impl FnOnce(&mut Self) -> Result<R, E>) -> Result<R, E> {
        let checkpoint = self.checkpoint();

        match f(self) {
            Ok(r) => Ok(r),
            Err(e) => {
                self.restore(checkpoint);
                Err(e)
            }
        }
    }

    /// Advance if `pred` matches the next token; otherwise return a `ParseError`.
    ///
    /// The `label` parameter describes what was expected — the word "expected"
    /// is prepended automatically. Pass `"fn"` rather than
    /// ``"expected `fn`"``.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// stream.expect(|t: &MyToken| t.kind == MyKind::Semicolon, ";")?;
    /// // On failure: "expected `;`"
    /// ```
    pub fn expect(&mut self, pred: impl FnOnce(&T) -> bool, label: impl Into<String>) -> Result<&T, ParseError> {
        if pred(self.peek()) {
            Ok(self.advance())
        } else {
            let span = self.peek().span();
            Err(ParseError::new(span, format!("expected `{}`", label.into())))
        }
    }
}