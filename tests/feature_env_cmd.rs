use std::process::Command;

#[test]
fn test_env_command_outputs_exports() {
    let cargo_bin = env!("CARGO_BIN_EXE_awspm");
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let home_path = temp_dir.path();
    let aws_dir = home_path.join(".aws");
    std::fs::create_dir_all(&aws_dir).expect("failed to create .aws dir");

    // Create config
    std::fs::write(
        aws_dir.join("config"),
        "[profile my-env-test]\nregion = us-env-1\n",
    )
    .expect("failed to write config");

    let output = Command::new(cargo_bin)
        .env("HOME", home_path)
        .env("AWS_CONFIG_FILE", aws_dir.join("config"))
        .arg("env")
        .arg("my-env-test")
        .output()
        .expect("failed to run awspm env");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check output format
    assert!(stdout.contains("export AWS_PROFILE=my-env-test"));
    assert!(stdout.contains("export AWS_REGION=us-env-1"));
}
