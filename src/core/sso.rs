use crate::core::error::AppError;
use crate::core::types::Profile;
use crate::core::utils::expand_tilde;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;

/// Represents the SSO session status for a profile.
#[derive(Debug, PartialEq, Clone)]
pub enum SsoStatus {
    /// Session is valid and active.
    Active,
    /// Session has expired (needs login).
    Expired,
    /// Profile is not configured for SSO.
    NotConfigured,
    /// Status could not be determined or cache file missing.
    Unknown,
}

/// Service for checking AWS SSO session validity.
pub struct SsoService {
    cache_dir: PathBuf,
}

#[derive(Deserialize)]
struct SsoCache {
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

impl SsoService {
    /// Creates a new `SsoService`, locating the AWS SSO cache directory.
    pub fn new(custom_path: Option<String>) -> Result<Self, AppError> {
        if let Some(path_str) = custom_path {
            return Ok(Self::with_cache_dir(expand_tilde(path_str)));
        }
        let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
        Ok(Self::with_cache_dir(home.join(".aws/sso/cache")))
    }

    /// Creates a new `SsoService` with a custom cache directory (for testing).
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    #[tracing::instrument(skip(self, profile), fields(profile = %profile.name))]
    /// Determines the SSO status for a given profile.
    pub fn get_status(&self, profile: &Profile) -> SsoStatus {
        let cache_key = match (&profile.sso_session, &profile.sso_start_url) {
            (Some(session), _) => self.hash_key(session),
            (None, Some(url)) => self.hash_key(url),
            (None, None) => {
                tracing::trace!("Profile has no SSO config");
                return SsoStatus::NotConfigured;
            }
        };

        let path = self.cache_dir.join(format!("{}.json", cache_key));
        tracing::trace!("Checking SSO cache at: {}", path.display());

        if !path.exists() {
            tracing::debug!("Cache file not found (Expired/Unknown)");
            return SsoStatus::Unknown;
        }

        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(cache) = serde_json::from_str::<SsoCache>(&content)
        {
            if let Ok(expires_at) = DateTime::parse_from_rfc3339(&cache.expires_at) {
                if expires_at > Utc::now() {
                    tracing::debug!("SSO Active (expires: {})", expires_at);
                    return SsoStatus::Active;
                } else {
                    tracing::debug!("SSO Expired (expired: {})", expires_at);
                    return SsoStatus::Expired;
                }
            } else {
                tracing::warn!("Failed to parse expiresAt: {}", cache.expires_at);
            }
        } else {
            tracing::warn!("Failed to read or parse cache file: {}", path.display());
        }

        SsoStatus::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_sso_status_active() {
        let dir = tempdir().unwrap();
        let service = SsoService::with_cache_dir(dir.path().to_path_buf());

        // Setup profile
        let mut profile = Profile::new("test".to_string());
        profile.sso_session = Some("session-id".to_string());

        // Create cache file
        let hash = service.hash_key("session-id");
        let cache_path = dir.path().join(format!("{}.json", hash));
        let future = Utc::now() + chrono::Duration::hours(1);
        let content = serde_json::json!({
            "expiresAt": future.to_rfc3339()
        });

        let mut file = File::create(cache_path).unwrap();
        write!(file, "{}", content).unwrap();

        // Verify
        assert_eq!(service.get_status(&profile), SsoStatus::Active);
    }

    #[test]
    fn test_sso_status_expired() {
        let dir = tempdir().unwrap();
        let service = SsoService::with_cache_dir(dir.path().to_path_buf());

        let mut profile = Profile::new("test".to_string());
        profile.sso_session = Some("session-id".to_string());

        let hash = service.hash_key("session-id");
        let cache_path = dir.path().join(format!("{}.json", hash));
        let past = Utc::now() - chrono::Duration::hours(1);
        let content = serde_json::json!({
            "expiresAt": past.to_rfc3339()
        });

        let mut file = File::create(cache_path).unwrap();
        write!(file, "{}", content).unwrap();

        assert_eq!(service.get_status(&profile), SsoStatus::Expired);
    }

    #[test]
    fn test_sso_status_not_configured() {
        let dir = tempdir().unwrap();
        let service = SsoService::with_cache_dir(dir.path().to_path_buf());
        let profile = Profile::new("test".to_string()); // No SSO config

        assert_eq!(service.get_status(&profile), SsoStatus::NotConfigured);
    }

    #[test]
    fn test_sso_status_unknown_missing_file() {
        let dir = tempdir().unwrap();
        let service = SsoService::with_cache_dir(dir.path().to_path_buf());
        let mut profile = Profile::new("test".to_string());
        profile.sso_session = Some("session-id".to_string());

        // No file created

        assert_eq!(service.get_status(&profile), SsoStatus::Unknown);
    }
}
