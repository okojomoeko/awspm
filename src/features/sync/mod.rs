use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::types::Metadata;
use anyhow::Result;

/// Command to synchronize AWS config with metadata.
pub struct SyncCommand;

impl SyncCommand {
    /// Executes the sync process, handling untracked and orphaned profiles.
    pub fn execute(check_only: bool, auto_approve: bool) -> Result<(), AppError> {
        let store = Store::new()?;
        let status = store.check_sync_status()?;

        // --- Handle Untracked ---
        if !status.untracked.is_empty() {
            println!("Found {} untracked profiles:", status.untracked.len());
            for p in &status.untracked {
                println!("  + {}", p.name);
            }

            if check_only {
                // return? or continue?
            } else {
                let should_add = if auto_approve {
                    true
                } else {
                    use dialoguer::Confirm;
                    Confirm::new()
                        .with_prompt("Do you want to add these profiles to metadata?")
                        .default(true)
                        .interact()
                        .unwrap_or(false)
                };

                if should_add {
                    for p in status.untracked {
                        let new_meta = Metadata {
                            aliases: Vec::new(),
                            tags: Vec::new(),
                            note: None,
                            last_used_at: None,
                            region: None,
                        };
                        store.save_metadata(&p.name, new_meta)?;
                        println!("Added: {}", p.name);
                    }
                }
            }
        } else {
            println!("No new profiles to add.");
        }

        // --- Handle Orphaned ---
        if !status.orphaned.is_empty() {
            println!(
                "\nFound {} orphaned profiles (in metadata but missing from AWS config/creds):",
                status.orphaned.len()
            );
            for name in &status.orphaned {
                println!("  - {}", name);
            }

            if !check_only {
                let should_remove = if auto_approve {
                    true
                } else {
                    use dialoguer::Confirm;
                    Confirm::new()
                        .with_prompt("Do you want to remove these orphaned profiles from metadata?")
                        .default(false)
                        .interact()
                        .unwrap_or(false)
                };

                if should_remove {
                    // We need a remove_metadata method on Store? Or just manually remove?
                    // Store doesn't have remove_metadata yet.
                    // For now, let's just print "Removal not implemented yet" or implement it inline if easy,
                    // but Store abstraction is better.
                    // Let's assume we will add `delete_metadata` to Store.
                    for name in status.orphaned {
                        store.delete_metadata(&name)?;
                        println!("Removed: {}", name);
                    }
                }
            }
        } else {
            println!("No orphaned profiles found.");
        }

        Ok(())
    }
}
