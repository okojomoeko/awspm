use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::interaction::{InteractionHandler, RealInteractionHandler};
use crate::core::policy::ProfileSafetyPolicy;
use crate::features::search::SearchCommand;
use anyhow::Result;
use console::style;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// Command to fix a profile to the current directory via .envrc.
pub struct PinCommand;

impl PinCommand {
    /// Executes the pin command in the current directory.
    pub fn execute(name_opt: Option<String>, yes: bool) -> Result<(), AppError> {
        let cwd = std::env::current_dir().map_err(AppError::IoError)?;
        Self::run(name_opt, cwd, yes)
    }

    /// Internal run logic (exposed for testing with arbitrary paths).
    pub fn run(
        name_opt: Option<String>,
        cwd: std::path::PathBuf,
        yes: bool,
    ) -> Result<(), AppError> {
        let store = Store::new()?;
        let global_config = store.get_global_config().unwrap_or_default();
        let policy = ProfileSafetyPolicy::new(global_config.sensitive_keywords.clone());
        let handler = Arc::new(RealInteractionHandler::new());

        // 1. Resolve Name
        let Some(profile_name) = SearchCommand::resolve_or_search(name_opt)? else {
            return Ok(());
        };

        // 2. Check Sensitivity
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
        }

        // 3. Update .envrc
        let envrc_path = cwd.join(".envrc");
        Self::update_envrc(&envrc_path, &profile_name)?;

        println!(
            "{} Setup local profile: {}",
            style("✓").green(),
            style(profile_name).cyan()
        );
        if Self::check_direnv_installed() {
            println!("Run {} to apply changes.", style("direnv allow").yellow());
        }

        Ok(())
    }

    fn update_envrc(path: &Path, profile_name: &str) -> Result<(), AppError> {
        let export_line = format!("export AWS_PROFILE={}", profile_name);

        let mut content = if path.exists() {
            fs::read_to_string(path).map_err(AppError::IoError)?
        } else {
            String::new()
        };

        let re =
            Regex::new(r"(?m)^export AWS_PROFILE=.*$").expect("Failed to compile constant regex");

        if re.is_match(&content) {
            content = re.replace(&content, export_line.as_str()).to_string();
        } else {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&export_line);
            content.push('\n');
        }

        let mut file = fs::File::create(path).map_err(AppError::IoError)?;
        file.write_all(content.as_bytes())
            .map_err(AppError::IoError)?;
        Ok(())
    }

    fn check_direnv_installed() -> bool {
        std::process::Command::new("direnv")
            .arg("--version")
            .output()
            .is_ok()
    }
}
