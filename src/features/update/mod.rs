use crate::core::config::Store;
use crate::core::error::AppError;
use anyhow::Result;

/// Arguments for the update command.
pub struct UpdateArgs {
    /// Name of the profile to update.
    pub profile_name: String,
    /// List of tags to add.
    pub add_tags: Vec<String>,
    /// List of tags to remove.
    pub remove_tags: Vec<String>,
    /// List of aliases to add.
    pub add_aliases: Vec<String>,
    /// List of aliases to remove.
    pub remove_aliases: Vec<String>,
    /// Note content to set.
    pub set_note: Option<String>,
    /// Whether to unset the note.
    pub unset_note: bool,
    /// AWS Region to set.
    pub set_region: Option<String>,
    /// Whether to unset the region.
    pub unset_region: bool,
}

/// Command to update profile metadata.
pub struct UpdateCommand;

impl UpdateCommand {
    /// Executes the update command.
    pub fn execute(args: UpdateArgs) -> Result<(), AppError> {
        let store = Store::new()?;

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

        // 2. Load Existing Metadata
        // NOTE: Load existing metadata from Store actually gets the metadata from the profile struct if merged.
        // But to update, we need to read what is in the file specifically to avoid overwriting with AWS Config data?
        // Actually `store.save_metadata` assumes we are saving what we want.
        // We should construct the new metadata based on the current state.
        // The `profile` object has the *merged* metadata.
        // This means if we strictly use `profile.metadata`, we are safe.
        // Because `save_metadata` only writes to the metadata file.

        let mut current_metadata = profile.metadata.unwrap_or_default();

        // 3. Apply Modifications
        for tag in args.add_tags {
            if !current_metadata.tags.contains(&tag) {
                current_metadata.tags.push(tag);
            }
        }
        current_metadata
            .tags
            .retain(|t| !args.remove_tags.contains(t));

        for alias in args.add_aliases {
            if !current_metadata.aliases.contains(&alias) {
                current_metadata.aliases.push(alias);
            }
        }
        current_metadata
            .aliases
            .retain(|a| !args.remove_aliases.contains(a));

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
        // We use the canonical name from the profile since we resolved it.
        store.save_metadata(&profile.name, current_metadata)?;
        println!("Updated metadata for profile '{}'", profile.name);

        Ok(())
    }
}
