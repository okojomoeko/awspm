use crate::core::error::AppError;
use crate::core::types::{Metadata, Profile};
use configparser::ini::Ini;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// In-memory representation of the metadata YAML file.
#[derive(Debug, Serialize, Deserialize)]
struct MetadataStore {
    /// Map of profile names to their metadata.
    pub profiles: HashMap<String, Metadata>,
}

/// Unified store for managing access to AWS configuration and Extended Metadata.
#[derive(Clone)]
pub struct Store {
    aws_config_path: PathBuf,
    aws_credentials_path: PathBuf,
    metadata_path: PathBuf,
}

impl Store {
    /// Creates a new store using default paths.
    pub fn new() -> Result<Self, AppError> {
        let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;

        // AWS Config Path
        let aws_config_path = if let Ok(path) = std::env::var("AWS_CONFIG_FILE") {
            PathBuf::from(path)
        } else {
            home.join(".aws").join("config")
        };

        // AWS Credentials Path
        let aws_credentials_path = if let Ok(path) = std::env::var("AWS_SHARED_CREDENTIALS_FILE") {
            PathBuf::from(path)
        } else {
            home.join(".aws").join("credentials")
        };

        // Metadata Path
        let metadata_path = if let Ok(path) = std::env::var("AWSPM_METADATA_FILE") {
            PathBuf::from(path)
        } else {
            home.join(".awspm.yaml")
        };

        Ok(Self {
            aws_config_path,
            aws_credentials_path,
            metadata_path,
        })
    }

    /// Loads all profiles, merging AWS config with Metadata (Global + Local).
    pub fn load_profiles(&self) -> Result<Vec<Profile>, AppError> {
        // 1. Load AWS Config & Credentials
        let config_profiles = self.load_aws_config().unwrap_or_default();
        let cred_profiles = self.load_aws_credentials().unwrap_or_default();

        // Merge Config & Credentials
        let mut profiles_map: HashMap<String, Profile> = HashMap::new();

        for p in config_profiles {
            profiles_map.insert(p.name.clone(), p);
        }

        for p in cred_profiles {
            profiles_map
                .entry(p.name.clone())
                .and_modify(|existing| {
                    existing.source = crate::core::types::ProfileSource::Merged;
                    // Merge other fields if needed, but config usually takes precedence for region/output
                })
                .or_insert(p);
        }

        // 1.5 Load Environment Variables (AWS_PROFILE)
        if let Ok(env_profile) = std::env::var("AWS_PROFILE") {
            profiles_map
                .entry(env_profile.clone())
                .and_modify(|existing| {
                    existing.source = crate::core::types::ProfileSource::Merged;
                })
                .or_insert_with(|| {
                    let mut p = Profile::new(env_profile);
                    p.source = crate::core::types::ProfileSource::Env;
                    p
                });
        }

        // 1.6 Load Raw Environment Variables (AWS_ACCESS_KEY_ID)
        // If these are set, they take precedence over config/credentials/AWS_PROFILE in standard AWS SDKs.
        // We represent this as a special "env-vars" profile.
        if std::env::var("AWS_ACCESS_KEY_ID").is_ok() {
            profiles_map
                .entry("env-vars".to_string())
                .and_modify(|existing| {
                    existing.source = crate::core::types::ProfileSource::Merged;
                })
                .or_insert_with(|| {
                    let mut p = Profile::new("env-vars".to_string());
                    p.source = crate::core::types::ProfileSource::Env;
                    p
                });
        }

        let mut profiles: Vec<Profile> = profiles_map.into_values().collect();

        // 2. Load Metadata (Global + Local Merge)
        let metadata_map = self.load_metadata_map()?;

        // 3. Attach Metadata to Profiles
        for profile in profiles.iter_mut() {
            if let Some(meta) = metadata_map.get(&profile.name) {
                profile.metadata = Some(meta.clone());
            }
        }

        Ok(profiles)
    }

    /// Finds a specific profile by exact name.
    pub fn find_by_name(&self, name: &str) -> Result<Option<Profile>, AppError> {
        // Optimization: For now, we load all. In future we could optimize if needed.
        // Given CLI usage, loading all is fine (100ms scale).
        let profiles = self.load_profiles()?;
        Ok(profiles.into_iter().find(|p| p.name == name))
    }

