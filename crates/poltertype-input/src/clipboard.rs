//! Clipboard capability boundary for the privacy-first Work build.
//!
//! Selection conversion is intentionally unavailable: this build must
//! never read or overwrite another application's clipboard. The trait is
//! kept as a compatibility seam for the existing engine/settings code,
//! but no system clipboard backend is constructed.

use crate::InputError;

/// Minimal interface retained for compatibility with the engine.
pub trait Clipboard: Send + Sync {
    fn text(&self) -> Result<Option<String>, InputError>;
    fn set_text(&self, text: &str) -> Result<(), InputError>;
}

/// Why selection conversion is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardGap {
    DisabledInWorkBuild,
}

impl std::fmt::Display for ClipboardGap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisabledInWorkBuild => write!(
                f,
                "selection conversion is disabled in the privacy-first Work build"
            ),
        }
    }
}

/// Work builds never expose a system clipboard implementation.
pub fn clipboard() -> Result<Box<dyn Clipboard>, ClipboardGap> {
    Err(ClipboardGap::DisabledInWorkBuild)
}

/// Work builds never allow selection conversion.
pub fn selection_support() -> Result<(), ClipboardGap> {
    Err(ClipboardGap::DisabledInWorkBuild)
}
