use super::collector::ProcessInfo;
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::time::Instant;

/// Input mode for the process tracer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}

/// Sort mode for process list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Cpu,
    Memory,
    Pid,
    Name,
}

impl SortMode {
    /// Get the next sort mode in cycle
    pub fn next(&self) -> Self {
        match self {
            SortMode::Cpu => SortMode::Memory,
            SortMode::Memory => SortMode::Pid,
            SortMode::Pid => SortMode::Name,
            SortMode::Name => SortMode::Cpu,
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &str {
        match self {
            SortMode::Cpu => "CPU",
            SortMode::Memory => "Memory",
            SortMode::Pid => "PID",
            SortMode::Name => "Name",
        }
    }
}

/// View mode for process display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,  // Flat list view
    Tree,  // Hierarchical tree view
}

/// Process tree node
#[derive(Debug, Clone)]
pub struct ProcessTreeNode {
    pub process_idx: usize,    // Index into processes Vec
    pub children: Vec<u32>,    // PIDs of child processes
    pub is_expanded: bool,     // Expansion state
    pub depth: usize,          // Tree depth for indentation
}

/// State for the process tracer module
#[derive(Debug)]
pub struct ProcessTracerState {
    /// All processes
    pub processes: Vec<ProcessInfo>,

    /// Indices of processes after filtering
    pub filtered_indices: Vec<usize>,

    /// Currently selected index in filtered list
    pub selected_index: usize,

    /// Ratatui table state
    pub table_state: TableState,

    /// Search query string
    pub search_query: String,

    /// Current input mode
    pub input_mode: InputMode,

    /// Current sort mode
    pub sort_mode: SortMode,

    /// Last update timestamp
    pub last_update: Instant,

    /// Notification message with timestamp
    pub notification: Option<(String, Instant)>,

    /// Current view mode
    pub view_mode: ViewMode,

    /// Process tree (PID -> TreeNode)
    pub tree_nodes: HashMap<u32, ProcessTreeNode>,

    /// Tree root PIDs (processes with no parent or parent not in list)
    pub tree_roots: Vec<u32>,

    /// Flattened visible tree nodes (for navigation)
    pub visible_tree_nodes: Vec<u32>,
}

impl ProcessTracerState {
    pub fn new() -> Self {
        let mut state = Self {
            processes: Vec::new(),
            filtered_indices: Vec::new(),
            selected_index: 0,
            table_state: TableState::default(),
            search_query: String::new(),
            input_mode: InputMode::Normal,
            sort_mode: SortMode::Cpu,
            last_update: Instant::now(),
            notification: None,
            view_mode: ViewMode::List,
            tree_nodes: HashMap::new(),
            tree_roots: Vec::new(),
            visible_tree_nodes: Vec::new(),
        };

        state.table_state.select(Some(0));
        state
    }

    /// Update process list
    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.processes = processes;
        self.apply_filters();
        self.apply_sort();

        // Keep selection valid
        if self.selected_index >= self.filtered_indices.len() && !self.filtered_indices.is_empty()
        {
            self.selected_index = self.filtered_indices.len() - 1;
            self.table_state.select(Some(self.selected_index));
        }

        // Rebuild tree if in tree view
        if self.view_mode == ViewMode::Tree {
            self.build_tree();
        }
    }

    /// Apply search filter
    pub fn apply_filters(&mut self) {
        if self.search_query.is_empty() {
            // No filter, show all
            self.filtered_indices = (0..self.processes.len()).collect();
        } else {
            // Filter by process name
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = self
                .processes
                .iter()
                .enumerate()
                .filter(|(_, proc)| proc.name.to_lowercase().contains(&query_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        // Reset selection
        self.selected_index = 0;
        self.table_state.select(Some(0));
    }

    /// Apply current sort mode
    pub fn apply_sort(&mut self) {
        let processes = &self.processes;

        self.filtered_indices.sort_by(|&a, &b| {
            let proc_a = &processes[a];
            let proc_b = &processes[b];

            match self.sort_mode {
                SortMode::Cpu => proc_b
                    .cpu_percent
                    .partial_cmp(&proc_a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortMode::Memory => proc_b.memory_rss.cmp(&proc_a.memory_rss),
                SortMode::Pid => proc_a.pid.cmp(&proc_b.pid),
                SortMode::Name => proc_a.name.cmp(&proc_b.name),
            }
        });
    }

    /// Cycle to next sort mode
    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.apply_sort();
        self.set_notification(format!("Sort by: {}", self.sort_mode.as_str()));
    }

    /// Select next process
    pub fn select_next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.filtered_indices.len();
        self.table_state.select(Some(self.selected_index));
    }

    /// Select previous process
    pub fn select_previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        if self.selected_index == 0 {
            self.selected_index = self.filtered_indices.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        self.table_state.select(Some(self.selected_index));
    }

    /// Jump to first process
    pub fn select_first(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = 0;
            self.table_state.select(Some(0));
        }
    }

    /// Jump to last process
    pub fn select_last(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = self.filtered_indices.len() - 1;
            self.table_state.select(Some(self.selected_index));
        }
    }

