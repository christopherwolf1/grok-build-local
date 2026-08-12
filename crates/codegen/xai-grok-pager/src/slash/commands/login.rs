//! `/login` — local-runtime setup (hosted grok.com login is not required).

use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

pub struct LoginCommand;

impl SlashCommand for LoginCommand {
    fn name(&self) -> &str {
        "login"
    }

    fn description(&self) -> &str {
        "Show how to use a local runtime (no grok.com login)"
    }

    fn usage(&self) -> &str {
        "/login"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Message(xai_grok_shell::agent::config::local_runtime_operator_help())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_describes_local_runtime_not_hosted_account() {
        let cmd = LoginCommand;
        assert!(cmd.description().contains("local runtime"));
        assert!(!cmd.description().to_ascii_lowercase().contains("account"));
        let help = xai_grok_shell::agent::config::local_runtime_operator_help();
        assert!(help.contains("GROK_LOCAL_MODEL"));
        assert!(help.contains("127.0.0.1:11434"));
    }
}
