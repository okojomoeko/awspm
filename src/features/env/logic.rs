use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::types::Profile;
use crate::features::search::SearchCommand;
use anyhow::Result;

/// Logic for handling environment variable exports.
pub struct EnvLogic;

impl EnvLogic {
    /// Executes the env command, printing export statements to stdout.
    pub fn execute(name_opt: Option<String>) -> Result<(), AppError> {
        let store = Store::new()?;

        // 1. Resolve
        let Some(profile_name) = SearchCommand::resolve_or_search(name_opt)? else {
            return Ok(());
        };

        // 2. Fetch
        let profile = store
            .find_by_name(&profile_name)?
            .ok_or_else(|| AppError::ProfileNotFound(profile_name.clone()))?;

        // 3. Generate
        let exports = Self::generate_exports(&profile);

        // 4. Print
        for line in exports {
            println!("{}", line);
        }

        Ok(())
    }

    /// Generates the list of export strings for the given profile.
    pub fn generate_exports(profile: &Profile) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("export AWS_PROFILE={}", profile.name));

        // TODO: Include Region?
        // TODO: Include SSO vars? (start_url etc usually not needed in env if profile handles it, but maybe AWS_REGION is useful)
        // If metadata has region override, we should export it.

        if let Some(region) = profile
            .metadata
            .as_ref()
            .and_then(|m| m.region.as_ref())
            .or(profile.region.as_ref())
        {
            lines.push(format!("export AWS_REGION={}", region));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Metadata;
    use crate::core::types::Profile;

    #[test]
    fn test_generate_exports() {
        // Setup simple profile
        let p = Profile::new("test-env".to_string());

        let exports = EnvLogic::generate_exports(&p);
        assert!(exports.contains(&"export AWS_PROFILE=test-env".to_string()));
    }

    #[test]
    fn test_generate_exports_with_region() {
        let mut p = Profile::new("test-env".to_string());
        p.metadata = Some(Metadata {
            region: Some("us-test-1".to_string()),
            ..Default::default()
        });

        let exports = EnvLogic::generate_exports(&p);
        assert!(exports.contains(&"export AWS_PROFILE=test-env".to_string()));
        assert!(
            exports.contains(&"export AWS_REGION=us-test-1".to_string()),
            "Should export region override"
        );
    }
}
