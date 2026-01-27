use std::env;
use std::process::Command;

#[test]
fn test_exec_unsets_aws_env_vars() {
    // 1. Setup AWSPM binary path
    let cargo_bin = env!("CARGO_BIN_EXE_awspm");

    // 2. Setup a dummy HOME with .aws/config
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let home_path = temp_dir.path();
    let aws_dir = home_path.join(".aws");
    std::fs::create_dir_all(&aws_dir).expect("failed to create .aws dir");

    // Create a dummy config so 'default' profile exists
    std::fs::write(
        aws_dir.join("config"),
        "[profile default]\nregion = us-west-1\n",
    )
    .expect("failed to write aws config");

    // 3. Run `awspm exec` with AWS_ACCESS_KEY_ID set in the parent process
    // usage: awspm exec default -- sh -c "env | grep AWS_"

    #[cfg(unix)]
    let output = Command::new(cargo_bin)
        .env("HOME", home_path)
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .env("AWS_ACCESS_KEY_ID", "SHOULD_BE_REMOVED")
        .env("AWS_SECRET_ACCESS_KEY", "SHOULD_BE_REMOVED")
        .env("AWS_SESSION_TOKEN", "SHOULD_BE_REMOVED")
        .arg("exec")
        .arg("default")
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg("env")
        .output()
        .expect("failed to run awspm exec");

    #[cfg(windows)]
    let output = Command::new(cargo_bin)
        .env("USERPROFILE", home_path) // Windows uses USERPROFILE
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .env("AWS_ACCESS_KEY_ID", "SHOULD_BE_REMOVED")
        .env("AWS_SECRET_ACCESS_KEY", "SHOULD_BE_REMOVED")
        .env("AWS_SESSION_TOKEN", "SHOULD_BE_REMOVED")
        .arg("exec")
        .arg("default")
        .arg("--")
        .arg("cmd")
        .arg("/C")
        .arg("set")
        .output()
        .expect("failed to run awspm exec");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "awspm exec failed.\nStdout: {}\nStderr: {}",
        stdout,
        stderr
    );

    // 4. Verify AWS_PROFILE is set
    assert!(
        stdout.contains("AWS_PROFILE=default"),
        "Should set AWS_PROFILE"
    );

    // 5. Verify AWS credential vars are REMOVED
    assert!(
        !stdout.contains("AWS_ACCESS_KEY_ID=SHOULD_BE_REMOVED"),
        "Should remove AWS_ACCESS_KEY_ID"
    );
    assert!(
        !stdout.contains("AWS_SECRET_ACCESS_KEY=SHOULD_BE_REMOVED"),
        "Should remove AWS_SECRET_ACCESS_KEY"
    );
    // Session token might not happen to be set if not provided, but we checked logic
    assert!(
        !stdout.contains("AWS_SESSION_TOKEN=SHOULD_BE_REMOVED"),
        "Should remove AWS_SESSION_TOKEN"
    );
}
