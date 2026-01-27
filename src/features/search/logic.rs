use crate::core::types::Profile;

pub struct SearchLogic;

impl SearchLogic {
    /// Retrieves all profiles sorted by "Last Used" time (descending), then by name (ascending).
    pub fn sort_profiles(mut profiles: Vec<Profile>) -> Vec<Profile> {
        profiles.sort_by(|a, b| {
            let a_time = a.metadata.as_ref().and_then(|m| m.last_used_at.as_deref());
            let b_time = b.metadata.as_ref().and_then(|m| m.last_used_at.as_deref());

            // Sort by time DESC, then name ASC
            b_time.cmp(&a_time).then_with(|| a.name.cmp(&b.name))
        });
        profiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Metadata;

    #[test]
    fn test_sort_profiles() {
        // Arrange
        let p1 = Profile {
            metadata: Some(Metadata {
                last_used_at: Some("2023-01-02T12:00:00Z".to_string()),
                ..Default::default()
            }),
            ..Profile::new("B".to_string())
        };

        let p2 = Profile {
            metadata: Some(Metadata {
                last_used_at: Some("2023-01-01T12:00:00Z".to_string()),
                ..Default::default()
            }),
            ..Profile::new("C".to_string())
        };

        let p3 = Profile::new("A".to_string());
        // No metadata => never used

        let _profiles = [p1.clone(), p2.clone(), p3.clone()];

        // Shuffle input to prove sort
        let input = vec![p3.clone(), p1.clone(), p2.clone()];

        // Act
        let sorted = SearchLogic::sort_profiles(input);

        // Assert
        assert_eq!(sorted[0].name, "B");
        assert_eq!(sorted[1].name, "C");
        assert_eq!(sorted[2].name, "A");
    }
}
