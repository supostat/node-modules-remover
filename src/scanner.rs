use anyhow::Result;
use bytesize::ByteSize;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct NodeModulesEntry {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: Option<SystemTime>,
}

impl NodeModulesEntry {
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

/// Scan for node_modules directories, only finding first-level occurrences.
/// When a node_modules is found, we don't recurse into it to find nested ones.
pub fn scan_for_node_modules(
    root: &Path,
    progress_callback: Option<Arc<Mutex<dyn FnMut(&str) + Send>>>,
) -> Result<Vec<NodeModulesEntry>> {
    let entries = Arc::new(Mutex::new(Vec::new()));

    scan_directory(root, &entries, &progress_callback)?;

    let result = entries.lock().unwrap().clone();
    Ok(result)
}

fn scan_directory(
    dir: &Path,
    entries: &Arc<Mutex<Vec<NodeModulesEntry>>>,
    progress_callback: &Option<Arc<Mutex<dyn FnMut(&str) + Send>>>,
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

    // Check if any subdirectory is node_modules
    let mut dirs_to_recurse = Vec::new();

    for path in subdirs {
        if path
            .file_name()
            .map(|n| n == "node_modules")
            .unwrap_or(false)
        {
            // Found a node_modules directory - add it and DON'T recurse into it
            let size = calculate_dir_size(&path);
            let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());

            entries.lock().unwrap().push(NodeModulesEntry {
                path,
                size,
                last_modified,
            });
        } else {
            // Not a node_modules - we should recurse into it
            dirs_to_recurse.push(path);
        }
    }

    // Recurse into non-node_modules directories in parallel
    dirs_to_recurse.par_iter().for_each(|path| {
        let _ = scan_directory(path, entries, progress_callback);
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

pub fn delete_node_modules(path: &Path) -> Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_finds_first_level_node_modules() {
        let temp = tempdir().unwrap();
        let project1 = temp.path().join("project1");
        let nm1 = project1.join("node_modules");
        let nested_nm = nm1.join("some_package").join("node_modules");

        fs::create_dir_all(&nested_nm).unwrap();

        let results = scan_for_node_modules(temp.path(), None).unwrap();

        // Should only find project1/node_modules, not the nested one
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, nm1);
    }

    #[test]
    fn test_finds_multiple_projects() {
        let temp = tempdir().unwrap();

        let project1_nm = temp.path().join("project1").join("node_modules");
        let project2_nm = temp.path().join("project2").join("node_modules");

        fs::create_dir_all(&project1_nm).unwrap();
        fs::create_dir_all(&project2_nm).unwrap();

        let results = scan_for_node_modules(temp.path(), None).unwrap();

        assert_eq!(results.len(), 2);
    }
}
