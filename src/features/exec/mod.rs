/// Implementation logic for the exec command.
pub mod logic;
use crate::core::error::AppError;
use crate::features::search::SearchCommand;
use anyhow::Result;
use logic::{ExecLogic, RealCommandExecutor};

/// Command to execute a subprocess within a profile context.
pub struct ExecCommand;

impl ExecCommand {
    /// Executes the given arguments as a command.
    pub fn execute(
        name_opt: Option<String>,
        args: Vec<String>,
        yes: bool,
        preserve_env: bool,
    ) -> Result<(), AppError> {
        // Resolve profile name here (Command layer)
        let Some(profile_name) = SearchCommand::resolve_or_search(name_opt)? else {
            return Ok(());
        };

        let executor = RealCommandExecutor;
        let code = ExecLogic::execute(&executor, profile_name, args, yes, preserve_env)?;
        if code != 0 {
            std::process::exit(code);
        }
        Ok(())
    }
}
// touched
