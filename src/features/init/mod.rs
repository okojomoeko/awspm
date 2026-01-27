use crate::core::config::Store;
use crate::core::error::AppError;

/// Command to initialize the metadata file.
pub struct InitCommand;

impl InitCommand {
    /// Creates the .awspm.yaml file if it doesn't exist.
    pub fn execute() -> Result<(), AppError> {
        let store = Store::new()?;
        // Access private path via a method or just re-implement simple check?
        // Since Store encapsulates paths, maybe we should add an `init` method to Store?
        // OR, for now, just replicate the logic since it's simple.
        // BUT strict Vertical Slice prefers keeping "Business Logic" here.
        // The business logic is "Check if exists, if not create default".

        // Actually, Store doesn't expose path.
        // Let's add a helper to Store or just use the same env var logic here.
        // To be safe and DRY on paths, let's add `init_metadata_file` to Store.
        store.init_metadata_file()
    }
}
