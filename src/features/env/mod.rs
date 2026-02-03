/// Implementation logic for the env command.
pub mod logic;
use crate::core::error::AppError;
use anyhow::Result;
use logic::EnvLogic;

/// Command to print environment variables.
pub struct EnvCommand;

impl EnvCommand {
    /// Executes the env command.
    pub fn execute(name_opt: Option<String>) -> Result<(), AppError> {
        EnvLogic::execute(name_opt)
    }
}
