//! Property support validation for Hyper-V WMI classes.
//!
//! Hyper-V WMI properties vary by Windows version and VM configuration version.
//! This module provides utilities to check property availability before use.

use crate::error::{Error, Result};
use crate::wmi::WmiConnection;

/// Result of a property support check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertySupport {
    /// Property is supported.
    Supported,
    /// Property is not supported (reason provided).
    NotSupported(String),
    /// Unable to determine support status.
    Unknown,
}

impl PropertySupport {
    /// Check if property is supported.
    pub fn is_supported(&self) -> bool {
        matches!(self, PropertySupport::Supported)
    }

    /// Convert to Result, returning error if not supported.
    pub fn require(self, property_name: &str) -> Result<()> {
        match self {
            PropertySupport::Supported => Ok(()),
            PropertySupport::NotSupported(reason) => Err(Error::PropertyNotSupported {
                property: property_name.to_string(),
                reason,
            }),
            PropertySupport::Unknown => Err(Error::PropertyNotSupported {
                property: property_name.to_string(),
                reason: "Support status unknown".to_string(),
            }),
        }
    }
}

/// Validator for checking property support on WMI classes.
pub struct PropertyValidator<'a> {
    connection: &'a WmiConnection,
    /// Cached class definitions for property checking.
    class_cache: std::collections::HashMap<String, Vec<String>>,
}

impl<'a> PropertyValidator<'a> {
    /// Create a new property validator.
    pub fn new(connection: &'a WmiConnection) -> Self {
        Self {
            connection,
            class_cache: std::collections::HashMap::new(),
        }
    }

    /// Check if a property exists on a WMI class.
    pub fn supports_property(&mut self, class_name: &str, property_name: &str) -> PropertySupport {
        // Check cache first
        if let Some(properties) = self.class_cache.get(class_name) {
            return if properties.iter().any(|p| p == property_name) {
                PropertySupport::Supported
            } else {
                PropertySupport::NotSupported(format!(
                    "Property '{}' not found in class '{}'",
                    property_name, class_name
                ))
            };
        }

        // Query class definition
        match self.load_class_properties(class_name) {
            Ok(properties) => {
                let supported = properties.iter().any(|p| p == property_name);
                self.class_cache.insert(class_name.to_string(), properties);

                if supported {
                    PropertySupport::Supported
                } else {
                    PropertySupport::NotSupported(format!(
                        "Property '{}' not found in class '{}'",
                        property_name, class_name
                    ))
                }
            }
            Err(_) => PropertySupport::Unknown,
        }
    }

    /// Check if a processor setting property is supported.
    pub fn supports_processor_property(&mut self, property_name: &str) -> PropertySupport {
        self.supports_property("Msvm_ProcessorSettingData", property_name)
    }

    /// Check if a memory setting property is supported.
    pub fn supports_memory_property(&mut self, property_name: &str) -> PropertySupport {
        self.supports_property("Msvm_MemorySettingData", property_name)
    }

    /// Check if a system setting property is supported.
    pub fn supports_system_property(&mut self, property_name: &str) -> PropertySupport {
        self.supports_property("Msvm_VirtualSystemSettingData", property_name)
    }

    /// Check if a security setting property is supported.
    pub fn supports_security_property(&mut self, property_name: &str) -> PropertySupport {
        self.supports_property("Msvm_SecuritySettingData", property_name)
    }

    /// Check if HWThreadsPerCore property is supported.
    pub fn supports_hw_threads_per_core(&mut self) -> PropertySupport {
        self.supports_processor_property("HwThreadsPerCore")
    }

    /// Check if L3CacheWays property is supported.
    pub fn supports_l3_cache_ways(&mut self) -> PropertySupport {
        self.supports_processor_property("L3CacheWays")
    }

    /// Check if ExposeVirtualizationExtensions property is supported.
    pub fn supports_expose_virtualization_extensions(&mut self) -> PropertySupport {
        self.supports_processor_property("ExposeVirtualizationExtensions")
    }

    /// Check if CpuGroupId property is supported.
    pub fn supports_cpu_group_id(&mut self) -> PropertySupport {
        self.supports_processor_property("CpuGroupId")
    }

    /// Check if EnableHostResourceProtection property is supported.
    pub fn supports_host_resource_protection(&mut self) -> PropertySupport {
        self.supports_processor_property("EnableHostResourceProtection")
    }

    /// Check if HierarchicalVirtualization properties are supported.
    pub fn supports_hierarchical_virtualization(&mut self) -> PropertySupport {
        self.supports_processor_property("MaxHierarchicalPartitions")
    }

    /// Load all property names for a WMI class.
    fn load_class_properties(&self, class_name: &str) -> Result<Vec<String>> {
        use windows::Win32::System::Wmi::WBEM_FLAG_NONSYSTEM_ONLY;

        let class_def = self.connection.get_class(class_name)?;

        let mut properties = Vec::new();

        unsafe {
            // Begin enumeration of properties
            let hr = class_def.BeginEnumeration(WBEM_FLAG_NONSYSTEM_ONLY.0);
            if hr.is_err() {
                return Ok(properties);
            }

            loop {
                let mut name = windows::core::BSTR::new();
                let hr = class_def.Next(
                    0,
                    &mut name,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );

                if hr.is_err() || name.is_empty() {
                    break;
                }

                if let Ok(prop_name) = String::try_from(&name) {
                    properties.push(prop_name);
                }
            }

            let _ = class_def.EndEnumeration();
        }

        Ok(properties)
    }

    /// Clear the class definition cache.
    pub fn clear_cache(&mut self) {
        self.class_cache.clear();
    }
}

