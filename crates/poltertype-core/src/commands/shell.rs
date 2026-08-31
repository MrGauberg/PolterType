//! Compatibility schema for legacy `run_shell` smart commands.
//!
//! The privacy-first Work build deliberately has no process-execution
//! implementation. Old config files still parse, but execution is
//! refused unconditionally.

/// A legacy program invocation kept only so existing `config.toml` files
/// remain readable.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShellCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub insert_output: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ShellRefusal {
    DisabledInWorkBuild,
    EmptyProgram,
}

impl std::fmt::Display for ShellRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisabledInWorkBuild => {
                write!(f, "`run_shell` is disabled in the privacy-first Work build")
            }
            Self::EmptyProgram => write!(f, "`run_shell` needs a non-empty `program`"),
        }
    }
}

/// Work builds refuse process execution even when an old config still
/// carries the former opt-in flag.
pub fn check(cmd: &ShellCommand, _allow_run_shell: bool) -> Result<(), ShellRefusal> {
    if cmd.program.trim().is_empty() {
        return Err(ShellRefusal::EmptyProgram);
    }
    Err(ShellRefusal::DisabledInWorkBuild)
}

/// Kept for API compatibility; never starts a process.
pub fn run(_cmd: &ShellCommand) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(program: &str) -> ShellCommand {
        ShellCommand {
            program: program.to_owned(),
            args: vec!["ignored".to_owned()],
            insert_output: true,
        }
    }

    #[test]
    fn work_build_refuses_run_shell_even_when_legacy_flag_is_true() {
        assert_eq!(
            check(&cmd("echo"), true),
            Err(ShellRefusal::DisabledInWorkBuild)
        );
        assert_eq!(run(&cmd("echo")), None);
    }

    #[test]
    fn empty_program_is_still_reported_cleanly() {
        assert_eq!(check(&cmd("   "), true), Err(ShellRefusal::EmptyProgram));
    }
}
