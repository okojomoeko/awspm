mod logic;
mod view;

use crate::core::config::Store;
use crate::core::error::AppError;
use crate::core::interaction::RealInteractionHandler;
use logic::SearchLogic;
use std::sync::Arc;
use view::SearchView;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Specific target field to search within.
pub enum SearchTarget {
    /// Search across all fields.
    All,
    /// Search only tags.
    Tags,
    /// Search only aliases.
    Aliases,
    /// Search only profile names.
    Name,
}

/// Command to interactively search and select profiles.
pub struct SearchCommand;

impl SearchCommand {
    /// Executes the interactive search.
    pub fn execute(
        target: SearchTarget,
        query: Option<String>,
    ) -> Result<Option<String>, AppError> {
        // 1. Load Data
        let store = Store::new()?;
        let global_config = store.get_global_config().unwrap_or_default();
        let profiles = store.load_profiles()?; // Can be improved by find logic later if needed

        // 2. Logic (Sort)
        let sorted = SearchLogic::sort_profiles(profiles);

        // 3. View (Interactive)
        use crate::core::policy::ProfileSafetyPolicy;
        let policy = ProfileSafetyPolicy::new(global_config.sensitive_keywords.clone());
        let view = SearchView::new(Arc::new(RealInteractionHandler::new()), policy);
        view.prompt_search(&sorted, target, query)
    }

    /// Helper to resolve a profile: matches `name_opt` if some, or runs interactive search if none.
    pub fn resolve_or_search(name_opt: Option<String>) -> Result<Option<String>, AppError> {
        if let Some(name) = name_opt {
            let store = Store::new()?;
            let profile_opt = store.find_by_name_or_alias(&name)?;
            match profile_opt {
                Some(profile) => Ok(Some(profile.name)),
                None => Err(AppError::ProfileNotFound(name)),
            }
        } else {
            Self::execute(SearchTarget::All, None)
        }
    }
}
