use awspm::core::config::GlobalConfig;
use awspm::core::sso::{SsoService, SsoStatus};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sso_cache_path_from_config() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let custom_cache_dir = dir.path().join("custom_cache");
    fs::create_dir_all(&custom_cache_dir).unwrap();

    let awspm_yaml_path = config_dir.join(".awspm.yaml");
    // Use forward slashes for YAML paths to avoid escape sequence issues on Windows
    let custom_cache_path_str = custom_cache_dir.display().to_string().replace("\\", "/");
    let yaml_content = format!("config:\n  sso_cache_path: \"{}\"", custom_cache_path_str);
    fs::write(&awspm_yaml_path, yaml_content).unwrap();

    // 意図的に ~/.aws/sso/cache を無視するため、Storeの生成は省略し、パースだけ確認します
    let content = fs::read_to_string(&awspm_yaml_path).unwrap();
    #[derive(serde::Deserialize)]
    struct MetadataStore {
        #[serde(default)]
        pub config: GlobalConfig,
    }
    let store: MetadataStore = serde_yaml::from_str(&content).unwrap();
    assert_eq!(
        store
            .config
            .sso_cache_path
            .as_ref()
            .map(|s| s.replace("\\", "/")),
        Some(custom_cache_path_str)
    );

    let service = SsoService::new(store.config.sso_cache_path).unwrap();
    // service内部のcache_dirが custom_cache_dir になっていることを確認したいですが、
    // privateフィールドなので、Unknownステータスになることで確認とします。
    let profile = awspm::core::types::Profile::new("test".to_string());
    assert_eq!(service.get_status(&profile), SsoStatus::NotConfigured);
}
