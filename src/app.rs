use crate::scanner::CleanEntry;
use ratatui::widgets::ListState;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Welcome,
    List,
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

    pub fn select_all(&mut self, entries: &[CleanEntry]) {
        self.indices.clear();
        self.total_size = 0;
        for (i, entry) in entries.iter().enumerate() {
            self.indices.insert(i);
            self.total_size += entry.size;
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
        }
    }

    pub fn set_entries(&mut self, entries: Vec<CleanEntry>) {
        self.total_size = entries.iter().map(|e| e.size).sum();
        self.entries = entries;
        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
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
        if self.entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_select(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.selection.toggle(i, self.entries[i].size);
        }
    }

    pub fn select_all(&mut self) {
        self.selection.select_all(&self.entries);
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

        if self.entries.is_empty() {
            self.list_state.select(None);
        } else if let Some(current) = self.list_state.selected() {
            if current >= self.entries.len() {
                self.list_state.select(Some(self.entries.len() - 1));
            }
        }
    }
}
