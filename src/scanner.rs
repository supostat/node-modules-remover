use crate::profiles::Profile;
use anyhow::Result;
use bytesize::ByteSize;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Type alias for the progress callback to reduce complexity
pub type ProgressCallback = Arc<Mutex<dyn FnMut(&str) + Send>>;

#[derive(Debug, Clone)]
pub struct CleanEntry {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: Option<SystemTime>,
}

impl CleanEntry {
    pub fn size_human(&self) -> String {
        ByteSize::b(self.size).to_string()
    }

    pub fn last_modified_human(&self) -> String {
        match self.last_modified {
            Some(time) => {
                if let Ok(duration) = time.elapsed() {
                    let secs = duration.as_secs();
                    if secs < 60 {
                        format!("{}s ago", secs)
                    } else if secs < 3600 {
                        format!("{}m ago", secs / 60)
                    } else if secs < 86400 {
                        format!("{}h ago", secs / 3600)
                    } else {
                        format!("{}d ago", secs / 86400)
                    }
                } else {
                    "Unknown".to_string()
                }
            }
            None => "Unknown".to_string(),
        }
    }
}

/// Scan for target directories matching the given profiles.
/// Only finds first-level occurrences per profile target.
/// Validates that a marker file exists in the parent directory before including.
pub fn scan_for_entries(
    root: &Path,
    profiles: &[&Profile],
    progress_callback: Option<ProgressCallback>,
) -> Result<Vec<CleanEntry>> {
    let entries = Arc::new(Mutex::new(Vec::new()));

    scan_directory(root, profiles, &entries, &progress_callback)?;

    let mut result = entries.lock().expect("entries mutex poisoned").clone();
    result.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(result)
}

fn scan_directory(
    dir: &Path,
    profiles: &[&Profile],
    entries: &Arc<Mutex<Vec<CleanEntry>>>,
    progress_callback: &Option<ProgressCallback>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    // Update progress
    if let Some(callback) = progress_callback {
        if let Ok(mut cb) = callback.lock() {
            cb(dir.to_string_lossy().as_ref());
        }
    }

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()), // Skip directories we can't read
    };

    let subdirs: Vec<PathBuf> = read_dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();

    let mut dirs_to_recurse = Vec::new();

    for path in subdirs {
        let is_symlink = path
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);

        if is_symlink {
            continue;
        }

        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let matched_by_profile = profiles
            .iter()
            .any(|profile| profile.targets.contains(&dir_name) && profile.has_marker(&path));

        if matched_by_profile {
            let size = calculate_dir_size(&path);
            let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());

            entries
                .lock()
                .expect("entries mutex poisoned")
                .push(CleanEntry {
                    path,
                    size,
                    last_modified,
                });
        } else {
            dirs_to_recurse.push(path);
        }
    }

    // Recurse into non-matched directories in parallel
    dirs_to_recurse.par_iter().for_each(|path| {
        let _ = scan_directory(path, profiles, entries, progress_callback);
    });

    Ok(())
}

fn calculate_dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

pub fn delete_entry(path: &Path) -> Result<()> {
    let metadata = path
        .symlink_metadata()
        .map_err(|e| anyhow::anyhow!("Failed to read metadata for '{}': {}", path.display(), e))?;

    if metadata.file_type().is_symlink() {
        anyhow::bail!("Refusing to delete '{}': path is a symlink", path.display());
    }

    fs::remove_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{ALL_PROFILES, PROFILE_NODE, PROFILE_RUST};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_finds_first_level_node_modules_with_marker() {
        let temp = tempdir().unwrap();
        let project1 = temp.path().join("project1");
        let nm1 = project1.join("node_modules");
        let nested_nm = nm1.join("some_package").join("node_modules");

        fs::create_dir_all(&nested_nm).unwrap();
        fs::write(project1.join("package.json"), "{}").unwrap();

        let profiles: Vec<&Profile> = vec![&PROFILE_NODE];
        let results = scan_for_entries(temp.path(), &profiles, None).unwrap();

        // Should only find project1/node_modules, not the nested one
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, nm1);
    }

    #[test]
    fn test_skips_node_modules_without_marker() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("project");
        let nm = project.join("node_modules");

        fs::create_dir_all(&nm).unwrap();
        // No package.json — should not be found

        let profiles: Vec<&Profile> = vec![&PROFILE_NODE];
        let results = scan_for_entries(temp.path(), &profiles, None).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_finds_multiple_projects_with_markers() {
        let temp = tempdir().unwrap();

        let project1 = temp.path().join("project1");
        let project1_nm = project1.join("node_modules");
        let project2 = temp.path().join("project2");
        let project2_nm = project2.join("node_modules");

        fs::create_dir_all(&project1_nm).unwrap();
        fs::write(project1.join("package.json"), "{}").unwrap();
        fs::create_dir_all(&project2_nm).unwrap();
        fs::write(project2.join("package.json"), "{}").unwrap();

        let profiles: Vec<&Profile> = vec![&PROFILE_NODE];
        let results = scan_for_entries(temp.path(), &profiles, None).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_finds_rust_target_with_marker() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("rust-project");
        let target = project.join("target");

        fs::create_dir_all(&target).unwrap();
        fs::write(project.join("Cargo.toml"), "[package]").unwrap();

        let profiles: Vec<&Profile> = vec![&PROFILE_RUST];
        let results = scan_for_entries(temp.path(), &profiles, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, target);
    }

    #[test]
    fn test_skips_rust_target_without_marker() {
        let temp = tempdir().unwrap();
        let project = temp.path().join("rust-project");
        let target = project.join("target");

        fs::create_dir_all(&target).unwrap();
        // No Cargo.toml

        let profiles: Vec<&Profile> = vec![&PROFILE_RUST];
        let results = scan_for_entries(temp.path(), &profiles, None).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_all_profiles_find_both_node_and_rust() {
        let temp = tempdir().unwrap();

        let node_project = temp.path().join("node-project");
        let nm = node_project.join("node_modules");
        fs::create_dir_all(&nm).unwrap();
        fs::write(node_project.join("package.json"), "{}").unwrap();

        let rust_project = temp.path().join("rust-project");
        let target = rust_project.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(rust_project.join("Cargo.toml"), "[package]").unwrap();

        let profiles: Vec<&Profile> = ALL_PROFILES.to_vec();
        let results = scan_for_entries(temp.path(), &profiles, None).unwrap();

        assert_eq!(results.len(), 2);
        let paths: Vec<&Path> = results.iter().map(|e| e.path.as_path()).collect();
        assert!(paths.contains(&nm.as_path()));
        assert!(paths.contains(&target.as_path()));
    }
}
