use crate::logsafe;

#[test]
fn redaction_hides_the_word_and_keeps_its_length() {
    assert_eq!(logsafe::render("mañana"), "<6 chars>");
    assert_eq!(logsafe::render("привіт"), "<6 chars>");
    assert_eq!(logsafe::render(""), "<0 chars>");
}

#[test]
fn redaction_has_no_plaintext_escape_hatch() {
    assert_eq!(logsafe::redact_word("work-secret"), "<11 chars>");
}
