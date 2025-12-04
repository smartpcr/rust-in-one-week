//! Msvm_* WMI classes for Hyper-V management
//!
//! Provides access to Hyper-V VMs, switches, and related resources through WMI.

use super::{hyperv, WmiObject};
use crate::error::Result;

/// Represents a Hyper-V VM from WMI (Msvm_ComputerSystem)
#[derive(Debug, Clone)]
pub struct MsvmVm {
    /// VM name (ElementName)
    pub name: String,
    /// VM GUID (Name property)
    pub id: String,
    /// Enabled state
    pub enabled_state: hyperv::EnabledState,
    /// Health state
    pub health_state: u32,
    /// Number of processors
    pub processor_count: Option<u32>,
    /// Memory in MB
    pub memory_mb: Option<u64>,
    /// VM generation (1 or 2)
    pub generation: Option<u32>,
    /// Creation time
    pub creation_time: Option<String>,
    /// Notes/description
    pub notes: Option<String>,
    /// WMI object path for operations
    pub path: String,
}

impl MsvmVm {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmVm {
            name: obj.get_string_required("ElementName")?,
            id: obj.get_string_required("Name")?,
            enabled_state: hyperv::EnabledState::from(obj.get_u32("EnabledState")?.unwrap_or(0)),
            health_state: obj.get_u32("HealthState")?.unwrap_or(0),
            processor_count: None, // Fetched separately from settings
            memory_mb: None,       // Fetched separately from settings
            generation: None,      // Fetched separately from settings
            creation_time: obj.get_string("InstallDate")?,
            notes: obj.get_string("Description")?,
            path: obj.path()?,
        })
    }

    /// Check if VM is running
    pub fn is_running(&self) -> bool {
        matches!(self.enabled_state, hyperv::EnabledState::Enabled)
    }

    /// Check if VM is off
    pub fn is_off(&self) -> bool {
        matches!(self.enabled_state, hyperv::EnabledState::Disabled)
    }

    /// Get human-readable state string
    pub fn state_string(&self) -> &'static str {
        match self.enabled_state {
            hyperv::EnabledState::Enabled => "Running",
            hyperv::EnabledState::Disabled => "Off",
            hyperv::EnabledState::Paused => "Paused",
            hyperv::EnabledState::Suspended => "Saved",
            hyperv::EnabledState::Starting | hyperv::EnabledState::Starting2 => "Starting",
            hyperv::EnabledState::Stopping => "Stopping",
            hyperv::EnabledState::Saving => "Saving",
            hyperv::EnabledState::Pausing => "Pausing",
            hyperv::EnabledState::Resuming => "Resuming",
            hyperv::EnabledState::ShuttingDown => "Shutting Down",
            _ => "Unknown",
        }
    }
}

/// Represents a Hyper-V virtual switch from WMI (Msvm_VirtualEthernetSwitch)
#[derive(Debug, Clone)]
pub struct MsvmSwitch {
    /// Switch name
    pub name: String,
    /// Switch GUID
    pub id: String,
    /// Switch type description
    pub switch_type: Option<String>,
    /// Health state
    pub health_state: u32,
    /// WMI object path
    pub path: String,
}

impl MsvmSwitch {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmSwitch {
            name: obj.get_string_required("ElementName")?,
            id: obj.get_string_required("Name")?,
            switch_type: obj.get_string("Description")?,
            health_state: obj.get_u32("HealthState")?.unwrap_or(0),
            path: obj.path()?,
        })
    }
}

/// Represents a Hyper-V snapshot from WMI (Msvm_VirtualSystemSettingData with VirtualSystemType containing "Recovery")
#[derive(Debug, Clone)]
pub struct MsvmSnapshot {
    /// Snapshot name
    pub name: String,
    /// Snapshot GUID (InstanceID)
    pub id: String,
    /// Parent VM ID
    pub vm_id: String,
    /// Creation time
    pub creation_time: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// WMI object path
    pub path: String,
}

impl MsvmSnapshot {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmSnapshot {
            name: obj.get_string_required("ElementName")?,
            id: obj.get_string_required("InstanceID")?,
            vm_id: obj
                .get_string("VirtualSystemIdentifier")?
                .unwrap_or_default(),
            creation_time: obj.get_string("CreationTime")?,
            notes: obj.get_string("Notes")?,
            path: obj.path()?,
        })
    }
}

/// VM settings data from Msvm_VirtualSystemSettingData
#[derive(Debug, Clone)]
pub struct MsvmVmSettings {
    /// Instance ID
    pub instance_id: String,
    /// VM name
    pub name: String,
    /// VM generation
    pub generation: Option<u32>,
    /// Notes
    pub notes: Option<String>,
    /// BIOS GUID
    pub bios_guid: Option<String>,
    /// Configuration path
    pub config_path: Option<String>,
    /// WMI path
    pub path: String,
}

