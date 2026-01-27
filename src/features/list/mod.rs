/// List filtering logic.
pub mod logic;
mod view;

use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::sso::SsoService;
use crate::features::list::logic::ListFilter;
use view::ListView;

/// Command to list profiles.
pub struct ListCommand;

impl ListCommand {
    /// Executes the list command with optional filtering and short format.
    pub fn execute(query: Option<String>, short: bool) -> Result<(), AppError> {
        // 1. Load Data
        let store = Store::new()?;
        let profiles = store.load_profiles()?;
        let sso_service = SsoService::new()?;

        // 2. Logic (Filter)
        let filtered = ListFilter::filter(profiles, query.as_deref());

        // 3. View (Render)
        let view = ListView::new(short, query.as_deref().unwrap_or(""), sso_service);
        view.render(&filtered);

        Ok(())
    }
}
