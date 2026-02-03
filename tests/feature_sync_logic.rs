use awspm::core::config::Store;
use awspm::core::types::ProfileSource;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_aws_credentials() {
    let dir = tempdir().unwrap();
    let credentials_path = dir.path().join("credentials");

    // Create dummy credentials file
    let content = "[default]\naws_access_key_id=foo\naws_secret_access_key=bar\n\n[dev-profile]\naws_access_key_id=test\naws_secret_access_key=test";
    fs::write(&credentials_path, content).unwrap();

    // Use temp_env to strictly control environment variables
    temp_env::with_vars(
        [
            (
                "AWS_SHARED_CREDENTIALS_FILE",
                Some(credentials_path.to_str().unwrap()),
            ),
            ("AWS_CONFIG_FILE", Some("/non/existent/config")),
            ("AWS_PROFILE", None),
        ],
        || {
            let store = Store::new().unwrap();
            let profiles = store.load_profiles().unwrap();

            // "default" and "dev-profile" should be present from credentials
            assert!(
                profiles
                    .iter()
                    .any(|p| p.name == "default" && p.source == ProfileSource::Credentials)
            );
            assert!(
                profiles
                    .iter()
                    .any(|p| p.name == "dev-profile" && p.source == ProfileSource::Credentials)
            );
        },
    );
}

#[test]
fn test_check_sync_status_orphans_and_untracked() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config");
    let metadata_path = dir.path().join(".awspm.yaml");

    // 1. Setup AWS Config (Sources)
    fs::write(&config_path, "[profile active_user]\nregion=us-east-1").unwrap();

    // 2. Setup Metadata (Orphans)
    fs::write(
        &metadata_path,
        "profiles:\n  orphaned_user:\n    tags: [old]\n",
    )
    .unwrap();

    temp_env::with_vars(
        [
            ("AWS_CONFIG_FILE", Some(config_path.to_str().unwrap())),
            ("AWSPM_METADATA_FILE", Some(metadata_path.to_str().unwrap())),
            ("AWS_SHARED_CREDENTIALS_FILE", Some("/non/existent/creds")),
            ("AWS_PROFILE", None),
        ],
        || {
            let store = Store::new().unwrap();
            let status = store.check_sync_status().unwrap();

            // Verify Untracked (active_user is in config but not metadata)
            assert_eq!(status.untracked.len(), 1);
            assert_eq!(status.untracked[0].name, "active_user");

            // Verify Orphaned (orphaned_user is in metadata but not config)
            assert_eq!(status.orphaned.len(), 1);
            assert_eq!(status.orphaned[0], "orphaned_user");
        },
    );
}

#[test]
fn test_ignore_env_vars_profile_in_sync() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config");
    let metadata_path = dir.path().join(".awspm.yaml");

    // Empty config and metadata
    fs::write(&config_path, "").unwrap();
    fs::write(&metadata_path, "profiles: {}").unwrap();

    temp_env::with_vars(
        [
            ("AWS_CONFIG_FILE", Some(config_path.to_str().unwrap())),
            ("AWSPM_METADATA_FILE", Some(metadata_path.to_str().unwrap())),
            ("AWS_SHARED_CREDENTIALS_FILE", Some("/non/existent/creds")),
            ("AWS_ACCESS_KEY_ID", Some("fake-key")),
            ("AWS_SECRET_ACCESS_KEY", Some("fake-secret")),
        ],
        || {
            let store = Store::new().unwrap();
            let status = store.check_sync_status().unwrap();

            // Should NOT be untracked
            assert!(
                status.untracked.is_empty(),
                "env-vars should be ignored in sync check"
            );
        },
    );
}
