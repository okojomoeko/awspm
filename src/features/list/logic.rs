use crate::core::types::Profile;

/// Utility to filter profiles based on a query.
pub struct ListFilter;

impl ListFilter {
    /// Filters profiles based on the query string.
    /// Supports `tag:`, `alias:`, `name:` prefixes.
    pub fn filter(profiles: Vec<Profile>, query: Option<&str>) -> Vec<Profile> {
        let query_str = query.unwrap_or("");
        if query_str.is_empty() {
            return profiles;
        }

        let query_lower = query_str.to_lowercase();

        profiles
            .into_iter()
            .filter(|p| {
                if let Some(val) = query_lower.strip_prefix("tag:") {
                    p.metadata
                        .as_ref()
                        .is_some_and(|m| m.tags.iter().any(|t| t.to_lowercase().contains(val)))
                } else if let Some(val) = query_lower.strip_prefix("alias:") {
                    p.metadata
                        .as_ref()
                        .is_some_and(|m| m.aliases.iter().any(|a| a.to_lowercase().contains(val)))
                } else if let Some(val) = query_lower.strip_prefix("name:") {
                    p.name.to_lowercase().contains(val)
                } else {
                    // General Search
                    p.name.to_lowercase().contains(&query_lower)
                        || p.metadata.as_ref().is_some_and(|m| {
                            m.aliases
                                .iter()
                                .any(|a| a.to_lowercase().contains(&query_lower))
                                || m.tags
                                    .iter()
                                    .any(|t| t.to_lowercase().contains(&query_lower))
                                || m.note
                                    .as_ref()
                                    .is_some_and(|n| n.to_lowercase().contains(&query_lower))
                        })
                }
            })
            .collect()
    }
}
