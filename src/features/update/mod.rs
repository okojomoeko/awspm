/// Implementation logic for the update command.
pub mod logic;
use crate::core::config::Store;
use crate::core::error::AppError;
use anyhow::Result;
pub use logic::UpdateArgs;
use logic::UpdateLogic;

/// Command to update profile metadata.
pub struct UpdateCommand;

impl UpdateCommand {
    /// Executes the update command.
    pub fn execute(args: UpdateArgs) -> Result<(), AppError> {
        let store = Store::new()?;
        UpdateLogic::execute(&store, args)
    }
}
