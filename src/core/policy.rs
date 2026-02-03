use std::collections::HashSet;

/// Policy defining which profiles are considered "sensitive" (e.g., production environments).
pub struct ProfileSafetyPolicy {
    sensitive_keywords: HashSet<String>,
}

impl Default for ProfileSafetyPolicy {
    fn default() -> Self {
        let mut sensitive_keywords = HashSet::new();
        sensitive_keywords.insert("prod".to_string());
        sensitive_keywords.insert("production".to_string());
        sensitive_keywords.insert("live".to_string());
        sensitive_keywords.insert("main".to_string());

        Self { sensitive_keywords }
    }
}

impl ProfileSafetyPolicy {
    /// Creates a new policy with optional custom sensitive keywords.
    /// If None, usage defaults.
    pub fn new(custom_keywords: Option<Vec<String>>) -> Self {
        let mut sensitive_keywords = HashSet::new();
        if let Some(keywords) = custom_keywords {
            for k in keywords {
                sensitive_keywords.insert(k.to_lowercase());
            }
        } else {
            // Default
            sensitive_keywords.insert("prod".to_string());
            sensitive_keywords.insert("production".to_string());
            sensitive_keywords.insert("live".to_string());
            sensitive_keywords.insert("main".to_string());
        }
        Self { sensitive_keywords }
    }

    /// Checks if a profile is considered "sensitive" based on name or tags.
    pub fn is_sensitive_profile(&self, profile_name: &str, tags: &[String]) -> bool {
        let lower_name = profile_name.to_lowercase();
        for keyword in &self.sensitive_keywords {
            if lower_name.contains(keyword) {
                return true;
            }
        }
        for tag in tags {
            if self.sensitive_keywords.contains(&tag.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_keywords() {
        // Custom keywords
        let policy = ProfileSafetyPolicy::new(Some(vec!["foo".to_string()]));
        assert!(policy.is_sensitive_profile("foo-bar", &[]));
        assert!(!policy.is_sensitive_profile("prod", &[])); // verify default not included if custom provided

        // Defaults
        let default_policy = ProfileSafetyPolicy::new(None);
        assert!(default_policy.is_sensitive_profile("prod", &[]));
        assert!(!default_policy.is_sensitive_profile("foo", &[]));
    }
}
