//! Completion command implementation

use clap::CommandFactory;
use clap_complete::{generate, Shell as ClapShell};

use crate::cli::{Cli, CompletionArgs, Shell};
use crate::error::Result;

pub fn execute(args: CompletionArgs) -> Result<()> {
    let shell = match args.shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::Powershell => ClapShell::PowerShell,
        Shell::Elvish => ClapShell::Elvish,
    };

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    generate(shell, &mut cmd, bin_name, &mut std::io::stdout());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_bash() {
        let args = CompletionArgs { shell: Shell::Bash };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_completion_zsh() {
        let args = CompletionArgs { shell: Shell::Zsh };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_completion_fish() {
        let args = CompletionArgs { shell: Shell::Fish };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_completion_powershell() {
        let args = CompletionArgs {
            shell: Shell::Powershell,
        };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_completion_elvish() {
        let args = CompletionArgs {
            shell: Shell::Elvish,
        };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_shells() {
        for shell in [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::Powershell,
            Shell::Elvish,
        ] {
            let args = CompletionArgs { shell };
            assert!(execute(args).is_ok());
        }
    }
}
