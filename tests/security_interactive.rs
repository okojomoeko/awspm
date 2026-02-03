use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_sensitive_exec_aborts_non_interactive() {
    // 1. Setup environment
    let temp_dir = tempfile::tempdir().unwrap();
    let home_path = temp_dir.path();
    let aws_dir = home_path.join(".aws");
    std::fs::create_dir_all(&aws_dir).unwrap();

    // AWS Config with a sensitive name "production"
    std::fs::write(
        aws_dir.join("config"),
        "[profile production]\nregion=us-east-1",
    )
    .unwrap();

    // Use env!("CARGO_BIN_EXE_awspm") which is set by cargo at build time.
    // This avoids the deprecated Command::cargo_bin lookup.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_awspm"));
    cmd.env("HOME", home_path)
        .env("USERPROFILE", home_path)
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .env("AWSPM_METADATA_FILE", home_path.join(".awspm.yaml"))
        .arg("exec")
        .arg("production")
        .arg("--")
        .arg("echo")
        .arg("DANGEROUS_ACTION");

    // Scenario 1: Non-interactive execution (e.g. from script/CI)
    // Even if we pipe "y", it SHOULD fail because we are not in a TTY
    // and haven't implemented a --yes flag yet (or we assume strict security).
    //
    // The previous e2e failure showed that `echo "n" | awspm ...` SUCCEEDED.
    // That means `dialoguer` in non-tty mode might be defaulting to true,
    // or consuming input in a way we don't control well.
    //
    // We want to enforce: No TTY -> Auto Deny (unless explicit flag, which is not passed here).

    cmd.write_stdin("y\n");

    // Current expectation (before fix): It unfortunately succeeds (exit code 0).
    // Desired expectation (after fix): It fails (exit code != 0) and prints "Aborted".

    // We write the assertion for the DESIRED state (Failure).
    // This test should FAIL now (showing it passes unexpectedly), and PASS after we fix the code.
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("DANGEROUS_ACTION").not());
}

#[test]
fn test_sensitive_exec_bypasses_with_yes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_path = temp_dir.path();
    let aws_dir = home_path.join(".aws");
    std::fs::create_dir_all(&aws_dir).unwrap();

    std::fs::write(
        aws_dir.join("config"),
        "[profile production]\nregion=us-east-1",
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_awspm"));
    cmd.env("HOME", home_path)
        .env("USERPROFILE", home_path) // Windows
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .env("AWSPM_METADATA_FILE", home_path.join(".awspm.yaml"))
        .arg("exec")
        .arg("production")
        .arg("--yes") // <--- The flag being tested
        .arg("--")
        .arg("echo")
        .arg("ALLOWED_ACTION");

    // Even without stdin input, it should pass because of --yes
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ALLOWED_ACTION"));
}
