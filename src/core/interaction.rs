use console::style;
use dialoguer::Confirm;

/// Trait to handle user interactions, allowing for mocking in tests.
pub trait InteractionHandler {
    /// Prompts the user to confirm a sensitive action involving the given profile.
    /// Returns true if confirmed, false otherwise.
    fn confirm_sensitive_action(&self, profile_name: &str) -> bool;
}

/// The concrete implementation of `InteractionHandler` that uses real terminal I/O.
pub struct RealInteractionHandler;

impl RealInteractionHandler {
    /// Creates a new real interaction handler.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealInteractionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionHandler for RealInteractionHandler {
    fn confirm_sensitive_action(&self, profile_name: &str) -> bool {
        eprintln!(
            "{}",
            style(format!(
                "⚠️  WARNING: You are about to use a SENSITIVE profile: '{}'",
                profile_name
            ))
            .red()
            .bold()
        );
        eprintln!(
            "{}",
            style("This action could affect live systems.").yellow()
        );

        Confirm::new()
            .with_prompt("Do you want to proceed?")
            .default(false)
            .interact()
            .unwrap_or(false)
    }
}

#[cfg(test)]
/// Mock interaction module.
pub mod mock {
    use super::*;
    use std::sync::Mutex;

    /// Mock implementation of UserInteraction for testing.
    pub struct MockInteractionHandler {
        /// Result to return for confirm actions.
        pub confirm_result: bool,
        /// Last profile name passed to methods.
        pub last_profile_name: Mutex<Option<String>>,
    }

    impl MockInteractionHandler {
        /// Creates a new mock handler.
        pub fn new(confirm_result: bool) -> Self {
            Self {
                confirm_result,
                last_profile_name: Mutex::new(None),
            }
        }
    }

    impl InteractionHandler for MockInteractionHandler {
        fn confirm_sensitive_action(&self, profile_name: &str) -> bool {
            *self.last_profile_name.lock().unwrap() = Some(profile_name.to_string());
            self.confirm_result
        }
    }
}
