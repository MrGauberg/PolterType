//! Executable plug-ins are disabled in the privacy-first Work build.
//!
//! The data-only pack format remains available in `poltertype-core`.
//! This module keeps the app/UI compatibility surface for legacy
//! Extension manifests, but no function here starts a process.

use std::collections::HashMap;

use poltertype_core::plugins::DiscoveredExtension;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Departed {
    pub id: String,
    pub why: String,
}

#[derive(Default)]
pub struct Supervisor;

impl Supervisor {
    pub fn new() -> Self {
        Self
    }

    pub fn reap(&mut self) -> Vec<Departed> {
        Vec::new()
    }

    pub fn has_services(&self) -> bool {
        false
    }

    pub fn stop_all(&mut self) {}
}

const DISABLED: &str = "executable plug-ins are disabled in the privacy-first Work build";

pub fn run_command(_ext: &DiscoveredExtension, _command_id: &str) -> Result<(), String> {
    Err(DISABLED.to_owned())
}

pub fn run_command_for_row(
    _ext: &DiscoveredExtension,
    _command_id: &str,
    _row_id: &str,
) -> Result<(), String> {
    Err(DISABLED.to_owned())
}

pub fn run_command_for_row_waiting(
    _ext: &DiscoveredExtension,
    _command_id: &str,
    _row_id: &str,
) -> Result<String, String> {
    Err(DISABLED.to_owned())
}

pub fn read_rows(_ext: &DiscoveredExtension, _command_id: &str) -> Vec<super::menu::MenuRow> {
    Vec::new()
}

pub fn read_state(_ext: &DiscoveredExtension) -> Option<HashMap<String, String>> {
    None
}

pub fn read_report(_ext: &DiscoveredExtension, _command_id: &str) -> Result<String, String> {
    Err(DISABLED.to_owned())
}