/// Well-known processor setting properties.
pub mod processor_properties {
    pub const HW_THREADS_PER_CORE: &str = "HwThreadsPerCore";
    pub const L3_CACHE_WAYS: &str = "L3CacheWays";
    pub const EXPOSE_VIRTUALIZATION_EXTENSIONS: &str = "ExposeVirtualizationExtensions";
    pub const CPU_GROUP_ID: &str = "CpuGroupId";
    pub const LIMIT: &str = "Limit";
    pub const RESERVATION: &str = "Reservation";
    pub const WEIGHT: &str = "Weight";
    pub const MAX_HIERARCHICAL_PARTITIONS: &str = "MaxHierarchicalPartitions";
    pub const MAX_HIERARCHICAL_VPS: &str = "MaxHierarchicalVps";
    pub const ENABLE_HOST_RESOURCE_PROTECTION: &str = "EnableHostResourceProtection";
    pub const MAX_PROCESSORS_PER_NUMA_NODE: &str = "MaxProcessorsPerNumaNode";
    pub const MAX_NUMA_NODES_PER_SOCKET: &str = "MaxNumaNodesPerSocket";
}

/// Well-known memory setting properties.
pub mod memory_properties {
    pub const DYNAMIC_MEMORY_ENABLED: &str = "DynamicMemoryEnabled";
    pub const VIRTUAL_QUANTITY: &str = "VirtualQuantity";
    pub const RESERVATION: &str = "Reservation";
    pub const LIMIT: &str = "Limit";
    pub const TARGET_MEMORY_BUFFER: &str = "TargetMemoryBuffer";
    pub const HUGE_PAGES_ENABLED: &str = "HugePagesEnabled";
    pub const SGX_ENABLED: &str = "SgxEnabled";
    pub const SGX_SIZE: &str = "SgxSize";
}

/// Well-known system setting properties.
pub mod system_properties {
    pub const VM_CONFIGURATION_VERSION: &str = "Version";
    pub const VIRTUAL_SYSTEM_SUBTYPE: &str = "VirtualSystemSubType";
    pub const SECURE_BOOT_ENABLED: &str = "SecureBootEnabled";
    pub const TURN_OFF_ON_GUEST_RESTART: &str = "TurnOffOnGuestRestart";
    pub const AUTOMATIC_START_ACTION: &str = "AutomaticStartupAction";
    pub const AUTOMATIC_STOP_ACTION: &str = "AutomaticShutdownAction";
    pub const AUTOMATIC_START_DELAY: &str = "AutomaticStartupActionDelay";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_support_is_supported() {
        assert!(PropertySupport::Supported.is_supported());
        assert!(!PropertySupport::NotSupported("test".to_string()).is_supported());
        assert!(!PropertySupport::Unknown.is_supported());
    }

    #[test]
    fn test_property_support_require() {
        assert!(PropertySupport::Supported.require("test").is_ok());
        assert!(PropertySupport::NotSupported("reason".to_string())
            .require("test")
            .is_err());
        assert!(PropertySupport::Unknown.require("test").is_err());
    }

    #[test]
    fn test_property_support_require_error_contains_property_name() {
        let result = PropertySupport::NotSupported("not available".to_string())
            .require("MyProperty");

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_string = format!("{}", err);
        assert!(err_string.contains("MyProperty"));
    }

    #[test]
    fn test_property_support_equality() {
        assert_eq!(PropertySupport::Supported, PropertySupport::Supported);
        assert_eq!(PropertySupport::Unknown, PropertySupport::Unknown);
        assert_eq!(
            PropertySupport::NotSupported("reason".to_string()),
            PropertySupport::NotSupported("reason".to_string())
        );
        assert_ne!(
            PropertySupport::NotSupported("reason1".to_string()),
            PropertySupport::NotSupported("reason2".to_string())
        );
        assert_ne!(PropertySupport::Supported, PropertySupport::Unknown);
    }

    #[test]
    fn test_processor_property_constants() {
        // Verify constants are non-empty and valid WMI property names
        assert!(!processor_properties::HW_THREADS_PER_CORE.is_empty());
        assert!(!processor_properties::L3_CACHE_WAYS.is_empty());
        assert!(!processor_properties::EXPOSE_VIRTUALIZATION_EXTENSIONS.is_empty());
        assert!(!processor_properties::CPU_GROUP_ID.is_empty());
        assert!(!processor_properties::LIMIT.is_empty());
        assert!(!processor_properties::RESERVATION.is_empty());
        assert!(!processor_properties::WEIGHT.is_empty());

        // WMI property names should not contain spaces
        assert!(!processor_properties::HW_THREADS_PER_CORE.contains(' '));
        assert!(!processor_properties::MAX_HIERARCHICAL_PARTITIONS.contains(' '));
    }

    #[test]
    fn test_memory_property_constants() {
        assert!(!memory_properties::DYNAMIC_MEMORY_ENABLED.is_empty());
        assert!(!memory_properties::VIRTUAL_QUANTITY.is_empty());
        assert!(!memory_properties::HUGE_PAGES_ENABLED.is_empty());
        assert!(!memory_properties::SGX_ENABLED.is_empty());

        // WMI property names should not contain spaces
        assert!(!memory_properties::DYNAMIC_MEMORY_ENABLED.contains(' '));
        assert!(!memory_properties::TARGET_MEMORY_BUFFER.contains(' '));
    }

    #[test]
    fn test_system_property_constants() {
        assert!(!system_properties::VM_CONFIGURATION_VERSION.is_empty());
        assert!(!system_properties::SECURE_BOOT_ENABLED.is_empty());
        assert!(!system_properties::AUTOMATIC_START_ACTION.is_empty());

        // WMI property names should not contain spaces
        assert!(!system_properties::TURN_OFF_ON_GUEST_RESTART.contains(' '));
    }
}
