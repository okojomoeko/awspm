use anyhow::Result;
use std::env;

use crate::core::error::AppError;

/// Command to show the current AWS profile.
pub struct CurrentCommand;

impl CurrentCommand {
    /// Executes the current command, checking environment variables first.
    pub fn execute() -> Result<(), AppError> {
        // 1. Check for raw environment variables (highest precedence)
        if env::var("AWS_ACCESS_KEY_ID").is_ok() {
            println!("env-vars");
            return Ok(());
        }

        // 2. Check for standard AWS_PROFILE
        match env::var("AWS_PROFILE") {
            Ok(val) => println!("{}", val),
            Err(_) => println!("default"),
        }
        Ok(())
    }
}
