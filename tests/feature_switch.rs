use awspm::core::config::Store;
use awspm::core::error::AppError;
use awspm::features::switch::SwitchCommand;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_update_timestamp_on_switch() -> Result<(), AppError> {
    let dir = tempdir().map_err(AppError::IoError)?;
    let config_path = dir.path().join("config");
    let metadata_path = dir.path().join(".awspm.yaml");

    // Create Profile A
    let mut file = File::create(&config_path).map_err(AppError::IoError)?;
    writeln!(file, "[profile A]").map_err(AppError::IoError)?;

    // Initial content for metadata
    let mut meta_file = File::create(&metadata_path).map_err(AppError::IoError)?;
    writeln!(meta_file, "profiles: {{}}").map_err(AppError::IoError)?;

    // Use SHELL=echo to avoid interactive blocking
    temp_env::with_vars(
        vec![
            ("AWS_CONFIG_FILE", Some(config_path.as_os_str())),
            ("AWSPM_METADATA_FILE", Some(metadata_path.as_os_str())),
            ("SHELL", Some("echo".as_ref())),
        ],
        || {
            // Ensure Profile Exist
            let store = Store::new().unwrap();
            let profile = store.find_by_name("A").unwrap().expect("Profile A exists");
            assert!(profile.metadata.is_none() || profile.metadata.unwrap().last_used_at.is_none());

            // Action
            let result = SwitchCommand::execute(Some("A".to_string()), false);
            assert!(result.is_ok());

            // Assert
            let store = Store::new().unwrap();
            let profile_after = store.find_by_name("A").unwrap().expect("Profile A exists");
            let metadata = profile_after.metadata.expect("Metadata created");
            assert!(metadata.last_used_at.is_some());
        },
    );

    Ok(())
}
