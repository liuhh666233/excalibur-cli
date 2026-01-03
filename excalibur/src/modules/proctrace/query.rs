use super::collector::{
    read_environment, read_process, read_working_directory, ProcessCollector, ProcessInfo,
    Supervisor,
};
use super::network::{find_process_by_port, get_process_bindings, NetworkBinding};
use super::systemd::{fetch_systemd_metadata, SystemdMetadata};
use color_eyre::Result;
use std::collections::HashMap;

/// Query type for finding processes
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    ByName(String),  // Process name substring match
    ByPid(u32),      // Exact PID
    ByPort(u16),     // Listening port
}

/// Query result with full context
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub process: ProcessInfo,
    pub ancestor_chain: Vec<ProcessInfo>,         // PID → PPID → ... → init
    pub working_directory: Option<String>,        // /proc/[pid]/cwd
    pub environment: HashMap<String, String>,     // /proc/[pid]/environ
    pub network_bindings: Vec<NetworkBinding>,    // Network connections
    pub systemd_metadata: Option<SystemdMetadata>, // Systemd unit info
}

/// Query engine for process analysis
#[derive(Debug)]
pub struct QueryEngine {
    collector: ProcessCollector,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new() -> Self {
        Self {
            collector: ProcessCollector::new(),
        }
    }

    /// Execute a query and return full context
    pub fn execute(&mut self, query: QueryType) -> Result<Vec<QueryResult>> {
        match query {
            QueryType::ByName(name) => self.query_by_name(&name),
            QueryType::ByPid(pid) => self.query_by_pid(pid),
            QueryType::ByPort(port) => self.query_by_port(port),
        }
    }

    /// Query processes by name (substring match)
    fn query_by_name(&mut self, name: &str) -> Result<Vec<QueryResult>> {
        let all_processes = self.collector.collect()?;
        let name_lower = name.to_lowercase();

        let mut results = Vec::new();
        for process in all_processes {
            if process.name.to_lowercase().contains(&name_lower) {
                let result = self.build_query_result(process)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Query process by exact PID
    fn query_by_pid(&mut self, pid: u32) -> Result<Vec<QueryResult>> {
        // Read single process
        let process = read_process(pid)?;
        let result = self.build_query_result(process)?;
        Ok(vec![result])
    }

    /// Query process by listening port
    fn query_by_port(&mut self, port: u16) -> Result<Vec<QueryResult>> {
        // Find process listening on this port
        match find_process_by_port(port)? {
            Some(pid) => {
                let process = read_process(pid)?;
                let result = self.build_query_result(process)?;
                Ok(vec![result])
            }
            None => {
                // Port query failed - likely a permission issue
                // Return error with helpful message
                Err(color_eyre::eyre::eyre!(
                    "No process found for port {}. \
                    Note: Querying ports used by root processes requires sudo/root privileges.",
                    port
                ))
            }
        }
    }

    /// Build complete QueryResult from ProcessInfo
    fn build_query_result(&mut self, process: ProcessInfo) -> Result<QueryResult> {
        let pid = process.pid;

        // Build ancestor chain
        let ancestor_chain = self.build_ancestor_chain(pid)?;

        // Read working directory
        let working_directory = read_working_directory(pid).ok();

        // Read environment variables
        let environment = read_environment(pid).unwrap_or_default();

        // Get network bindings
        let network_bindings = get_process_bindings(pid).unwrap_or_default();

        // Systemd metadata (if supervisor is systemd)
        let systemd_metadata = match &process.supervisor {
            Supervisor::Systemd { unit } => fetch_systemd_metadata(unit).ok(),
            _ => None,
        };

        Ok(QueryResult {
            process,
            ancestor_chain,
            working_directory,
            environment,
            network_bindings,
            systemd_metadata,
        })
    }

    /// Build ancestor chain by recursively following PPID
    fn build_ancestor_chain(&mut self, pid: u32) -> Result<Vec<ProcessInfo>> {
        let mut chain = Vec::new();
        let mut current_pid = pid;

        // Traverse until init (PID 1)
        while current_pid != 1 {
            match read_process(current_pid) {
                Ok(process) => {
                    current_pid = process.ppid;
                    chain.push(process);
                }
                Err(_) => {
                    // Process might have terminated or we don't have permission
                    break;
                }
            }

            // Safety: prevent infinite loops
            if chain.len() > 100 {
                break;
            }
        }

        // Add init if we reached it
        if current_pid == 1 {
            if let Ok(init) = read_process(1) {
                chain.push(init);
            }
        }

        // Reverse so init is first, queried process is last
        chain.reverse();

        Ok(chain)
    }
}
