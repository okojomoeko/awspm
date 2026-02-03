use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::interaction::{InteractionHandler, RealInteractionHandler};
use crate::core::policy::ProfileSafetyPolicy;
use crate::core::sso::{SsoService, SsoStatus};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

/// Abstraction for running external commands.
pub trait CommandExecutor {
    /// Runs the specified program with arguments and environment variables.
    fn run(
        &self,
        program: &str,
        args: &[String],
        env_vars: &HashMap<String, String>,
        remove_env_vars: &[&str],
    ) -> Result<i32, AppError>;
}

/// Real implementation that actually runs subprocesses.
pub struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn run(
        &self,
        program: &str,
        args: &[String],
        env_vars: &HashMap<String, String>,
        remove_env_vars: &[&str],
    ) -> Result<i32, AppError> {
        let mut cmd = Command::new(program);
        cmd.args(args);

        for (key, val) in env_vars {
            cmd.env(key, val);
        }

        for key in remove_env_vars {
            cmd.env_remove(key);
        }

        // On Unix, exec replaces the process, but in Rust std::process::Command spawns a child.
        // To truly behave like 'exec', we would need platform-specific extensions (std::os::unix::process::CommandExt::exec).
        // For now, we spawn and wait, propagating the exit code, which matches the original implementation behavior.
        let status = cmd.status().map_err(AppError::IoError)?;

        if !status.success() {
            return Ok(status.code().unwrap_or(1));
        }

        Ok(0)
    }
}

/// Logic for executing commands in a profile context.
pub struct ExecLogic;

impl ExecLogic {
    /// Executes a command with the environment of the specified profile.
    pub fn execute<E: CommandExecutor>(
        executor: &E,
        profile_name: String,
        args: Vec<String>,
        yes: bool,
        preserve_env: bool,
    ) -> Result<i32, AppError> {
        if args.is_empty() {
            return Err(AppError::Unknown(anyhow::anyhow!("No command specified")));
        }

        let store = Store::new()?;
        let global_config = store.get_global_config().unwrap_or_default();
        let policy = ProfileSafetyPolicy::new(global_config.sensitive_keywords.clone());
        let handler = Arc::new(RealInteractionHandler::new());
        let sso_service = SsoService::new(global_config.sso_cache_path.clone())?;

        // 1. Fetch & Policy
        let profile = store
            .find_by_name(&profile_name)?
            .ok_or_else(|| AppError::ProfileNotFound(profile_name.clone()))?;

        let tags = profile
            .metadata
            .as_ref()
            .map(|m| m.tags.clone())
            .unwrap_or_default();

        // Check sensitivity
        let is_sensitive = policy.is_sensitive_profile(&profile_name, &tags);

        // Logic: if sensitive AND not yes AND not confirmed -> Abort
        if is_sensitive && !yes && !handler.confirm_sensitive_action(&profile_name) {
            return Err(AppError::Unknown(anyhow::anyhow!(
                "Aborted by user or policy."
            )));
        }

        // 3. SSO Check
        let status = sso_service.get_status(&profile);
        if status == SsoStatus::Expired {
            println!("Warning: SSO session for '{}' is expired.", profile_name);
        }

        // 4. Prepare Execution
        let program = &args[0];
        let cmd_args = if args.len() > 1 {
            args[1..].to_vec()
        } else {
            Vec::new()
        };

        let mut env_vars = HashMap::new();
        env_vars.insert("AWS_PROFILE".to_string(), profile_name.clone());

        // Region Priority: Metadata Override > AWS Config
        let region_opt = profile
            .metadata
            .as_ref()
            .and_then(|m| m.region.as_ref())
            .or(profile.region.as_ref());

        if let Some(region) = region_opt {
            env_vars.insert("AWS_REGION".to_string(), region.clone());
        }

        let mut remove_vars = Vec::new();
        if !preserve_env {
            remove_vars.push("AWS_ACCESS_KEY_ID");
            remove_vars.push("AWS_SECRET_ACCESS_KEY");
            remove_vars.push("AWS_SESSION_TOKEN");
        }

        executor.run(program, &cmd_args, &env_vars, &remove_vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use tempfile::tempdir;

    type ExecData = (String, Vec<String>, HashMap<String, String>, Vec<String>);

    struct MockExecutor {
        last_run: RefCell<Option<ExecData>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                last_run: RefCell::new(None),
            }
        }
    }

    impl CommandExecutor for MockExecutor {
        fn run(
            &self,
            program: &str,
            args: &[String],
            env_vars: &HashMap<String, String>,
            remove_env_vars: &[&str],
        ) -> Result<i32, AppError> {
            let remove_owned: Vec<String> = remove_env_vars.iter().map(|s| s.to_string()).collect();
            *self.last_run.borrow_mut() = Some((
                program.to_string(),
                args.to_vec(),
                env_vars.clone(),
                remove_owned,
            ));
            Ok(0)
        }
    }

    #[test]
    fn test_execute_logic_happy_path() {
        // 1. Setup minimal environment for Store
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config");
        let metadata_path = dir.path().join(".awspm.yaml");
        let credentials_path = dir.path().join("credentials");

        fs::write(&config_path, "[profile test-profile]\nregion=us-test-1").unwrap();
        fs::write(&metadata_path, "profiles: {}").unwrap();
        fs::write(&credentials_path, "").unwrap();

        temp_env::with_vars(
            [
                ("AWS_CONFIG_FILE", Some(config_path.to_str().unwrap())),
                ("AWSPM_METADATA_FILE", Some(metadata_path.to_str().unwrap())),
                (
                    "AWS_SHARED_CREDENTIALS_FILE",
                    Some(credentials_path.to_str().unwrap()),
                ),
            ],
            || {
                // 2. Run Logic with Mock
                let executor = MockExecutor::new();
                let args = vec!["echo".to_string(), "hello".to_string()];

                let result = ExecLogic::execute(
                    &executor,
                    "test-profile".to_string(),
                    args,
                    false, // yes
                    false, // preserve_env
                );

                assert!(result.is_ok(), "Execution failed: {:?}", result.err());

                // 3. Verify Mock Interaction
                let run_data = executor.last_run.borrow();
                assert!(run_data.is_some(), "Executor was not called!");
                let (program, cmd_args, envs, removed) = run_data.as_ref().unwrap();

                assert_eq!(program, "echo");
                assert_eq!(cmd_args, &vec!["hello".to_string()]);
                assert_eq!(envs.get("AWS_PROFILE").unwrap(), "test-profile");
                assert_eq!(envs.get("AWS_REGION").unwrap(), "us-test-1");

                assert!(removed.contains(&"AWS_ACCESS_KEY_ID".to_string()));
            },
        );
    }
}
