//! VM Snapshot (Checkpoint) management
//!
//! Provides snapshot creation, restoration, and management for Hyper-V VMs.

use crate::error::{HvError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Snapshot type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotType {
    /// Standard snapshot with full state
    Standard,
    /// Production snapshot (application-consistent)
    Production,
    /// Production snapshot with fallback to standard
    ProductionFallback,
}

impl SnapshotType {
    /// Returns the PowerShell parameter value
    pub fn to_powershell_value(&self) -> &str {
        match self {
            SnapshotType::Standard => "Standard",
            SnapshotType::Production => "Production",
            SnapshotType::ProductionFallback => "ProductionOnly",
        }
    }
}

/// Represents a Hyper-V VM snapshot (checkpoint)
pub struct Snapshot {
    name: String,
    snapshot_id: String,
    vm_name: String,
}

impl Snapshot {
    /// Create a new snapshot handle
    pub(crate) fn new(name: String, snapshot_id: String, vm_name: String) -> Self {
        Snapshot {
            name,
            snapshot_id,
            vm_name,
        }
    }

    /// Returns the snapshot name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the snapshot ID
    pub fn id(&self) -> &str {
        &self.snapshot_id
    }

    /// Returns the parent VM name
    pub fn vm_name(&self) -> &str {
        &self.vm_name
    }

    /// Gets the creation time of the snapshot
    pub fn creation_time(&self) -> Result<String> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-VMSnapshot -VMName '{}' -Name '{}').CreationTime.ToString('o')",
                    self.vm_name, self.name
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Gets the parent snapshot name (if any)
    pub fn parent_name(&self) -> Result<Option<String>> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-VMSnapshot -VMName '{}' -Name '{}').ParentSnapshotName",
                    self.vm_name, self.name
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parent = stdout.trim();

        if parent.is_empty() {
            Ok(None)
        } else {
            Ok(Some(parent.to_string()))
        }
    }

    /// Applies (restores) this snapshot to the VM
    pub fn apply(&self) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Restore-VMSnapshot -VMName '{}' -Name '{}' -Confirm:$false",
                    self.vm_name, self.name
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Deletes this snapshot
    pub fn delete(&self) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Remove-VMSnapshot -VMName '{}' -Name '{}' -Confirm:$false",
                    self.vm_name, self.name
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Deletes this snapshot and all child snapshots
    pub fn delete_subtree(&self) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Remove-VMSnapshot -VMName '{}' -Name '{}' -IncludeAllChildSnapshots -Confirm:$false",
                    self.vm_name, self.name
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Renames this snapshot
    pub fn rename(&mut self, new_name: &str) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Rename-VMSnapshot -VMName '{}' -Name '{}' -NewName '{}'",
                    self.vm_name, self.name, new_name
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        self.name = new_name.to_string();
        Ok(())
    }

    /// Exports this snapshot to a path
    pub fn export(&self, destination_path: &str) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Export-VMSnapshot -VMName '{}' -Name '{}' -Path '{}'",
                    self.vm_name, self.name, destination_path
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        Ok(())
    }
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("name", &self.name)
            .field("id", &self.snapshot_id)
            .field("vm_name", &self.vm_name)
            .finish()
    }
}

/// Snapshot info from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SnapshotInfo {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    id: Option<String>,
}

/// Lists all snapshots for a VM
pub fn list_snapshots(vm_name: &str) -> Result<Vec<Snapshot>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Get-VMSnapshot -VMName '{}' | Select-Object Name, Id | ConvertTo-Json -Compress",
                vm_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    // Handle both single object and array
    let snapshots: Vec<SnapshotInfo> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed)
            .map_err(|e| HvError::JsonError(format!("Failed to parse snapshot list: {}", e)))?
    } else {
        let single: SnapshotInfo = serde_json::from_str(trimmed)
            .map_err(|e| HvError::JsonError(format!("Failed to parse snapshot: {}", e)))?;
        vec![single]
    };

    Ok(snapshots
        .into_iter()
        .filter_map(|info| {
            let name = info.name?;
            let id = info.id.unwrap_or_else(|| name.clone());
            Some(Snapshot::new(name, id, vm_name.to_string()))
        })
        .collect())
}

/// Gets a specific snapshot
pub fn get_snapshot(vm_name: &str, snapshot_name: &str) -> Result<Snapshot> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Get-VMSnapshot -VMName '{}' -Name '{}' | Select-Object Id | ConvertTo-Json -Compress",
                vm_name, snapshot_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::SnapshotNotFound(format!(
            "{}/{}: {}",
            vm_name, snapshot_name, stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Err(HvError::SnapshotNotFound(format!(
            "{}/{}",
            vm_name, snapshot_name
        )));
    }

    let info: SnapshotInfo = serde_json::from_str(trimmed)
        .map_err(|e| HvError::JsonError(format!("Failed to parse snapshot: {}", e)))?;

    let id = info.id.unwrap_or_else(|| snapshot_name.to_string());

    Ok(Snapshot::new(
        snapshot_name.to_string(),
        id,
        vm_name.to_string(),
    ))
}

/// Creates a new snapshot for a VM
pub fn create_snapshot(
    vm_name: &str,
    snapshot_name: &str,
    snapshot_type: SnapshotType,
) -> Result<Snapshot> {
    // Note: The Checkpoint-VM cmdlet uses -CheckpointType (not -SnapshotType)
    // Available types: Standard, Production, ProductionOnly
    // For Standard checkpoints, we can omit the type parameter for compatibility
    let type_arg = match snapshot_type {
        SnapshotType::Standard => "", // Default is Standard, omit for compatibility
        SnapshotType::Production => "-CheckpointType Production",
        SnapshotType::ProductionFallback => "-CheckpointType ProductionOnly",
    };

    let cmd = if type_arg.is_empty() {
        format!(
            "Checkpoint-VM -Name '{}' -SnapshotName '{}'",
            vm_name, snapshot_name
        )
    } else {
        format!(
            "Checkpoint-VM -Name '{}' -SnapshotName '{}' {}",
            vm_name, snapshot_name, type_arg
        )
    };

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(stderr.to_string()));
    }

    get_snapshot(vm_name, snapshot_name)
}
