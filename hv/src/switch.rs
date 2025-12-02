//! Virtual Switch management using HNS (Host Network Service)
//!
//! Provides virtual switch management for Hyper-V networking.

use crate::error::{HvError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Virtual switch type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchType {
    /// External - connected to physical network adapter
    External,
    /// Internal - connects VMs to host
    Internal,
    /// Private - connects VMs to each other only
    Private,
}

impl From<u16> for SwitchType {
    fn from(value: u16) -> Self {
        match value {
            0 => SwitchType::Private,
            1 => SwitchType::Internal,
            2 => SwitchType::External,
            _ => SwitchType::Private,
        }
    }
}

impl SwitchType {
    /// Returns string representation for HNS
    pub fn as_str(&self) -> &str {
        match self {
            SwitchType::Private => "Private",
            SwitchType::Internal => "Internal",
            SwitchType::External => "External",
        }
    }
}

/// HNS Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HnsNetworkInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "Type", default)]
    pub network_type: Option<String>,
    #[serde(default)]
    pub policies: Option<serde_json::Value>,
    #[serde(default)]
    pub network_adapter_name: Option<String>,
    #[serde(default)]
    pub subnets: Option<Vec<HnsSubnet>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HnsSubnet {
    #[serde(default)]
    pub address_prefix: Option<String>,
    #[serde(default)]
    pub gateway_address: Option<String>,
}

/// Represents a Hyper-V virtual switch
pub struct VirtualSwitch {
    name: String,
    switch_id: String,
    switch_type: Option<SwitchType>,
}

impl VirtualSwitch {
    /// Create from enumeration info
    pub(crate) fn from_info(id: String, name: String, switch_type: Option<SwitchType>) -> Self {
        VirtualSwitch {
            name,
            switch_id: id,
            switch_type,
        }
    }

    /// Returns the switch name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the switch ID (GUID)
    pub fn id(&self) -> &str {
        &self.switch_id
    }

    /// Gets the switch type
    pub fn switch_type(&self) -> Result<SwitchType> {
        if let Some(st) = self.switch_type {
            return Ok(st);
        }

        // Query via PowerShell if not cached
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-VMSwitch -Id '{}').SwitchType", self.switch_id),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(format!("Failed to query switch type: {}", e)))?;

        if !output.status.success() {
            return Ok(SwitchType::Private);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        match stdout.trim().to_lowercase().as_str() {
            "external" => Ok(SwitchType::External),
            "internal" => Ok(SwitchType::Internal),
            _ => Ok(SwitchType::Private),
        }
    }

    /// Gets the list of VMs connected to this switch
    pub fn connected_vms(&self) -> Result<Vec<String>> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Get-VMNetworkAdapter -All | Where-Object {{ $_.SwitchId -eq '{}' }} | Select-Object -ExpandProperty VMName | Sort-Object -Unique",
                    self.switch_id
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(format!("Failed to query connected VMs: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Gets the physical network adapter name (for external switches)
    pub fn network_adapter(&self) -> Result<Option<String>> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-VMSwitch -Id '{}').NetAdapterInterfaceDescription",
                    self.switch_id
                ),
            ])
            .output()
            .map_err(|e| {
                HvError::OperationFailed(format!("Failed to query network adapter: {}", e))
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let adapter = stdout.trim();

        if adapter.is_empty() {
            Ok(None)
        } else {
            Ok(Some(adapter.to_string()))
        }
    }

    /// Checks if the switch allows management OS access
    pub fn allows_management_os(&self) -> Result<bool> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-VMSwitch -Id '{}').AllowManagementOS", self.switch_id),
            ])
            .output()
            .map_err(|e| {
                HvError::OperationFailed(format!("Failed to query management OS: {}", e))
            })?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_lowercase() == "true")
    }

    /// Deletes this virtual switch
    pub fn delete(&self) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Remove-VMSwitch -Id '{}' -Force", self.switch_id),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(format!("Failed to delete switch: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(format!(
                "Failed to delete switch: {}",
                stderr
            )));
        }

        Ok(())
    }
}

