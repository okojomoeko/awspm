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
    /// Creates a new policy with default sensitive keywords.
    pub fn new() -> Self {
        Self::default()
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
