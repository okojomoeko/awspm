use crate::core::config::Store;
use crate::core::error::AppError;
use anyhow::Result;

/// Arguments for the update logic.
/// Arguments for updating profile metadata.
pub struct UpdateArgs {
    /// Name of the profile to update.
    pub profile_name: String,
    /// Tags to add to the profile.
    pub add_tags: Vec<String>,
    /// Tags to remove from the profile.
    pub remove_tags: Vec<String>,
    /// Aliases to add to the profile.
    pub add_aliases: Vec<String>,
    /// Aliases to remove from the profile.
    pub remove_aliases: Vec<String>,
    /// Note to set for the profile.
    pub set_note: Option<String>,
    /// Whether to unset the note.
    pub unset_note: bool,
    /// Region to set for the profile.
    pub set_region: Option<String>,
    /// Whether to unset the region.
    pub unset_region: bool,
}

/// Logic for handling profile updates.
pub struct UpdateLogic;

impl UpdateLogic {
    /// Executes the update logic to modify profile metadata.
    pub fn execute(store: &Store, args: UpdateArgs) -> Result<(), AppError> {
        // 1. Resolve Profile
        let profile_opt = store.find_by_name_or_alias(&args.profile_name)?;
        let profile = match profile_opt {
            Some(p) => p,
            None => {
                return Err(AppError::Unknown(anyhow::anyhow!(
                    "Profile '{}' not found. Run 'sync' if it exists in AWS config.",
                    args.profile_name
                )));
            }
        };

        // 2. Load Metadata (from merged profile)
        let mut current_metadata = profile.metadata.clone().unwrap_or_default();

        // 3. Apply Modifications
        for tag in args.add_tags {
            if !current_metadata.tags.contains(&tag) {
                current_metadata.tags.push(tag);
            } else {
                println!(
                    "Tag '{}' already exists on profile '{}'.",
                    tag, profile.name
                );
            }
        }
        for tag in args.remove_tags {
            if current_metadata.tags.contains(&tag) {
                current_metadata.tags.retain(|t| t != &tag);
            } else {
                println!("Tag '{}' not found on profile '{}'.", tag, profile.name);
            }
        }

        for alias in &args.add_aliases {
            // Check for conflicts
            if let Ok(Some(existing)) = store.find_by_name_or_alias(alias) {
                // If it finds a different profile, it's a conflict.
                if existing.name != args.profile_name {
                    return Err(AppError::Unknown(anyhow::anyhow!(
                        "Alias '{}' conflicts with existing profile or alias on '{}'.",
                        alias,
                        existing.name
                    )));
                }
            }

            if !current_metadata.aliases.contains(alias) {
                current_metadata.aliases.push(alias.clone());
            } else {
                println!(
                    "Alias '{}' already exists on profile '{}'.",
                    alias, profile.name
                );
            }
        }
        for alias in args.remove_aliases {
            if current_metadata.aliases.contains(&alias) {
                current_metadata.aliases.retain(|a| a != &alias);
            } else {
                println!("Alias '{}' not found on profile '{}'.", alias, profile.name);
            }
        }

        // Deduplicate and sort for clean YAML output
        current_metadata.tags.sort();
        current_metadata.tags.dedup();
        current_metadata.aliases.sort();
        current_metadata.aliases.dedup();

        if let Some(note) = args.set_note {
            current_metadata.note = Some(note);
        } else if args.unset_note {
            current_metadata.note = None;
        }

        if let Some(region) = args.set_region {
            current_metadata.region = Some(region);
        } else if args.unset_region {
            current_metadata.region = None;
        }

        // 4. Save
        store.save_metadata(&profile.name, current_metadata)?;

        // Return Ok
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_update_add_alias_conflict_with_existing_profile() {
        use std::io::Write;
        let dir = tempdir().unwrap();
        let home = dir.path().to_path_buf();
        let config_path = home.join("aws_config");
        let metadata_path = home.join(".awspm.yaml");

        // 1. Create AWS Config file with checking profiles
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "[profile prod]").unwrap();
        writeln!(f, "[profile db]").unwrap();

        temp_env::with_vars(
            [
                ("AWSPM_METADATA_FILE", Some(metadata_path.to_str().unwrap())),
                ("AWS_CONFIG_FILE", Some(config_path.to_str().unwrap())),
                ("HOME", Some(home.to_str().unwrap())), // Set HOME to temp dir just in case
            ],
            || {
                let store = Store::new().unwrap();
                store.init_metadata_file().unwrap();

                // 3. Try to add alias "db" to "prod"
                let args = UpdateArgs {
                    profile_name: "prod".to_string(),
                    add_aliases: vec!["db".to_string()],
                    add_tags: vec![],
                    remove_tags: vec![],
                    remove_aliases: vec![],
                    set_note: None,
                    unset_note: false,
                    set_region: None,
                    unset_region: false,
                };

                let result = UpdateLogic::execute(&store, args);

                // Debug output
                if let Err(e) = &result {
                    println!("Debug: Error was {}", e);
                }

                // ASSERTION: This should fail because 'db' matches an existing profile.
                assert!(
                    result.is_err(),
                    "Should fail when adding alias 'db' which conflicts with profile 'db'"
                );
            },
        );
    }

    #[test]
    fn test_update_deduplicate_tags() {
        use std::io::Write;
        let dir = tempdir().unwrap();
        let home = dir.path().to_path_buf();
        let config_path = home.join("aws_config");
        let metadata_path = home.join(".awspm.yaml");

        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "[profile dev]").unwrap();

        temp_env::with_vars(
            [
                ("AWSPM_METADATA_FILE", Some(metadata_path.to_str().unwrap())),
                ("AWS_CONFIG_FILE", Some(config_path.to_str().unwrap())),
                ("HOME", Some(home.to_str().unwrap())),
            ],
            || {
                let store = Store::new().unwrap();
                store.init_metadata_file().unwrap();

                // Add tags with duplicates and out of order
                let args = UpdateArgs {
                    profile_name: "dev".to_string(),
                    add_tags: vec!["z".to_string(), "a".to_string(), "z".to_string()],
                    remove_tags: vec![],
                    add_aliases: vec![],
                    remove_aliases: vec![],
                    set_note: None,
                    unset_note: false,
                    set_region: None,
                    unset_region: false,
                };

                UpdateLogic::execute(&store, args).unwrap();

                let profile = store.find_by_name("dev").unwrap().unwrap();
                let tags = profile.metadata.unwrap().tags;
                assert_eq!(tags, vec!["a".to_string(), "z".to_string()]);
            },
        );
    }
}