impl std::fmt::Debug for VirtualSwitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualSwitch")
            .field("name", &self.name)
            .field("id", &self.switch_id)
            .field("switch_type", &self.switch_type)
            .finish()
    }
}

/// Information about a virtual switch from enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SwitchInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub switch_type: Option<String>,
    #[serde(default)]
    pub allow_management_o_s: Option<bool>,
    #[serde(default)]
    pub net_adapter_interface_description: Option<String>,
}

/// Enumerate all virtual switches
pub fn enumerate_switches() -> Result<Vec<VirtualSwitch>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-VMSwitch | Select-Object Id, Name, SwitchType | ConvertTo-Json -Compress",
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to enumerate switches: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to enumerate switches: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    // Handle both single object and array
    let switches: Vec<SwitchInfo> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed)
            .map_err(|e| HvError::JsonError(format!("Failed to parse switch list: {}", e)))?
    } else {
        let single: SwitchInfo = serde_json::from_str(trimmed)
            .map_err(|e| HvError::JsonError(format!("Failed to parse switch: {}", e)))?;
        vec![single]
    };

    Ok(switches
        .into_iter()
        .filter_map(|info| {
            let id = info.id?;
            let name = info.name.unwrap_or_else(|| id.clone());
            let switch_type = info
                .switch_type
                .as_ref()
                .map(|s| match s.to_lowercase().as_str() {
                    "external" => SwitchType::External,
                    "internal" => SwitchType::Internal,
                    _ => SwitchType::Private,
                });
            Some(VirtualSwitch::from_info(id, name, switch_type))
        })
        .collect())
}

/// Get a switch by name
pub fn get_switch(name: &str) -> Result<VirtualSwitch> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Get-VMSwitch -Name '{}' | Select-Object Id, Name, SwitchType | ConvertTo-Json -Compress",
                name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get switch: {}", e)))?;

    if !output.status.success() {
        return Err(HvError::SwitchNotFound(name.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Err(HvError::SwitchNotFound(name.to_string()));
    }

    let info: SwitchInfo = serde_json::from_str(trimmed)
        .map_err(|e| HvError::JsonError(format!("Failed to parse switch: {}", e)))?;

    let id = info
        .id
        .ok_or_else(|| HvError::SwitchNotFound(name.to_string()))?;
    let switch_name = info.name.unwrap_or_else(|| name.to_string());
    let switch_type = info
        .switch_type
        .as_ref()
        .map(|s| match s.to_lowercase().as_str() {
            "external" => SwitchType::External,
            "internal" => SwitchType::Internal,
            _ => SwitchType::Private,
        });

    Ok(VirtualSwitch::from_info(id, switch_name, switch_type))
}

/// Create a new virtual switch
pub fn create_switch(name: &str, switch_type: SwitchType) -> Result<VirtualSwitch> {
    if switch_type == SwitchType::External {
        return Err(HvError::InvalidParameter(
            "External switch requires a network adapter name. Use create_external_switch()."
                .to_string(),
        ));
    }

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "New-VMSwitch -Name '{}' -SwitchType {} | Select-Object Id | ConvertTo-Json -Compress",
                name,
                switch_type.as_str()
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to create switch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to create switch: {}",
            stderr
        )));
    }

    get_switch(name)
}

/// Create an external virtual switch
pub fn create_external_switch(
    name: &str,
    network_adapter_name: &str,
    allow_management_os: bool,
) -> Result<VirtualSwitch> {
    let mgmt_os = if allow_management_os {
        "$true"
    } else {
        "$false"
    };

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "New-VMSwitch -Name '{}' -NetAdapterName '{}' -AllowManagementOS {} | Select-Object Id | ConvertTo-Json -Compress",
                name, network_adapter_name, mgmt_os
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to create external switch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to create external switch: {}",
            stderr
        )));
    }

    get_switch(name)
}
