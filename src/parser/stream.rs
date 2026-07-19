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
    /// The vec must be non-empty and end with an EOF sentinel. Debug builds
    /// assert this; release builds trust the caller.
    #[must_use]
    pub fn new(tokens: Vec<T>) -> Self {
        debug_assert!(tokens.last().is_some_and(Token::is_eof), "TokenStream requires a non-empty token vec ending with an EOF sentinel");
        Self { tokens, pos: 0 }
    }

    /// The next token without consuming it.
    ///
    /// Once the stream reaches EOF, this always returns the EOF token.
    #[inline]
    #[must_use]
    pub fn peek(&self) -> &T {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    /// Look `n` tokens ahead without consuming (0 is the same as `peek`).
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn is_at_end(&self) -> bool {
        self.peek().is_eof()
    }

    /// Save the current position for potential backtracking.
    #[inline]
    #[must_use]
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
    ///     s.expect(|t| t.kind == Kind::Plus, "+")?;
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

    /// Collect results from repeated calls to `f` until it fails.
    ///
    /// Never returns an error — if `f` fails on the first call, returns an
    /// empty vec. Backtracks automatically on failure.
    ///
    /// # Infinite loop guard
    ///
    /// If `f` succeeds without consuming any tokens, `many` stops immediately.
    /// Make sure `f` always consumes at least one token when it returns `Ok`.
    pub fn many<R>(&mut self, mut f: impl FnMut(&mut Self) -> Result<R, ParseError>) -> Vec<R> {
        let mut results = Vec::new();

        loop {
            let checkpoint = self.checkpoint();

            if let Ok(r) = f(self) {
                results.push(r);

                // If f succeeded but consumed nothing, we'd loop forever.
                if self.checkpoint() == checkpoint {
                    break;
                }
            } else {
                self.restore(checkpoint);
                break;
            }
        }

        results
    }

    /// Like [`many`], but requires at least one item.
    ///
    /// Returns the first parse's error if it fails; after that, behaves
    /// exactly like `many` (stops on the first failure, backtracks).
    ///
    /// [`many`]: TokenStream::many
    pub fn many1<R>(&mut self, mut f: impl FnMut(&mut Self) -> Result<R, ParseError>) -> Result<Vec<R>, ParseError> {
        let mut results = vec![f(self)?];
        results.append(&mut self.many(f));
        Ok(results)
    }

    /// Try `f` once; return `Some` on success, `None` on failure.
    ///
    /// Backtracks automatically if `f` returns `Err`.
    pub fn optional<R>(&mut self, f: impl FnOnce(&mut Self) -> Result<R, ParseError>) -> Option<R> {
        let checkpoint = self.checkpoint();

        if let Ok(r) = f(self) { Some(r) } else {
            self.restore(checkpoint);
            None
        }
    }

    /// Parse a sequence of items separated by tokens matching `sep`.
    ///
    /// Requires at least one item — returns `f`'s error if the first
    /// parse fails. Does not consume a trailing separator.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // parses: expr ("," expr)*
    /// let args = stream.separated_by(
    ///     |s| Expr::parse(s),
    ///     |t: &MyToken| t.kind == MyKind::Comma,
    /// )?;
    /// ```
    pub fn separated_by<R>(&mut self, mut item: impl FnMut(&mut Self) -> Result<R, ParseError>, mut sep: impl FnMut(&T) -> bool) -> Result<Vec<R>, ParseError> {
        let mut results = vec![item(self)?];

        loop {
            // Save before consuming the separator so we can backtrack
            // if the next item fails (handles trailing separators cleanly).
            let checkpoint = self.checkpoint();

            if !sep(self.peek()) {
                break;
            }

            self.advance(); // consume separator

            if let Ok(r) = item(self) { results.push(r) } else {
                self.restore(checkpoint); // put separator back
                break;
            }
        }

        Ok(results)
    }
}

impl<T: Token + Copy> TokenStream<T> {
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
    pub fn expect(&mut self, pred: impl FnOnce(&T) -> bool, label: impl Into<String>) -> Result<T, ParseError> {
        if pred(self.peek()) {
            Ok(*self.advance())
        } else {
            let span = self.peek().span();
            Err(ParseError::new(span, format!("expected `{}`", label.into())))
        }
    }
}