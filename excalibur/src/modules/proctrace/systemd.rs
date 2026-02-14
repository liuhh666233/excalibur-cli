use color_eyre::Result;
use std::collections::HashMap;
use std::process::Command;

/// Rich systemd metadata from `systemctl show`
#[derive(Debug, Clone)]
pub struct SystemdMetadata {
    pub unit_name: String,
    pub description: Option<String>,
    pub load_state: String,   // loaded, not-found, masked
    pub active_state: String, // active, inactive, failed
    pub sub_state: String,    // running, dead, exited
    pub main_pid: Option<u32>,
    pub exec_start: Option<String>,
    pub restart_policy: Option<String>,
    pub wanted_by: Vec<String>,
}

/// Fetch rich metadata from systemctl show
pub fn fetch_systemd_metadata(unit_name: &str) -> Result<SystemdMetadata> {
    let output = Command::new("systemctl")
        .args(["show", unit_name])
        .output()?;

    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "systemctl show failed for {}",
            unit_name
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let properties = parse_systemctl_output(&stdout);

    Ok(SystemdMetadata {
        unit_name: unit_name.to_string(),
        description: properties.get("Description").cloned(),
        load_state: properties
            .get("LoadState")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        active_state: properties
            .get("ActiveState")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        sub_state: properties
            .get("SubState")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        main_pid: properties
            .get("MainPID")
            .and_then(|s| s.parse::<u32>().ok()),
        exec_start: properties.get("ExecStart").cloned(),
        restart_policy: properties.get("Restart").cloned(),
        wanted_by: properties
            .get("WantedBy")
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default(),
    })
}

/// Parse systemctl show output (KEY=VALUE format)
fn parse_systemctl_output(output: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();

    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            props.insert(key.to_string(), value.to_string());
        }
    }

    props
}
