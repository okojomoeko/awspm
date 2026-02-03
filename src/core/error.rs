use thiserror::Error;

/// Application-level error type used throughout `awspm`.
#[derive(Error, Debug)]
pub enum AppError {
    /// Error occurring when loading AWS configuration.
    #[error("Failed to load configuration: {0}")]
    ConfigLoadError(String),

    /// Error occurring when parsing configuration files.
    #[error("Failed to parse configuration: {0}")]
    ConfigParseError(String),

    /// Error occurring when accessing the metadata repository.
    #[error("Metadata access error: {0}")]
    MetadataAccessError(String),

    /// Standard I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error during serialization/deserialization (YAML).
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    /// The requested profile was not found.
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    /// The requested alias matched multiple profiles.
    #[error(
        "Ambiguous profile reference: '{0}' matched multiple profiles. Please use the full profile name."
    )]
    AmbiguousProfile(String),

    /// An unclassified error (wrapped `anyhow::Error`).
    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),

    /// The user's home directory could not be determined.
    #[error("Could not determine home directory")]
    HomeDirNotFound,
}
