//! Host capabilities and VM version information.
//!
//! This module provides utilities for querying host-level capabilities
//! and VM version compatibility.

use crate::error::Result;
use crate::wmi::{WbemClassObjectExt, WmiConnection};

/// VM configuration version information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmVersionInfo {
    /// Major version number.
    pub major: u32,
    /// Minor version number.
    pub minor: u32,
    /// Full version string (e.g., "9.0").
    pub version_string: String,
}

impl VmVersionInfo {
    /// Parse a version string (e.g., "9.0" or "254.0").
    pub fn parse(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            Some(Self {
                major,
                minor,
                version_string: version.to_string(),
            })
        } else {
            None
        }
    }

    /// Check if this version is at least the specified version.
    pub fn is_at_least(&self, major: u32, minor: u32) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }

    /// Check if this version is compatible with required version.
    pub fn is_compatible_with(&self, required: &VmVersionInfo) -> bool {
        self.is_at_least(required.major, required.minor)
    }

    /// Get known version for Windows Server 2016.
    pub fn windows_server_2016() -> Self {
        Self {
            major: 8,
            minor: 0,
            version_string: "8.0".to_string(),
        }
    }

    /// Get known version for Windows Server 2019.
    pub fn windows_server_2019() -> Self {
        Self {
            major: 9,
            minor: 0,
            version_string: "9.0".to_string(),
        }
    }

    /// Get known version for Windows Server 2022.
    pub fn windows_server_2022() -> Self {
        Self {
            major: 10,
            minor: 0,
            version_string: "10.0".to_string(),
        }
    }
}

impl std::fmt::Display for VmVersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version_string)
    }
}

/// Host-level Hyper-V capabilities.
#[derive(Debug, Clone)]
pub struct HostCapabilities {
    /// Default VM configuration version for new VMs.
    pub default_vm_version: Option<VmVersionInfo>,
    /// Whether live migration is supported.
    pub live_migration_supported: bool,
    /// Whether NUMA spanning is enabled.
    pub numa_spanning_enabled: bool,
    /// Maximum number of virtual processors per VM.
    pub max_processors_per_vm: u32,
    /// Maximum memory per VM in MB.
    pub max_memory_per_vm_mb: u64,
    /// Whether nested virtualization is supported.
    pub nested_virtualization_supported: bool,
    /// Whether GPU-P is supported.
    pub gpu_p_supported: bool,
    /// Whether TPM is supported.
    pub tpm_supported: bool,
    /// Whether SGX is supported.
    pub sgx_supported: bool,
    /// Host OS version string.
    pub host_os_version: String,
}

impl Default for HostCapabilities {
    fn default() -> Self {
        Self {
            default_vm_version: None,
            live_migration_supported: false,
            numa_spanning_enabled: true,
            max_processors_per_vm: 240,
            max_memory_per_vm_mb: 12 * 1024 * 1024, // 12 TB
            nested_virtualization_supported: false,
            gpu_p_supported: false,
            tpm_supported: false,
            sgx_supported: false,
            host_os_version: String::new(),
        }
    }
}

impl HostCapabilities {
    /// Query host capabilities from WMI.
    pub fn query(connection: &WmiConnection) -> Result<Self> {
        let mut caps = Self::default();

        // Query Msvm_VirtualSystemManagementServiceSettingData for default VM version
        if let Ok(Some(settings)) =
            connection.query_first("SELECT * FROM Msvm_VirtualSystemManagementServiceSettingData")
        {
            // Note: DefaultVirtualHardDiskPath is the path, not version
            // This demonstrates the query pattern for future capability queries
            let _path = settings.get_string_prop("DefaultVirtualHardDiskPath")?;
        }

        // Query for NUMA spanning setting
        if let Ok(Some(host_settings)) =
            connection.query_first("SELECT * FROM Msvm_VirtualSystemManagementServiceSettingData")
        {
            if let Ok(Some(numa)) = host_settings.get_bool("NumaSpanningEnabled") {
                caps.numa_spanning_enabled = numa;
            }
        }

        // Check for live migration support via Msvm_VirtualSystemMigrationService
        if let Ok(results) = connection.query("SELECT * FROM Msvm_VirtualSystemMigrationService") {
            caps.live_migration_supported = !results.is_empty();
        }

        // Query processor capabilities
        if let Ok(Some(proc_caps)) =
            connection.query_first("SELECT * FROM Msvm_ProcessorPool WHERE Primordial = TRUE")
        {
            if let Ok(Some(max_proc)) = proc_caps.get_u32("MaxProcessorsPerVm") {
                caps.max_processors_per_vm = max_proc;
            }
        }

        // Check for security service (TPM support)
        if let Ok(results) = connection.query("SELECT * FROM Msvm_SecurityService") {
            caps.tpm_supported = !results.is_empty();
        }

        // Check for GPU-P support
        if let Ok(results) = connection.query("SELECT * FROM Msvm_PartitionableGpu") {
            caps.gpu_p_supported = !results.is_empty();
        }

        Ok(caps)
    }

