use std::path::Path;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: &'static str,
    pub description: &'static str,
    pub targets: &'static [&'static str],
    pub markers: &'static [&'static str],
}

impl Profile {
    /// Check if the parent directory of the found target contains at least one marker file.
    pub fn has_marker(&self, target_path: &Path) -> bool {
        let Some(parent) = target_path.parent() else {
            return false;
        };

        self.markers
            .iter()
            .any(|marker| parent.join(marker).exists())
    }
}

pub const PROFILE_NODE: Profile = Profile {
    name: "node",
    description: "Node.js (node_modules)",
    targets: &["node_modules"],
    markers: &["package.json"],
};

pub const PROFILE_RUST: Profile = Profile {
    name: "rust",
    description: "Rust (target)",
    targets: &["target"],
    markers: &["Cargo.toml"],
};

pub const ALL_PROFILES: &[&Profile] = &[&PROFILE_NODE, &PROFILE_RUST];

pub fn get_profiles_by_names(names: &[&str]) -> Vec<&'static Profile> {
    ALL_PROFILES
        .iter()
        .copied()
        .filter(|profile| names.contains(&profile.name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_has_marker_returns_true_when_marker_exists() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("my-project");
        let target = project.join("node_modules");
        fs::create_dir_all(&target).unwrap();
        fs::write(project.join("package.json"), "{}").unwrap();

        assert!(PROFILE_NODE.has_marker(&target));
    }

    #[test]
    fn test_has_marker_returns_false_without_marker() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("my-project");
        let target = project.join("node_modules");
        fs::create_dir_all(&target).unwrap();

        assert!(!PROFILE_NODE.has_marker(&target));
    }

    #[test]
    fn test_has_marker_rust_profile() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("my-rust-project");
        let target = project.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(project.join("Cargo.toml"), "[package]").unwrap();

        assert!(PROFILE_RUST.has_marker(&target));
    }

    #[test]
    fn test_get_profiles_by_names_single() {
        let profiles = get_profiles_by_names(&["node"]);
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "node");
    }

    #[test]
    fn test_get_profiles_by_names_multiple() {
        let profiles = get_profiles_by_names(&["node", "rust"]);
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_get_profiles_by_names_unknown_ignored() {
        let profiles = get_profiles_by_names(&["unknown"]);
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_all_profiles_contains_both() {
        assert_eq!(ALL_PROFILES.len(), 2);
        assert_eq!(ALL_PROFILES[0].name, "node");
        assert_eq!(ALL_PROFILES[1].name, "rust");
    }
}
