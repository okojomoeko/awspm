use awspm::core::error::AppError;
use awspm::features::pin::PinCommand;
use std::fs::File;
use std::io::{Read, Write};
use tempfile::tempdir;

#[test]
fn test_awspm_local_creates_envrc() -> Result<(), AppError> {
    let dir = tempdir().map_err(AppError::IoError)?;
    let config_path = dir.path().join("config");
    let metadata_path = dir.path().join(".awspm.yaml");

    // Setup: Create 1 profile
    let mut file = File::create(&config_path).map_err(AppError::IoError)?;
    writeln!(file, "[profile my-project]").map_err(AppError::IoError)?;
    writeln!(file, "region = us-east-1").map_err(AppError::IoError)?;

    // Initialize metadata file
    let mut meta_file = File::create(&metadata_path).map_err(AppError::IoError)?;
    writeln!(meta_file, "profiles: {{}}").map_err(AppError::IoError)?;

    temp_env::with_vars(
        vec![
            ("AWS_CONFIG_FILE", Some(config_path.as_os_str())),
            ("AWSPM_METADATA_FILE", Some(metadata_path.as_os_str())),
            // Mock interaction? PinCommand uses RealInteractionHandler.
            // "my-project" is not sensitive, so no interaction.
        ],
        || {
            // We use run() with explicit CWD
            PinCommand::run(
                Some("my-project".to_string()),
                dir.path().to_path_buf(),
                false,
            )
            .unwrap();

            // Verify
            let envrc_path = dir.path().join(".envrc");
            assert!(envrc_path.exists());
            let mut content = String::new();
            File::open(&envrc_path)
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
            assert!(content.contains("export AWS_PROFILE=my-project"));
        },
    );

    Ok(())
}

#[test]
fn test_awspm_local_updates_envrc() -> Result<(), AppError> {
    let dir = tempdir().map_err(AppError::IoError)?;
    let config_path = dir.path().join("config");
    let metadata_path = dir.path().join(".awspm.yaml");

    let mut file = File::create(&config_path).map_err(AppError::IoError)?;
    writeln!(file, "[profile my-project]").map_err(AppError::IoError)?;
    writeln!(file, "[profile other-project]").map_err(AppError::IoError)?;
    writeln!(file, "region = us-east-1").map_err(AppError::IoError)?; // need region or not purely required but good practice

    let mut meta_file = File::create(&metadata_path).map_err(AppError::IoError)?;
    writeln!(meta_file, "profiles: {{}}").map_err(AppError::IoError)?;

    // Pre-create .envrc
    let envrc_path = dir.path().join(".envrc");
    let mut file = File::create(&envrc_path).map_err(AppError::IoError)?;
    writeln!(file, "export FOO=bar").map_err(AppError::IoError)?;
    writeln!(file, "export AWS_PROFILE=old-project").map_err(AppError::IoError)?;

    temp_env::with_vars(
        vec![
            ("AWS_CONFIG_FILE", Some(config_path.as_os_str())),
            ("AWSPM_METADATA_FILE", Some(metadata_path.as_os_str())),
        ],
        || {
            let result = PinCommand::run(
                Some("other-project".to_string()),
                dir.path().to_path_buf(),
                false,
            );
            assert!(result.is_ok());

            let mut content = String::new();
            File::open(&envrc_path)
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();

            assert!(content.contains("export FOO=bar"));
            assert!(content.contains("export AWS_PROFILE=other-project"));
            assert!(!content.contains("export AWS_PROFILE=old-project"));
        },
    );

    Ok(())
}
