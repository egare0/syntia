use crate::{Source, Span};

/// Everything needed to render one diagnostic. Crate-internal -
/// `ParseError` and `LexError` both funnel through here.
pub(crate) struct Diagnostic<'a> {
    pub span: Span,
    pub message: &'a str,
    pub found: Option<&'a str>,
    pub expected: &'a [String],
    pub notes: &'a [String],
    pub labels: &'a [(Span, String)],
}

pub(crate) fn render(diag: Diagnostic<'_>, source: &Source<'_>) -> String {
    let start = source.pos_of(diag.span.start);

    // Gutter width: widest line number among the primary span and all labels.
    let max_line = diag.labels.iter().map(|(span, _)| source.pos_of(span.start).line).fold(start.line, u32::max);
    let gutter = max_line.to_string().len().max(2);
    let pad = " ".repeat(gutter);

    let mut out = String::new();

    out.push_str(&format!("error: {}\n", diag.message));
    out.push_str(&format!("{pad}--> {}:{}\n", start.line, start.col));
    out.push_str(&format!("{pad} |\n"));

    // Primary span, marked with ^.
    let found = diag.found.map(|f| format!(" found `{f}`"));
    push_snippet(&mut out, source, diag.span, '^', found, gutter);

    // Secondary labels, marked with -.
    for (span, msg) in diag.labels {
        out.push_str(&format!("{pad} |\n"));
        push_snippet(&mut out, source, *span, '-', Some(format!(" {msg}")), gutter);
    }

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

/// One source line plus its underline. `mark` is `^` for the primary
/// span and `-` for secondary labels.
fn push_snippet(out: &mut String, source: &Source<'_>, span: Span, mark: char, label: Option<String>, gutter: usize) {
    let start = source.pos_of(span.start);
    let line_text = source.line_text(start.line).unwrap_or("");
    let pad = " ".repeat(gutter);

    let col0 = (start.col as usize).saturating_sub(1);

    // Character-based counting keeps the underline aligned on
    // multibyte input.
    let span_chars = source.slice(span).split('\n').next().map_or(0, |s| s.chars().count());
    let available = line_text.chars().count().saturating_sub(col0);
    let count = span_chars.min(available).max(1);

    out.push_str(&format!("{:>gutter$} | {line_text}\n", start.line));
    out.push_str(&format!(
        "{pad} | {}{}{}\n",
        " ".repeat(col0),
        mark.to_string().repeat(count),
        label.unwrap_or_default(),
    ));
}