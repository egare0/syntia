use syntia::{Source, Span, Spanned, Token};

// ── a minimal token type for test ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum K {
    Num,
    Plus,
    Minus,
    Eof
}

#[derive(Debug, Clone, Copy)]
struct Tok {
    kind: K,
    span: Span,
}

impl Token for Tok {
    fn span(&self) -> Span {
        self.span
    }

    fn is_eof(&self) -> bool {
        self.kind == K::Eof
    }
}

fn tok(kind: K, start: usize, end: usize) -> Tok {
    Tok { kind, span: Span::new(start, end) }
}

fn eof(at: usize) -> Tok {
    Tok { kind: K::Eof, span: Span::point(at) }
}

// ── Span ─────────────────────────────────────────────────────────────────────

#[test]
fn span_len() {
    assert_eq!(Span::new(2, 7).len(), 5);
}

#[test]
fn span_is_empty() {
    assert!(Span::point(3).is_empty());
    assert!(!Span::new(1, 4).is_empty());
}

#[test]
fn span_merge() {
    assert_eq!(Span::new(0, 5).merge(Span::new(3, 10)), Span::new(0, 10));
}

#[test]
fn span_contains_offset() {
    let s = Span::new(2, 6);
    assert!(s.contains_offset(2));
    assert!(s.contains_offset(5));
    assert!(!s.contains_offset(6)); // end exclusive
}

// ── Source ───────────────────────────────────────────────────────────────────

#[test]
fn source_slice() {
    let src = Source::new("hello world");
    assert_eq!(src.slice(Span::new(6, 11)), "world");
}

#[test]
fn source_pos_of_start() {
    let pos = Source::new("hello").pos_of(0);
    assert_eq!((pos.line, pos.col), (1, 1));
}

#[test]
fn source_pos_after_newline() {
    let pos = Source::new("hello\nworld").pos_of(6); // 'w'
    assert_eq!((pos.line, pos.col), (2, 1));
}

#[test]
fn source_line_text() {
    let src = Source::new("line one\nline two\nline three");
    assert_eq!(src.line_text(1), Some("line one"));
    assert_eq!(src.line_text(2), Some("line two"));
    assert_eq!(src.line_text(99), None);
}

// ── Spanned ──────────────────────────────────────────────────────────────────

#[test]
fn spanned_new() {
    let s = Spanned::new(42u32, Span::new(0, 2));
    assert_eq!(s.node, 42);
    assert_eq!(s.span, Span::new(0, 2));
}

#[test]
fn spanned_map_preserves_span() {
    let s = Spanned::new(10u32, Span::new(0, 2));
    let doubled = s.map(|n| n * 2);
    assert_eq!(doubled.node, 20);
    assert_eq!(doubled.span, Span::new(0, 2));
}

// ── Cursor ───────────────────────────────────────────────────────────────────

#[cfg(feature = "lexer")]
mod cursor_tests {
    use syntia::Source;

    #[test]
    fn advance_and_peek() {
        let src = Source::new("ab");
        let mut c = src.cursor();
        assert_eq!(c.peek(), Some('a'));
        c.advance();
        assert_eq!(c.peek(), Some('b'));
        c.advance();
        assert!(c.is_at_end());
    }

    #[test]
    fn eat_while_span() {
        let src = Source::new("123abc");
        let mut c = src.cursor();
        let span = c.eat_while(|ch| ch.is_ascii_digit());
        assert_eq!(src.slice(span), "123");
    }

    #[test]
    fn eat_whitespace() {
        let src = Source::new("   hello");
        let mut c = src.cursor();
        let span = c.eat_whitespace();
        assert_eq!(span.len(), 3);
        assert_eq!(c.peek(), Some('h'));
    }

    #[test]
    fn eat_ident_ok() {
        let src = Source::new("_foo bar");
        let mut c = src.cursor();
        let span = c.eat_ident().unwrap();
        assert_eq!(src.slice(span), "_foo");
    }

    #[test]
    fn eat_ident_none_on_digit() {
        let src = Source::new("123");
        assert!(src.cursor().eat_ident().is_none());
    }

