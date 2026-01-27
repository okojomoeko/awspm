use awspm::core::config::Store;
use awspm::core::error::AppError;

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_load_valid_config() -> Result<(), AppError> {
    let dir = tempdir().map_err(|e| AppError::Unknown(e.into()))?;
    let config_path = dir.path().join("config");
    let credentials_path = dir.path().join("credentials"); // Dummy credentials

    let mut file = File::create(&config_path).map_err(AppError::IoError)?;
    writeln!(file, "[default]").map_err(AppError::IoError)?;
    writeln!(file, "region = us-east-1").map_err(AppError::IoError)?;
    writeln!(file, "output = json").map_err(AppError::IoError)?;
    writeln!(file).map_err(AppError::IoError)?;
    writeln!(file, "[profile staging]").map_err(AppError::IoError)?;
    writeln!(file, "region = us-west-2").map_err(AppError::IoError)?;
    writeln!(file, "output = text").map_err(AppError::IoError)?;

    // Create empty credentials file to prevent fallback to ~/.aws/credentials
    File::create(&credentials_path).map_err(AppError::IoError)?;

    let metadata_path = dir.path().join(".awspm.yaml");
    temp_env::with_vars(
        vec![
            ("AWS_CONFIG_FILE", Some(config_path.as_os_str())),
            (
                "AWS_SHARED_CREDENTIALS_FILE",
                Some(credentials_path.as_os_str()),
            ),
            ("AWSPM_METADATA_FILE", Some(metadata_path.as_os_str())),
            ("AWS_PROFILE", None),
            ("AWS_ACCESS_KEY_ID", None),
        ],
        || {
            let store = Store::new().unwrap();
            let profiles = store.load_profiles().unwrap();

            assert_eq!(profiles.len(), 2);

            let default = profiles.iter().find(|p| p.name == "default").unwrap();
            assert_eq!(default.region.as_deref(), Some("us-east-1"));
            assert_eq!(default.output.as_deref(), Some("json"));

            let staging = profiles.iter().find(|p| p.name == "staging").unwrap();
            assert_eq!(staging.region.as_deref(), Some("us-west-2"));
            assert_eq!(staging.output.as_deref(), Some("text"));
        },
    );

    Ok(())
}

#[test]
fn test_load_empty_config() -> Result<(), AppError> {
    let dir = tempdir().map_err(|e| AppError::Unknown(e.into()))?;
    let config_path = dir.path().join("config");
    let credentials_path = dir.path().join("credentials");

    File::create(&config_path).map_err(AppError::IoError)?; // Create empty config
    File::create(&credentials_path).map_err(AppError::IoError)?; // Create empty credentials

    let metadata_path = dir.path().join(".awspm.yaml");
    temp_env::with_vars(
        vec![
            ("AWS_CONFIG_FILE", Some(config_path.as_os_str())),
            (
                "AWS_SHARED_CREDENTIALS_FILE",
                Some(credentials_path.as_os_str()),
            ),
            ("AWSPM_METADATA_FILE", Some(metadata_path.as_os_str())),
            ("AWS_PROFILE", None),
            ("AWS_ACCESS_KEY_ID", None),
        ],
        || {
            let store = Store::new().unwrap();
            let profiles = store.load_profiles().unwrap();
            assert!(
                profiles.is_empty(),
                "Profiles should be empty, but found: {:?}",
                profiles
            );
        },
    );

    Ok(())
}

#[test]
fn test_ignore_irrelevant_sections() -> Result<(), AppError> {
    let dir = tempdir().map_err(|e| AppError::Unknown(e.into()))?;
    let config_path = dir.path().join("config");
    let credentials_path = dir.path().join("credentials");

    let mut file = File::create(&config_path).map_err(AppError::IoError)?;
    writeln!(file, "[profile prod]").map_err(AppError::IoError)?;
    writeln!(file, "region = us-east-1").map_err(AppError::IoError)?;
    writeln!(file, "[sso-session my-sso]").map_err(AppError::IoError)?;
    writeln!(file, "sso_start_url = start_url").map_err(AppError::IoError)?;

    File::create(&credentials_path).map_err(AppError::IoError)?; // Empty credentials

    let metadata_path = dir.path().join(".awspm.yaml");
    temp_env::with_vars(
        vec![
            ("AWS_CONFIG_FILE", Some(config_path.as_os_str())),
            (
                "AWS_SHARED_CREDENTIALS_FILE",
                Some(credentials_path.as_os_str()),
            ),
            ("AWSPM_METADATA_FILE", Some(metadata_path.as_os_str())),
            ("AWS_PROFILE", None),
            ("AWS_ACCESS_KEY_ID", None),
        ],
        || {
            let store = Store::new().unwrap();
            let profiles = store.load_profiles().unwrap();

            assert_eq!(profiles.len(), 1);
            assert_eq!(profiles[0].name, "prod");
        },
    );

    Ok(())
}
