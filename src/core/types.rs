use serde::{Deserialize, Serialize};

/// Represents an AWS profile entry found in `~/.aws/config` or `~/.aws/credentials`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// The name of the profile (e.g., "default", "prod").
    pub name: String,
    /// The AWS region configured for this profile, if any.
    pub region: Option<String>,
    /// The output format (json, yaml, etc.), if configured.
    pub output: Option<String>,
    /// The SSO session name, if this profile uses AWS SSO.
    pub sso_session: Option<String>,
    /// The SSO start URL, if present.
    pub sso_start_url: Option<String>,
    /// Extended metadata managed by `awspm` (tags, aliases, etc.).
    pub metadata: Option<Metadata>,
    /// The source of the profile (Config, Credentials, Env, or hybrid).
    #[serde(skip)]
    pub source: ProfileSource,
}

/// Source of a profile.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum ProfileSource {
    #[default]
    /// Loaded from AWS config file.
    Config,
    /// Loaded from AWS credentials file.
    Credentials,
    /// Loaded from environment variables.
    Env,
    /// Present in multiple sources.
    Merged,
}

/// Extended metadata for a profile, stored in `~/.awspm.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Metadata {
    /// List of alternative names for the profile.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Tags for categorization (e.g., "production", "team-a").
    #[serde(default)]
    pub tags: Vec<String>,
    /// A human-readable note or description.
    #[serde(default)]
    pub note: Option<String>,
    /// An optional region override that takes precedence over the AWS config.
    #[serde(default)]
    pub region: Option<String>,
    /// Timestamp of when this profile was last used/switched to (ISO 8601).
    #[serde(default)]
    pub last_used_at: Option<String>, // ISO 8601 timestamp
}

impl Profile {
    /// Creates a new profile with the given name and empty fields.
    pub fn new(name: String) -> Self {
        Self {
            name,
            region: None,
            output: None,
            sso_session: None,
            sso_start_url: None,
            metadata: None,
            source: ProfileSource::default(),
        }
    }
}
