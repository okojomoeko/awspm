use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_custom_guardrails_policy() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_path = temp_dir.path();
    let aws_dir = home_path.join(".aws");
    std::fs::create_dir_all(&aws_dir).unwrap();

    // 1. Create a "production" profile (normally sensitive)
    std::fs::write(
        aws_dir.join("config"),
        "[profile production]\nregion=us-east-1\n[profile custom-sensitive]\nregion=us-west-1",
    )
    .unwrap();

    // 2. Create a custom policy that IGNORES "production" but FLAGS "custom-sensitive"
    // The "sensitive_keywords" list replaces the default list.
    let config_content = r#"
config:
  sensitive_keywords: ["custom-sensitive"]
profiles: {}
"#;
    std::fs::write(home_path.join(".awspm.yaml"), config_content).unwrap();

    // 3. Test "production" -> Should SUCCEED (no longer sensitive)
    let mut cmd_prod = Command::new(env!("CARGO_BIN_EXE_awspm"));
    cmd_prod
        .env("HOME", home_path)
        .env("USERPROFILE", home_path)
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .env("AWSPM_METADATA_FILE", home_path.join(".awspm.yaml"))
        .arg("exec")
        .arg("production")
        .arg("--")
        .arg("echo")
        .arg("SAFE");

    // We pipe "n" just in case, but it shouldn't prompt because it's not sensitive anymore.
    // If it WAS sensitive, "n" would abort. If it's NOT, it runs.
    // Since we are non-interactive here (no TTY), a sensitive profile would FAIL if not using --yes.
    // So if this SUCCEEDS, it means it was NOT treated as sensitive.
    cmd_prod.write_stdin("n\n");
    cmd_prod
        .assert()
        .success()
        .stdout(predicate::str::contains("SAFE"));

    // 4. Test "custom-sensitive" -> Should FAIL (now sensitive)
    let mut cmd_custom = Command::new(env!("CARGO_BIN_EXE_awspm"));
    cmd_custom
        .env("HOME", home_path)
        .env("USERPROFILE", home_path)
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .env("AWSPM_METADATA_FILE", home_path.join(".awspm.yaml"))
        .arg("exec")
        .arg("custom-sensitive")
        .arg("--")
        .arg("echo")
        .arg("BAD");

    // It should fail in non-interactive mode for sensitive profile
    cmd_custom.assert().failure();
}