impl MsvmVmSettings {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmVmSettings {
            instance_id: obj.get_string_required("InstanceID")?,
            name: obj.get_string_required("ElementName")?,
            generation: obj.get_u32("VirtualSystemSubType").ok().flatten().map(|v| {
                // VirtualSystemSubType contains generation info
                if v == 2 {
                    2
                } else {
                    1
                }
            }),
            notes: obj.get_string("Notes")?,
            bios_guid: obj.get_string("BIOSGUID")?,
            config_path: obj.get_string("ConfigurationDataRoot")?,
            path: obj.path()?,
        })
    }
}

/// Memory settings from Msvm_MemorySettingData
#[derive(Debug, Clone)]
pub struct MsvmMemorySettings {
    /// Virtual quantity in MB
    pub virtual_quantity_mb: u64,
    /// Dynamic memory enabled
    pub dynamic_memory_enabled: bool,
    /// Minimum memory if dynamic
    pub minimum_mb: Option<u64>,
    /// Maximum memory if dynamic
    pub maximum_mb: Option<u64>,
    /// Target memory percentage
    pub target_memory_buffer: Option<u32>,
}

impl MsvmMemorySettings {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmMemorySettings {
            virtual_quantity_mb: obj.get_u64("VirtualQuantity")?.unwrap_or(0),
            dynamic_memory_enabled: obj.get_bool("DynamicMemoryEnabled")?.unwrap_or(false),
            minimum_mb: obj.get_u64("Reservation")?,
            maximum_mb: obj.get_u64("Limit")?,
            target_memory_buffer: obj.get_u32("TargetMemoryBuffer")?,
        })
    }
}

/// Processor settings from Msvm_ProcessorSettingData
#[derive(Debug, Clone)]
pub struct MsvmProcessorSettings {
    /// Number of virtual processors
    pub virtual_quantity: u32,
    /// Limit (percentage of host CPU)
    pub limit: Option<u32>,
    /// Reservation (percentage of host CPU)
    pub reservation: Option<u32>,
    /// Weight for scheduling
    pub weight: Option<u32>,
}

impl MsvmProcessorSettings {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmProcessorSettings {
            virtual_quantity: obj.get_u32("VirtualQuantity")?.unwrap_or(1),
            limit: obj.get_u32("Limit")?,
            reservation: obj.get_u32("Reservation")?,
            weight: obj.get_u32("Weight")?,
        })
    }
}

/// Network adapter from Msvm_SyntheticEthernetPortSettingData
#[derive(Debug, Clone)]
pub struct MsvmNetworkAdapter {
    /// Adapter name
    pub name: String,
    /// MAC address (static or dynamic)
    pub mac_address: Option<String>,
    /// Static MAC address enabled
    pub static_mac_address: bool,
    /// Connected switch path
    pub switch_path: Option<String>,
    /// WMI path
    pub path: String,
}

impl MsvmNetworkAdapter {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        Ok(MsvmNetworkAdapter {
            name: obj.get_string("ElementName")?.unwrap_or_default(),
            mac_address: obj.get_string("Address")?,
            static_mac_address: obj.get_bool("StaticMacAddress")?.unwrap_or(false),
            switch_path: None, // Fetched via associations
            path: obj.path()?,
        })
    }
}

/// Hard disk drive from Msvm_StorageAllocationSettingData
#[derive(Debug, Clone)]
pub struct MsvmHardDisk {
    /// Disk name
    pub name: String,
    /// VHD path
    pub vhd_path: Option<String>,
    /// Controller number
    pub controller_number: Option<u32>,
    /// Controller location
    pub controller_location: Option<u32>,
    /// WMI path
    pub path: String,
}

impl MsvmHardDisk {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        // HostResource contains the VHD path
        let vhd_path = obj.get_string_array("HostResource")?.into_iter().next();

        Ok(MsvmHardDisk {
            name: obj.get_string("ElementName")?.unwrap_or_default(),
            vhd_path,
            controller_number: obj.get_u32("AddressOnParent").ok().flatten(),
            controller_location: None,
            path: obj.path()?,
        })
    }
}

/// DVD drive from Msvm_ResourceAllocationSettingData (ResourceType = 16)
#[derive(Debug, Clone)]
pub struct MsvmDvdDrive {
    /// Drive name
    pub name: String,
    /// Mounted ISO path
    pub iso_path: Option<String>,
    /// WMI path
    pub path: String,
}

impl MsvmDvdDrive {
    /// Parse from WMI object
    pub fn from_wmi(obj: &WmiObject) -> Result<Self> {
        let iso_path = obj.get_string_array("HostResource")?.into_iter().next();

        Ok(MsvmDvdDrive {
            name: obj.get_string("ElementName")?.unwrap_or_default(),
            iso_path,
            path: obj.path()?,
        })
    }
}
