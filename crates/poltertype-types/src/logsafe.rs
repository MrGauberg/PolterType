//! Redaction of user-typed text in logs and decision reasons.
//!
//! Work-build privacy contract: typed text is never rendered into a
//! diagnostic string, including debug builds. Keep only character count,
//! which is sufficient to diagnose buffer boundaries without retaining
//! the user's content.

/// Render `word` for a log line or decision reason as `<N chars>`.
/// There is intentionally no environment-variable or debug-build escape
/// hatch: a build used on a work device must have the same privacy
/// properties as the release binary.
pub fn redact_word(word: &str) -> String {
    render(word)
}

pub(crate) fn render(word: &str) -> String {
    format!("<{} chars>", word.chars().count())
}
