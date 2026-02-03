use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::interaction::{InteractionHandler, RealInteractionHandler};
use crate::core::policy::ProfileSafetyPolicy;
use crate::features::search::SearchCommand;
use anyhow::{Context, Result};
use console::style;
use std::process::Command;
use std::sync::Arc;

/// Command to switch the current shell to a profile context.
pub struct SwitchCommand;

impl SwitchCommand {
    /// Executes the switch command, spawning a subshell.
    pub fn execute(name_opt: Option<String>, yes: bool) -> Result<(), AppError> {
        let store = Store::new()?;
        let global_config = store.get_global_config().unwrap_or_default();
        let policy = ProfileSafetyPolicy::new(global_config.sensitive_keywords.clone());
        let handler = Arc::new(RealInteractionHandler::new());

        // 1. Resolve Profile Name
        let Some(profile_name) = SearchCommand::resolve_or_search(name_opt)? else {
            return Ok(());
        };

        // 2. Guardrails
        if let Some(profile) = store.find_by_name(&profile_name)? {
            let tags = profile
                .metadata
                .as_ref()
                .map(|m| m.tags.clone())
                .unwrap_or_default();
            if policy.is_sensitive_profile(&profile_name, &tags)
                && !yes
                && !handler.confirm_sensitive_action(&profile_name)
            {
                println!("{}", style("Aborted.").red());
                return Ok(());
            }

            // 3. Update usage
            let mut new_meta = profile.metadata.clone().unwrap_or_default();
            new_meta.last_used_at = Some(chrono::Utc::now().to_rfc3339());
            store.save_metadata(&profile_name, new_meta)?;

            // 4. Spawn Subshell
            #[cfg(unix)]
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            #[cfg(windows)]
            let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
            println!(
                "{} {} {}",
                style("Switched to profile:").green().bold(),
                style(&profile_name).yellow().bold(),
                style("(type 'exit' to return)").dim()
            );

            let mut cmd = Command::new(&shell);
            cmd.env("AWS_PROFILE", &profile_name);
            if let Some(region) = profile.metadata.as_ref().and_then(|m| m.region.as_ref()) {
                cmd.env("AWS_REGION", region);
            }

            let mut child = cmd
                .spawn()
                .context("Failed to spawn subshell")
                .map_err(AppError::Unknown)?;
            child
                .wait()
                .context("Failed to wait for subshell")
                .map_err(AppError::Unknown)?;
        }

        Ok(())
    }
}
