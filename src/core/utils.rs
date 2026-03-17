use std::path::{Path, PathBuf};

/// Expands the tilde `~` in a given path string to the current user's home directory.
/// If the path does not start with `~`, it returns the path as-is.
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let path_ref = path.as_ref();
    if let Some(path_str) = path_ref.to_str()
        && (path_str.starts_with("~/") || path_str == "~")
        && let Some(mut home) = dirs::home_dir()
    {
        if path_str == "~" {
            return home;
        }
        home.push(path_str.trim_start_matches("~/"));
        return home;
    }
    path_ref.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expand_tilde("~"), home);
            assert_eq!(expand_tilde("~/some/path"), home.join("some/path"));
        }

        assert_eq!(
            expand_tilde("/absolute/path"),
            PathBuf::from("/absolute/path")
        );
        assert_eq!(
            expand_tilde("relative/path"),
            PathBuf::from("relative/path")
        );
    }
}