    #[test]
    fn eat_digits_decimal() {
        let src = Source::new("42rest");
        let span = src.cursor().eat_digits(10).unwrap();
        assert_eq!(src.slice(span), "42");
    }

    #[test]
    fn eat_digits_hex() {
        let src = Source::new("ff1g");
        let span = src.cursor().eat_digits(16).unwrap();
        assert_eq!(src.slice(span), "ff1");
    }

    #[test]
    fn eat_digits_none_on_no_match() {
        assert!(Source::new("xyz").cursor().eat_digits(10).is_none());
    }

    #[test]
    fn eat_line_comment_ok() {
        let src = Source::new("// hello\nnext");
        let mut c = src.cursor();
        let span = c.eat_line_comment().unwrap();
        assert_eq!(src.slice(span), "// hello");
        assert_eq!(c.peek(), Some('\n'));
    }

    #[test]
    fn eat_line_comment_none_on_no_match() {
        assert!(Source::new("hello").cursor().eat_line_comment().is_none());
    }

    #[test]
    fn eat_quoted_ok() {
        let src = Source::new(r#""hello""#);
        let span = src.cursor().eat_quoted('"').unwrap().unwrap();
        assert_eq!(src.slice(span), r#""hello""#);
    }

    #[test]
    fn eat_quoted_with_escape() {
        let src = Source::new(r#""hel\"lo""#);
        let span = src.cursor().eat_quoted('"').unwrap().unwrap();
        assert_eq!(src.slice(span), r#""hel\"lo""#);
    }

    #[test]
    fn eat_quoted_unterminated() {
        let result = Source::new(r#""oops"#).cursor().eat_quoted('"').unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn eat_quoted_none_on_no_delimiter() {
        assert!(Source::new("hello").cursor().eat_quoted('"').is_none());
    }

    #[test]
    fn line_tracking() {
        let src = Source::new("a\nb");
        let mut c = src.cursor();
        c.advance(); // 'a'
        assert_eq!(c.pos().line, 1);
        c.advance(); // '\n'
        assert_eq!(c.pos().line, 2);
        assert_eq!(c.pos().col, 1);
    }

    #[test]
    fn span_from() {
        let src = Source::new("hello world");
        let mut c = src.cursor();
        let start = c.offset();
        for _ in 0..5 { c.advance(); }
        assert_eq!(src.slice(c.span_from(start)), "hello");
    }
}

// ── TokenStream ──────────────────────────────────────────────────────────────

#[cfg(feature = "parser")]
mod stream_tests {
    use syntia::parser::TokenStream;
    use syntia::Span;
    use super::{K, tok, eof};

    fn stream() -> TokenStream<super::Tok> {
        // represents: "1 + 2"
        TokenStream::new(vec![
            tok(K::Num,  0, 1),
            tok(K::Plus, 2, 3),
            tok(K::Num,  4, 5),
            eof(5),
        ])
    }

    #[test]
    fn peek_does_not_consume() {
        let s = stream();
        assert_eq!(s.peek().kind, K::Num);
        assert_eq!(s.peek().kind, K::Num);
    }

    #[test]
    fn advance_consumes() {
        let mut s = stream();
        assert_eq!(s.advance().kind, K::Num);
        assert_eq!(s.advance().kind, K::Plus);
        assert_eq!(s.advance().kind, K::Num);
        assert!(s.is_at_end());
    }

    #[test]
    fn peek_nth() {
        let s = stream();
        assert_eq!(s.peek_nth(0).kind, K::Num);
        assert_eq!(s.peek_nth(1).kind, K::Plus);
        assert_eq!(s.peek_nth(2).kind, K::Num);
    }

    #[test]
    fn checkpoint_restore() {
        let mut s = stream();
        let cp = s.checkpoint();
        s.advance();
        s.advance();
        s.restore(cp);
        assert_eq!(s.peek().kind, K::Num);
    }

    #[test]
    fn try_parse_backtracks_on_err() {
        let mut s = stream();
        let _: Result<_, _> = s.try_parse(|s| {
            s.advance();
            s.expect(|t| t.kind == K::Minus, "-") // Plus is next, fails
        });
        assert_eq!(s.peek().kind, K::Num); // back to start
    }

    #[test]
    fn expect_ok() {
        let mut s = stream();
        let tok = s.expect(|t| t.kind == K::Num, "num").unwrap();
        assert_eq!(tok.span, Span::new(0, 1));
        assert_eq!(s.peek().kind, K::Plus);
    }

    #[test]
    fn expect_err_has_expected_prefix() {
        let mut s = stream();
        s.advance(); // consume Num
        let err = s.expect(|t| t.kind == K::Minus, "-").unwrap_err();
        assert!(err.to_string().contains("expected"));
    }

    #[test]
    fn many_stops_on_mismatch() {
        let mut s = stream();
        let nums = s.many(|s| s.expect(|t| t.kind == K::Num, "num"));
        assert_eq!(nums.len(), 1); // only first Num, stops at Plus
        assert_eq!(s.peek().kind, K::Plus);
    }

    #[test]
    fn many_empty_on_immediate_mismatch() {
        let mut s = stream();
        let pluses = s.many(|s| s.expect(|t| t.kind == K::Plus, "+"));
        assert!(pluses.is_empty());
        assert_eq!(s.peek().kind, K::Num); // unchanged
    }

    #[test]
    fn optional_some() {
        let mut s = stream();
        assert!(s.optional(|s| s.expect(|t| t.kind == K::Num, "num")).is_some());
    }

    #[test]
    fn optional_none_backtracks() {
        let mut s = stream();
        assert!(s.optional(|s| s.expect(|t| t.kind == K::Minus, "-")).is_none());
        assert_eq!(s.peek().kind, K::Num);
    }

    #[test]
    fn separated_by_collects_all() {
        let mut s = TokenStream::new(vec![
            tok(K::Num, 0, 1), tok(K::Plus, 2, 3),
            tok(K::Num, 4, 5), tok(K::Plus, 6, 7),
            tok(K::Num, 8, 9), eof(9),
        ]);
        let nums = s.separated_by(
            |s| s.expect(|t| t.kind == K::Num, "`num`"),
            |t| t.kind == K::Plus,
        ).unwrap();
        assert_eq!(nums.len(), 3);
        assert!(s.is_at_end());
    }

    #[test]
    fn separated_by_does_not_consume_trailing_sep() {
        // Num Plus Num Plus Eof — trailing Plus must stay
        let mut s = TokenStream::new(vec![
            tok(K::Num, 0, 1), tok(K::Plus, 2, 3),
            tok(K::Num, 4, 5), tok(K::Plus, 6, 7),
            eof(7),
        ]);
        let nums = s.separated_by(
            |s| s.expect(|t| t.kind == K::Num, "num"),
            |t| t.kind == K::Plus,
        ).unwrap();
        assert_eq!(nums.len(), 2);
        assert_eq!(s.peek().kind, K::Plus); // still there
    }
}

// ── ParseError ───────────────────────────────────────────────────────────────

#[cfg(feature = "parser")]
mod error_tests {
    use syntia::{Source, Span};
    use syntia::parser::ParseError;

    #[test]
    fn render_contains_message() {
        let src = Source::new("let x = }");
        let rendered = ParseError::new(Span::new(8, 9), "unexpected token")
            .with_found("`}`")
            .render(&src);
        assert!(rendered.contains("unexpected token"));
        assert!(rendered.contains("found"));
        assert!(rendered.contains("}"));
    }

    #[test]
    fn render_shows_correct_line() {
        let src = Source::new("line1\nline2\nlet x = }");
        let rendered = ParseError::new(Span::new(19, 20), "unexpected")
            .render(&src);
        assert!(rendered.contains("3:")); // third line
    }

    #[test]
    fn with_note_appears_in_render() {
        let src = Source::new("1abc");
        let rendered = ParseError::new(Span::new(0, 4), "bad token")
            .with_note("numbers cannot contain letters")
            .render(&src);
        assert!(rendered.contains("note:"));
        assert!(rendered.contains("numbers cannot contain letters"));
    }

    #[test]
    fn display_contains_message() {
        let err = ParseError::new(Span::point(0), "oops");
        assert!(err.to_string().contains("oops"));
    }

    #[test]
    fn implements_std_error() {
        let err = ParseError::new(Span::point(0), "test");
        let _: &dyn std::error::Error = &err;
    }
}