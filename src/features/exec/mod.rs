use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::interaction::InteractionHandler;
use crate::core::interaction::RealInteractionHandler;
use crate::core::policy::ProfileSafetyPolicy;
use crate::core::sso::{SsoService, SsoStatus};
use crate::features::search::SearchCommand;
use anyhow::Result;
use std::process::Command;
use std::sync::Arc;

/// Command to execute a subprocess within a profile context.
pub struct ExecCommand;

impl ExecCommand {
    /// Executes the given arguments as a command.
    pub fn execute(name_opt: Option<String>, args: Vec<String>) -> Result<(), AppError> {
        if args.is_empty() {
            return Err(AppError::Unknown(anyhow::anyhow!("No command specified")));
        }

        let store = Store::new()?;
        let policy = ProfileSafetyPolicy::new();
        let handler = Arc::new(RealInteractionHandler::new());
        let sso_service = SsoService::new()?;

        // 1. Resolve
        let Some(profile_name) = SearchCommand::resolve_or_search(name_opt)? else {
            return Ok(());
        };

        // 2. Fetch & Policy
        let profile = store
            .find_by_name(&profile_name)?
            .ok_or_else(|| AppError::ProfileNotFound(profile_name.clone()))?;

        let tags = profile
            .metadata
            .as_ref()
            .map(|m| m.tags.clone())
            .unwrap_or_default();
        if policy.is_sensitive_profile(&profile_name, &tags)
            && !handler.confirm_sensitive_action(&profile_name)
        {
            println!("Aborted.");
            return Ok(());
        }

        // 3. SSO Check
        let status = sso_service.get_status(&profile);
        if status == SsoStatus::Expired {
            println!("Warning: SSO session for '{}' is expired.", profile_name);
        }

        // 4. Exec
        let mut cmd = Command::new(&args[0]);
        if args.len() > 1 {
            cmd.args(&args[1..]);
        }

        // Ensure raw environment variables don't override the profile
        cmd.env_remove("AWS_ACCESS_KEY_ID")
            .env_remove("AWS_SECRET_ACCESS_KEY")
            .env_remove("AWS_SESSION_TOKEN");

        cmd.env("AWS_PROFILE", &profile_name);
        if let Some(region) = profile.metadata.as_ref().and_then(|m| m.region.as_ref()) {
            cmd.env("AWS_REGION", region);
        }

        let status = cmd.status().map_err(AppError::IoError)?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}