    /// Check if a VM version is supported on this host.
    pub fn is_vm_version_supported(&self, version: &VmVersionInfo) -> bool {
        if let Some(ref default) = self.default_vm_version {
            // Generally, the host supports its default version and lower
            version.major <= default.major
        } else {
            true // Assume supported if we can't determine
        }
    }

    /// Get the recommended VM version for this host.
    pub fn recommended_vm_version(&self) -> Option<&VmVersionInfo> {
        self.default_vm_version.as_ref()
    }
}

/// Query the default VM configuration version for the host.
pub fn get_default_vm_version(connection: &WmiConnection) -> Result<Option<VmVersionInfo>> {
    // Query Msvm_VirtualSystemManagementCapabilities for supported versions
    let query = "SELECT * FROM Msvm_VirtualSystemManagementCapabilities";

    if let Some(caps) = connection.query_first(query)? {
        // Get SupportedVirtualSystemTypes which contains version info
        if let Ok(Some(types)) = caps.get_string_array("SupportedVirtualSystemTypes") {
            // Find the highest version
            let mut highest: Option<VmVersionInfo> = None;
            for type_str in types {
                // Format is like "Microsoft:Hyper-V:SubType:2"
                if let Some(version) = VmVersionInfo::parse(&type_str) {
                    if highest
                        .as_ref()
                        .map(|h| version.major > h.major)
                        .unwrap_or(true)
                    {
                        highest = Some(version);
                    }
                }
            }
            return Ok(highest);
        }
    }

    Ok(None)
}