    /// Finds a profile by name (exact match) or alias (unique match).
    pub fn find_by_name_or_alias(&self, query: &str) -> Result<Option<Profile>, AppError> {
        let profiles = self.load_profiles()?;

        // 1. Exact Name Match
        if let Some(p) = profiles.iter().find(|p| p.name == query) {
            return Ok(Some(p.clone()));
        }

        // 2. Alias Match
        let matched: Vec<&Profile> = profiles
            .iter()
            .filter(|p| {
                if let Some(meta) = &p.metadata {
                    meta.aliases.iter().any(|a| a == query)
                } else {
                    false
                }
            })
            .collect();

        match matched.len() {
            0 => Ok(None),
            1 => Ok(Some(matched[0].clone())),
            _ => Err(AppError::AmbiguousProfile(query.to_string())),
        }
    }

    /// Initializes a new metadata file if it doesn't exist.
    pub fn init_metadata_file(&self) -> Result<(), AppError> {
        if self.metadata_path.exists() {
            println!("File already exists: {:?}", self.metadata_path);
            return Ok(());
        }

        let default_content = "profiles: {}\n";
        fs::write(&self.metadata_path, default_content).map_err(|e| {
            AppError::Unknown(
                anyhow::Error::new(e).context("Failed to write initial metadata file"),
            )
        })?;
        println!("Created metadata file: {:?}", self.metadata_path);
        Ok(())
    }

    /// Saves metadata for a specific profile to the global store.
    pub fn save_metadata(&self, profile_name: &str, metadata: Metadata) -> Result<(), AppError> {
        // Read existing content or start new
        let content = if self.metadata_path.exists() {
            fs::read_to_string(&self.metadata_path)?
        } else {
            "profiles: {}\n".to_string()
        };

        let mut store: MetadataStore = serde_yaml::from_str(&content)?;
        store.profiles.insert(profile_name.to_string(), metadata);
        let new_content = serde_yaml::to_string(&store)?;

        // Backup existing
        if self.metadata_path.exists() {
            let backup_path = self.metadata_path.with_extension("yaml.bak");
            fs::copy(&self.metadata_path, &backup_path).map_err(AppError::IoError)?;
        }

        // Atomic Write
        let temp_path = self.metadata_path.with_extension("yaml.tmp");
        fs::write(&temp_path, new_content.as_bytes()).map_err(AppError::IoError)?;
        fs::rename(&temp_path, &self.metadata_path).map_err(AppError::IoError)?;

        Ok(())
    }

    /// Deletes metadata for a specific profile.
    pub fn delete_metadata(&self, profile_name: &str) -> Result<(), AppError> {
        if !self.metadata_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.metadata_path)?;
        let mut store: MetadataStore = serde_yaml::from_str(&content)?;

        if store.profiles.remove(profile_name).is_some() {
            let new_content = serde_yaml::to_string(&store)?;

            // Backup existing
            if self.metadata_path.exists() {
                let backup_path = self.metadata_path.with_extension("yaml.bak");
                fs::copy(&self.metadata_path, &backup_path).map_err(AppError::IoError)?;
            }

            // Atomic Write
            let temp_path = self.metadata_path.with_extension("yaml.tmp");
            fs::write(&temp_path, new_content.as_bytes()).map_err(AppError::IoError)?;
            fs::rename(&temp_path, &self.metadata_path).map_err(AppError::IoError)?;
        }

