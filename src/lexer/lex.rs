use crate::lexer::LexError;
use crate::Token;

/// Implement this on your lexer to wire it into syntia.
///
/// The trait is intentionally thin. You control the token type, the error type,
/// and everything about what lexing means. syntia provides [`Cursor`] to help,
/// but you can produce tokens any way you like — as long as the result
/// implements [`Token`].
///
/// [`Cursor`]: crate::lexer::Cursor
///
/// # Conventions
///
/// - Collect all lex errors rather than bailing on the first. Let users see
///   everything that's wrong at once.
/// - Emit tokens in source order.
/// - The last token in the returned `Vec` must be an EOF sentinel: a token
///   where [`Token::is_eof`] returns `true`. [`TokenStream`] relies on this.
///
/// [`TokenStream`]: crate::parser::TokenStream
///
/// # Errors
///
/// `Error` must implement [`LexError`] — a span and a message. In return,
/// every lex error can be rendered as a diagnostic via [`LexError::render`].
///
/// # Example
///
/// ```rust,ignore
/// struct MyLexer;
///
/// impl Lex for MyLexer {
///     type Token = MyToken;
///     type Error = MyLexError;
///
///     fn lex(&mut self, source: &str) -> Result<Vec<Self::Token>, Vec<Self::Error>> {
///         // use Cursor, build tokens, return
///     }
/// }
/// ```
pub trait Lex {
    type Token: Token;
    type Error: LexError;

    fn lex(&mut self, source: &str) -> Result<Vec<Self::Token>, Vec<Self::Error>>;
}