/// Check if a specific VM version is supported on the host.
pub fn is_vm_version_supported(connection: &WmiConnection, version: &str) -> Result<bool> {
    let query = format!(
        "SELECT * FROM Msvm_VirtualSystemManagementCapabilities WHERE SupportedVirtualSystemTypes LIKE '%{}%'",
        version.replace('\'', "''")
    );

    let results = connection.query(&query)?;
    Ok(!results.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_version_parse() {
        let v = VmVersionInfo::parse("9.0").unwrap();
        assert_eq!(v.major, 9);
        assert_eq!(v.minor, 0);

        let v = VmVersionInfo::parse("254.0").unwrap();
        assert_eq!(v.major, 254);
        assert_eq!(v.minor, 0);

        assert!(VmVersionInfo::parse("invalid").is_none());
    }

    #[test]
    fn test_vm_version_parse_with_minor() {
        let v = VmVersionInfo::parse("9.1").unwrap();
        assert_eq!(v.major, 9);
        assert_eq!(v.minor, 1);
        assert_eq!(v.version_string, "9.1");
    }

    #[test]
    fn test_vm_version_parse_invalid() {
        assert!(VmVersionInfo::parse("").is_none());
        assert!(VmVersionInfo::parse("9").is_none());
        assert!(VmVersionInfo::parse("abc.def").is_none());
        assert!(VmVersionInfo::parse("9.").is_none());
        assert!(VmVersionInfo::parse(".0").is_none());
    }

    #[test]
    fn test_vm_version_comparison() {
        let v9 = VmVersionInfo::parse("9.0").unwrap();
        let v10 = VmVersionInfo::parse("10.0").unwrap();

        assert!(v10.is_at_least(9, 0));
        assert!(v10.is_at_least(10, 0));
        assert!(!v9.is_at_least(10, 0));

        assert!(v10.is_compatible_with(&v9));
        assert!(!v9.is_compatible_with(&v10));
    }

    #[test]
    fn test_vm_version_comparison_minor() {
        let v9_0 = VmVersionInfo::parse("9.0").unwrap();
        let v9_1 = VmVersionInfo::parse("9.1").unwrap();

        assert!(v9_1.is_at_least(9, 0));
        assert!(v9_1.is_at_least(9, 1));
        assert!(!v9_0.is_at_least(9, 1));
        assert!(v9_0.is_at_least(9, 0));
    }

    #[test]
    fn test_vm_version_equality() {
        let v1 = VmVersionInfo::parse("9.0").unwrap();
        let v2 = VmVersionInfo::parse("9.0").unwrap();
        let v3 = VmVersionInfo::parse("10.0").unwrap();

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_vm_version_display() {
        let v = VmVersionInfo::parse("9.0").unwrap();
        assert_eq!(format!("{}", v), "9.0");

        let v = VmVersionInfo::windows_server_2022();
        assert_eq!(format!("{}", v), "10.0");
    }

    #[test]
    fn test_known_versions() {
        let ws2016 = VmVersionInfo::windows_server_2016();
        let ws2019 = VmVersionInfo::windows_server_2019();
        let ws2022 = VmVersionInfo::windows_server_2022();

        assert!(ws2019.is_at_least(ws2016.major, ws2016.minor));
        assert!(ws2022.is_at_least(ws2019.major, ws2019.minor));

        // Verify version ordering
        assert_eq!(ws2016.major, 8);
        assert_eq!(ws2019.major, 9);
        assert_eq!(ws2022.major, 10);
    }

    #[test]
    fn test_known_versions_compatibility() {
        let ws2016 = VmVersionInfo::windows_server_2016();
        let ws2019 = VmVersionInfo::windows_server_2019();
        let ws2022 = VmVersionInfo::windows_server_2022();

        // Newer hosts can run older VM versions
        assert!(ws2022.is_compatible_with(&ws2019));
        assert!(ws2022.is_compatible_with(&ws2016));
        assert!(ws2019.is_compatible_with(&ws2016));

        // Older hosts cannot run newer VM versions
        assert!(!ws2016.is_compatible_with(&ws2019));
        assert!(!ws2019.is_compatible_with(&ws2022));
    }

    #[test]
    fn test_host_capabilities_default() {
        let caps = HostCapabilities::default();

        assert!(caps.default_vm_version.is_none());
        assert!(!caps.live_migration_supported);
        assert!(caps.numa_spanning_enabled);
        assert_eq!(caps.max_processors_per_vm, 240);
        assert!(!caps.nested_virtualization_supported);
        assert!(!caps.gpu_p_supported);
        assert!(!caps.tpm_supported);
        assert!(!caps.sgx_supported);
    }

    #[test]
    fn test_host_capabilities_vm_version_check() {
        let mut caps = HostCapabilities::default();
        caps.default_vm_version = Some(VmVersionInfo::windows_server_2022());

        let v9 = VmVersionInfo::parse("9.0").unwrap();
        let v10 = VmVersionInfo::parse("10.0").unwrap();
        let v11 = VmVersionInfo::parse("11.0").unwrap();

        assert!(caps.is_vm_version_supported(&v9));
        assert!(caps.is_vm_version_supported(&v10));
        assert!(!caps.is_vm_version_supported(&v11));
    }

    #[test]
    fn test_host_capabilities_recommended_version() {
        let mut caps = HostCapabilities::default();
        assert!(caps.recommended_vm_version().is_none());

        caps.default_vm_version = Some(VmVersionInfo::windows_server_2022());
        let recommended = caps.recommended_vm_version().unwrap();
        assert_eq!(recommended.major, 10);
    }
}