        Ok(())
    }

    // --- Internal Helpers ---

    fn load_aws_credentials(&self) -> Result<Vec<Profile>, AppError> {
        if !self.aws_credentials_path.exists() {
            return Ok(Vec::new());
        }

        let mut config = Ini::new_cs();
        let map = config
            .load(&self.aws_credentials_path)
            .map_err(AppError::ConfigLoadError)?;

        let mut profiles = Vec::new();
        for (section, _) in map {
            let name = section; // Credentials file uses [profile-name] or [default] directly? usually just [name]
            // Actually, AWS credentials file sections are just the profile name.

            let mut profile = Profile::new(name);
            profile.source = crate::core::types::ProfileSource::Credentials;
            profiles.push(profile);
        }
        Ok(profiles)
    }

    fn load_aws_config(&self) -> Result<Vec<Profile>, AppError> {
        let mut config = Ini::new_cs();
        // Return empty if file doesn't exist? Or generic error? Original code returned error on parse fail.
        let map = config
            .load(&self.aws_config_path)
            .map_err(AppError::ConfigLoadError)?;

        let mut profiles = Vec::new();
        for (section, properties) in map {
            let name = if section == "default" {
                "default".to_string()
            } else if section.starts_with("profile ") {
                section.trim_start_matches("profile ").to_string()
            } else {
                continue;
            };

            let mut profile = Profile::new(name);
            profile.region = properties.get("region").cloned().flatten();
            profile.output = properties.get("output").cloned().flatten();
            profile.sso_session = properties.get("sso_session").cloned().flatten();
            profile.sso_start_url = properties.get("sso_start_url").cloned().flatten();

            profiles.push(profile);
        }

        // Sort for deterministic behavior
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    fn load_metadata_map(&self) -> Result<HashMap<String, Metadata>, AppError> {
        let mut profiles = HashMap::new();

        // 1. Load Global Config
        if self.metadata_path.exists() {
            let content = fs::read_to_string(&self.metadata_path)?;
            let store: MetadataStore = serde_yaml::from_str(&content)?;
            profiles.extend(store.profiles);
        }

        // 2. Load Local Config (from CWD) & Merge
        if let Ok(cwd) = std::env::current_dir() {
            let local_path = cwd.join(".awspm.yaml");
            if local_path.exists()
                && local_path != self.metadata_path
                && let Ok(content) = fs::read_to_string(&local_path)
                && let Ok(local_store) = serde_yaml::from_str::<MetadataStore>(&content)
            {
                self.merge_local_config(&mut profiles, local_store);
            }
        }
        Ok(profiles)
    }

    // --- Sync Check Logic ---

    /// Checks for profiles that are out of sync between AWS sources and local metadata.
    pub fn check_sync_status(&self) -> Result<SyncStatus, AppError> {
        let profiles = self.load_profiles()?;
        let metadata_map = self.load_metadata_map()?;

        let mut untracked = Vec::new();
        let mut orphaned = Vec::new();

        // 1. Find Untracked (In Profile Sources but NO Metadata)
        for profile in &profiles {
            if profile.name == "env-vars" {
                continue;
            }
            if profile.metadata.is_none() {
                untracked.push(profile.clone());
            }
        }

        // 2. Find Orphaned (In Metadata but NO Profile Source)
        // We need to check if every key in metadata_map exists in `profiles`.
        // Note: `load_profiles` already merges metadata.
        // If a profile exists ONLY in metadata, `load_profiles` normally wouldn't return it
        // because `load_profiles` iterates over AWS Config/Creds first.

        let profile_names: Vec<String> = profiles.iter().map(|p| p.name.clone()).collect();
        for name in metadata_map.keys() {
            if !profile_names.contains(name) {
                orphaned.push(name.clone());
            }
        }

        Ok(SyncStatus {
            untracked,
            orphaned,
        })
    }

    fn merge_local_config(
        &self,
        global_profiles: &mut HashMap<String, Metadata>,
        local_store: MetadataStore,
    ) {
        for (local_key, local_meta) in local_store.profiles {
            if let Some(global_meta) = global_profiles.get_mut(&local_key) {
                // Direct Override
                *global_meta = local_meta;
            } else {
                // Alias Match Logic
                let target_name = global_profiles.iter().find_map(|(name, meta)| {
                    if meta.aliases.contains(&local_key) {
                        Some(name.clone())
                    } else {
                        None
                    }
                });

                if let Some(name) = target_name
                    && let Some(target_meta) = global_profiles.get_mut(&name)
                {
                    // Merge fields
                    if !local_meta.tags.is_empty() {
                        target_meta.tags = local_meta.tags;
                    }
                    if local_meta.note.is_some() {
                        target_meta.note = local_meta.note;
                    }
                    if local_meta.region.is_some() {
                        target_meta.region = local_meta.region;
                    }
                }
            }
        }
    }
}
/// Status of synchronization between AWS config/credentials and local metadata.
#[derive(Debug)]
pub struct SyncStatus {
    /// Profiles that exist in AWS Config/Credentials but are not tracked in `awspm` metadata.
    pub untracked: Vec<Profile>,
    /// Profiles that have metadata in `~/.awspm.yaml` but do not exist in any AWS source.
    pub orphaned: Vec<String>,
}
