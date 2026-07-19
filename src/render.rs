use crate::{Source, Span};

/// Everything needed to render one diagnostic. Crate-internal -
/// `ParseError` and `LexError` both funnel through here.
pub(crate) struct Diagnostic<'a> {
    pub span: Span,
    pub message: &'a str,
    pub found: Option<&'a str>,
    pub expected: &'a [String],
    pub notes: &'a [String],
}

pub(crate) fn render(diag: Diagnostic<'_>, source: &Source<'_>) -> String {
    let start = source.pos_of(diag.span.start);
    let line_text = source.line_text(start.line).unwrap_or("");
    let gutter = start.line.to_string().len().max(2);
    let pad = " ".repeat(gutter);


    let mut out = String::new();
    out.push_str(&format!("error: {}\n", diag.message));
    out.push_str(&format!("{pad}--> {}:{}\n", start.line, start.col));
    out.push_str(&format!("{pad} |\n"));
    out.push_str(&format!("{:>gutter$} | {line_text}\n", start.line));

    let col0 = (start.col as usize).saturating_sub(1);

    // Count in characters, not bytes, so the underline stays aligned when
    // the line contains multibyte characters.
    let span_chars = source.slice(diag.span).split('\n').next().map_or(0, |s| s.chars().count());
    let available = line_text.chars().count().saturating_sub(col0);
    let count = span_chars.min(available).max(1);

    let label = match diag.found {
        Some(f) => format!(" found `{f}`"),
        None => String::new(),
    };

    out.push_str(&format!(
        "{pad} | {}{}{label}\n",
        " ".repeat(col0),
        "^".repeat(count),
    ));

    out.push_str(&format!("{pad} |\n"));

    if !diag.expected.is_empty() {
        let joined = diag.expected.join("`, `");
        out.push_str(&format!("{pad} = expected `{joined}`\n"));
    }

    for note in diag.notes {
        out.push_str(&format!("{pad} = note: {note}\n"));
    }

    out
}