    /// Page down (move by 10)
    pub fn page_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        self.selected_index = (self.selected_index + 10).min(self.filtered_indices.len() - 1);
        self.table_state.select(Some(self.selected_index));
    }

    /// Page up (move by 10)
    pub fn page_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        self.selected_index = self.selected_index.saturating_sub(10);
        self.table_state.select(Some(self.selected_index));
    }

    /// Get currently selected process
    pub fn get_selected_process(&self) -> Option<&ProcessInfo> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.processes.get(idx))
    }

    /// Set notification message
    pub fn set_notification(&mut self, message: String) {
        self.notification = Some((message, Instant::now()));
    }

    /// Clear expired notifications (older than 3 seconds)
    pub fn clear_expired_notifications(&mut self) {
        if let Some((_, timestamp)) = self.notification {
            if timestamp.elapsed().as_secs() >= 3 {
                self.notification = None;
            }
        }
    }

    /// Get process count strings
    pub fn get_counts(&self) -> (usize, usize) {
        (self.filtered_indices.len(), self.processes.len())
    }

    /// Build process tree from flat process list
    pub fn build_tree(&mut self) {
        self.tree_nodes.clear();
        self.tree_roots.clear();

        // Create a PID -> process_idx map for quick lookup
        let mut pid_to_idx: HashMap<u32, usize> = HashMap::new();
        for (idx, proc) in self.processes.iter().enumerate() {
            pid_to_idx.insert(proc.pid, idx);
        }

        // Build parent-child relationships
        for (idx, proc) in self.processes.iter().enumerate() {
            let node = ProcessTreeNode {
                process_idx: idx,
                children: Vec::new(),
                is_expanded: false,
                depth: 0,
            };
            self.tree_nodes.insert(proc.pid, node);
        }

        // Link children to parents
        for proc in &self.processes {
            if let Some(parent_idx) = pid_to_idx.get(&proc.ppid) {
                let parent_pid = self.processes[*parent_idx].pid;
                if let Some(parent_node) = self.tree_nodes.get_mut(&parent_pid) {
                    parent_node.children.push(proc.pid);
                }
            } else {
                // No parent in list -> root
                self.tree_roots.push(proc.pid);
            }
        }

        // Calculate depths
        for root_pid in &self.tree_roots.clone() {
            self.calculate_depth(*root_pid, 0);
        }

        // Build initial visible nodes (only roots)
        self.rebuild_visible_nodes();
    }

    /// Recursively calculate tree depth
    fn calculate_depth(&mut self, pid: u32, depth: usize) {
        if let Some(node) = self.tree_nodes.get_mut(&pid) {
            node.depth = depth;
            let children = node.children.clone();
            for child_pid in children {
                self.calculate_depth(child_pid, depth + 1);
            }
        }
    }

    /// Rebuild visible nodes based on expansion state
    pub fn rebuild_visible_nodes(&mut self) {
        self.visible_tree_nodes.clear();
        let roots = self.tree_roots.clone();
        for root_pid in &roots {
            self.add_visible_subtree(*root_pid);
        }
    }

    /// Recursively add visible nodes
    fn add_visible_subtree(&mut self, pid: u32) {
        self.visible_tree_nodes.push(pid);

        if let Some(node) = self.tree_nodes.get(&pid) {
            if node.is_expanded {
                let children = node.children.clone();
                for child_pid in children {
                    self.add_visible_subtree(child_pid);
                }
            }
        }
    }

    /// Toggle between List and Tree view
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::List => {
                // Build tree when entering tree view
                self.build_tree();
                ViewMode::Tree
            }
            ViewMode::Tree => ViewMode::List,
        };

        self.set_notification(format!("View: {:?}", self.view_mode));
    }

    /// Toggle expansion of current node in tree view
    pub fn toggle_tree_expansion(&mut self) {
        if self.view_mode != ViewMode::Tree {
            return;
        }

        // Get currently selected PID from visible nodes
        if let Some(&pid) = self.visible_tree_nodes.get(self.selected_index) {
            if let Some(node) = self.tree_nodes.get_mut(&pid) {
                // Only toggle if has children
                if !node.children.is_empty() {
                    node.is_expanded = !node.is_expanded;
                    self.rebuild_visible_nodes();

                    // Keep selection valid
                    if self.selected_index >= self.visible_tree_nodes.len() {
                        self.selected_index = self.visible_tree_nodes.len().saturating_sub(1);
                    }
                    self.table_state.select(Some(self.selected_index));
                }
            }
        }
    }

    /// Get selected process in tree view
    pub fn get_selected_process_tree(&self) -> Option<&ProcessInfo> {
        if self.view_mode == ViewMode::Tree {
            self.visible_tree_nodes
                .get(self.selected_index)
                .and_then(|&pid| {
                    self.tree_nodes.get(&pid)
                        .and_then(|node| self.processes.get(node.process_idx))
                })
        } else {
            self.get_selected_process()
        }
    }
}
