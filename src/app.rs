use crate::profiles::ALL_PROFILES;
use crate::scanner::CleanEntry;
use ratatui::widgets::ListState;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Welcome,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Size,
    Date,
    Name,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            SortMode::Size => "Size",
            SortMode::Date => "Date",
            SortMode::Name => "Name",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProfileFilter {
    All,
    Only(&'static str),
}

pub struct SelectionState {
    indices: HashSet<usize>,
    total_size: u64,
}

impl SelectionState {
    fn new() -> Self {
        Self {
            indices: HashSet::new(),
            total_size: 0,
        }
    }

    pub fn toggle(&mut self, index: usize, entry_size: u64) {
        if self.indices.contains(&index) {
            self.indices.remove(&index);
            self.total_size -= entry_size;
        } else {
            self.indices.insert(index);
            self.total_size += entry_size;
        }
    }

    pub fn clear(&mut self) {
        self.indices.clear();
        self.total_size = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn contains(&self, index: usize) -> bool {
        self.indices.contains(&index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.indices.iter()
    }
}

pub struct ScanState {
    pub active: bool,
    pub path: String,
    pub current_path: String,
}

impl ScanState {
    fn new() -> Self {
        Self {
            active: false,
            path: String::new(),
            current_path: String::new(),
        }
    }

    pub fn start(&mut self, path: String) {
        self.active = true;
        self.path = path;
        self.current_path.clear();
    }

    pub fn finish(&mut self) {
        self.active = false;
        self.current_path.clear();
    }
}

pub struct DeleteState {
    pub active: bool,
    pub progress: (usize, usize),
    pub current_path: String,
}

impl DeleteState {
    fn new() -> Self {
        Self {
            active: false,
            progress: (0, 0),
            current_path: String::new(),
        }
    }

    pub fn start(&mut self, total: usize) {
        self.active = true;
        self.progress = (0, total);
    }

    pub fn update(&mut self, current: usize, path: String) {
        self.progress = (current, self.progress.1);
        self.current_path = path;
    }

    pub fn finish(&mut self) {
        self.active = false;
    }
}

pub struct App {
    pub entries: Vec<CleanEntry>,
    pub list_state: ListState,
    pub selection: SelectionState,
    pub scan: ScanState,
    pub delete: DeleteState,
    pub mode: AppMode,
    pub input_path: String,
    pub cursor_position: usize,
    pub show_help: bool,
    pub show_confirm: bool,
    pub message: Option<String>,
    pub should_quit: bool,
    pub total_size: u64,
    pub scan_root: String,
    pub sort_mode: SortMode,
    pub filter_mode: ProfileFilter,
}

impl App {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            list_state: ListState::default(),
            selection: SelectionState::new(),
            scan: ScanState::new(),
            delete: DeleteState::new(),
            mode: AppMode::Welcome,
            input_path: String::new(),
            cursor_position: 0,
            show_help: false,
            show_confirm: false,
            message: None,
            should_quit: false,
            total_size: 0,
            scan_root: String::new(),
            sort_mode: SortMode::Size,
            filter_mode: ProfileFilter::All,
        }
    }

    pub fn set_entries(&mut self, entries: Vec<CleanEntry>, scan_root: String) {
        self.total_size = entries.iter().map(|e| e.size).sum();
        self.entries = entries;
        self.scan_root = scan_root;
        self.sort_entries();
        self.selection.clear();
        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| match &self.filter_mode {
                ProfileFilter::All => true,
                ProfileFilter::Only(name) => entry.profile_name == *name,
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn sort_entries(&mut self) {
        match self.sort_mode {
            SortMode::Size => self.entries.sort_by(|a, b| b.size.cmp(&a.size)),
            SortMode::Date => self.entries.sort_by(|a, b| {
                let time_a = a.last_modified.unwrap_or(std::time::UNIX_EPOCH);
                let time_b = b.last_modified.unwrap_or(std::time::UNIX_EPOCH);
                time_b.cmp(&time_a)
            }),
            SortMode::Name => self
                .entries
                .sort_by(|a, b| a.path.as_path().cmp(b.path.as_path())),
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Size => SortMode::Date,
            SortMode::Date => SortMode::Name,
            SortMode::Name => SortMode::Size,
        };
        self.sort_entries();
        self.selection.clear();
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn cycle_filter(&mut self) {
        let profile_names: Vec<&'static str> = ALL_PROFILES.iter().map(|p| p.name).collect();

        self.filter_mode = match &self.filter_mode {
            ProfileFilter::All => ProfileFilter::Only(profile_names[0]),
            ProfileFilter::Only(current) => {
                let current_idx = profile_names.iter().position(|n| n == current).unwrap_or(0);
                if current_idx + 1 < profile_names.len() {
                    ProfileFilter::Only(profile_names[current_idx + 1])
                } else {
                    ProfileFilter::All
                }
            }
        };
        self.selection.clear();
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= visible.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    visible.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_select(&mut self) {
        let visible = self.visible_indices();
        if let Some(visible_pos) = self.list_state.selected() {
            if let Some(&actual_idx) = visible.get(visible_pos) {
                self.selection
                    .toggle(actual_idx, self.entries[actual_idx].size);
            }
        }
    }

    pub fn select_all(&mut self) {
        let visible = self.visible_indices();
        self.selection.clear();
        for &idx in &visible {
            self.selection.toggle(idx, self.entries[idx].size);
        }
    }

    pub fn deselect_all(&mut self) {
        self.selection.clear();
    }

    pub fn remove_deleted(&mut self, deleted_indices: &[usize]) {
        let mut sorted_indices: Vec<usize> = deleted_indices.to_vec();
        sorted_indices.sort_by(|a, b| b.cmp(a));

        for i in sorted_indices {
            if i < self.entries.len() {
                self.entries.remove(i);
            }
        }

        self.selection.clear();
        self.total_size = self.entries.iter().map(|e| e.size).sum();

        let visible_count = self.visible_indices().len();
        if visible_count == 0 {
            self.list_state.select(None);
        } else if let Some(current) = self.list_state.selected() {
            if current >= visible_count {
                self.list_state.select(Some(visible_count - 1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_entry(name: &str, size: u64, profile: &'static str) -> CleanEntry {
        CleanEntry {
            path: PathBuf::from(format!("/tmp/{}", name)),
            size,
            last_modified: None,
            profile_name: profile,
        }
    }

    fn app_with_mixed_entries() -> App {
        let mut app = App::new();
        let entries = vec![
            make_entry("node_project/node_modules", 300, "node"),
            make_entry("rust_project/target", 200, "rust"),
            make_entry("another_node/node_modules", 100, "node"),
        ];
        app.set_entries(entries, "/tmp".to_string());
        app
    }

    #[test]
    fn test_visible_indices_all() {
        let app = app_with_mixed_entries();
        let visible = app.visible_indices();
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn test_visible_indices_node_filter() {
        let mut app = app_with_mixed_entries();
        app.filter_mode = ProfileFilter::Only("node");
        let visible = app.visible_indices();
        assert_eq!(visible.len(), 2);
        for &idx in &visible {
            assert_eq!(app.entries[idx].profile_name, "node");
        }
    }

    #[test]
    fn test_visible_indices_rust_filter() {
        let mut app = app_with_mixed_entries();
        app.filter_mode = ProfileFilter::Only("rust");
        let visible = app.visible_indices();
        assert_eq!(visible.len(), 1);
        assert_eq!(app.entries[visible[0]].profile_name, "rust");
    }

    #[test]
    fn test_cycle_sort() {
        let mut app = app_with_mixed_entries();
        assert_eq!(app.sort_mode, SortMode::Size);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Date);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Name);

        app.cycle_sort();
        assert_eq!(app.sort_mode, SortMode::Size);
    }

    #[test]
    fn test_cycle_filter() {
        let mut app = app_with_mixed_entries();
        assert_eq!(app.filter_mode, ProfileFilter::All);

        app.cycle_filter();
        assert_eq!(app.filter_mode, ProfileFilter::Only("node"));

        app.cycle_filter();
        assert_eq!(app.filter_mode, ProfileFilter::Only("rust"));

        app.cycle_filter();
        assert_eq!(app.filter_mode, ProfileFilter::All);
    }

    #[test]
    fn test_sort_entries_by_size() {
        let app = app_with_mixed_entries();
        // set_entries calls sort_entries with default SortMode::Size (descending)
        let sizes: Vec<u64> = app.entries.iter().map(|e| e.size).collect();
        for window in sizes.windows(2) {
            assert!(
                window[0] >= window[1],
                "Entries not sorted by size descending"
            );
        }
    }

    #[test]
    fn test_select_all_with_filter() {
        let mut app = app_with_mixed_entries();
        app.filter_mode = ProfileFilter::Only("node");

        app.select_all();

        let visible = app.visible_indices();
        assert_eq!(app.selection.len(), visible.len());

        // Only node entries should be selected
        for &idx in app.selection.iter() {
            assert_eq!(app.entries[idx].profile_name, "node");
        }

        // Rust entries must not be selected
        for (idx, entry) in app.entries.iter().enumerate() {
            if entry.profile_name == "rust" {
                assert!(!app.selection.contains(idx));
            }
        }
    }
}
