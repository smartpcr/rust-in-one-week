use crate::error::{Error, Result};

/// Storage controller type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerType {
    /// IDE controller (Gen1 only, 2 channels, 2 devices each).
    Ide,
    /// SCSI controller (Gen1 and Gen2, up to 64 devices).
    Scsi,
}

impl ControllerType {
    pub fn wmi_class(&self) -> &'static str {
        match self {
            ControllerType::Ide => "Msvm_ResourceAllocationSettingData",
            ControllerType::Scsi => "Msvm_ResourceAllocationSettingData",
        }
    }

    pub fn resource_subtype(&self) -> &'static str {
        match self {
            ControllerType::Ide => "Microsoft:Hyper-V:Emulated IDE Controller",
            ControllerType::Scsi => "Microsoft:Hyper-V:Synthetic SCSI Controller",
        }
    }
}

/// Represents a storage controller attached to a VM.
#[derive(Debug)]
pub struct StorageController {
    /// Controller type.
    pub controller_type: ControllerType,
    /// Controller instance ID.
    pub instance_id: String,
    /// Controller address/location.
    pub address: u32,
    /// WMI path.
    #[allow(dead_code)]
    pub(crate) path: String,
}

/// Disk attachment settings.
#[derive(Debug, Clone)]
pub struct DiskAttachment {
    /// Path to VHD/VHDX file.
    pub vhd_path: String,
    /// Controller type.
    pub controller_type: ControllerType,
    /// Controller number (0-3 for SCSI, 0-1 for IDE).
    pub controller_number: u32,
    /// Controller location (0-63 for SCSI, 0-1 for IDE).
    pub controller_location: u32,
}

impl DiskAttachment {
    /// Create a new disk attachment.
    pub fn new(vhd_path: impl Into<String>) -> Self {
        Self {
            vhd_path: vhd_path.into(),
            controller_type: ControllerType::Scsi,
            controller_number: 0,
            controller_location: 0,
        }
    }

    /// Set controller type.
    pub fn controller_type(mut self, controller_type: ControllerType) -> Self {
        self.controller_type = controller_type;
        self
    }

    /// Set controller number.
    pub fn controller_number(mut self, number: u32) -> Self {
        self.controller_number = number;
        self
    }

    /// Set controller location.
    pub fn controller_location(mut self, location: u32) -> Self {
        self.controller_location = location;
        self
    }

    /// Validate the attachment settings.
    pub fn validate(&self) -> Result<()> {
        if self.vhd_path.is_empty() {
            return Err(Error::Validation {
                field: "vhd_path",
                message: "VHD path cannot be empty".to_string(),
            });
        }

        match self.controller_type {
            ControllerType::Ide => {
                if self.controller_number > 1 {
                    return Err(Error::Validation {
                        field: "controller_number",
                        message: "IDE controller number must be 0 or 1".to_string(),
                    });
                }
                if self.controller_location > 1 {
                    return Err(Error::Validation {
                        field: "controller_location",
                        message: "IDE controller location must be 0 or 1".to_string(),
                    });
                }
            }
            ControllerType::Scsi => {
                if self.controller_number > 3 {
                    return Err(Error::Validation {
                        field: "controller_number",
                        message: "SCSI controller number must be 0-3".to_string(),
                    });
                }
                if self.controller_location > 63 {
                    return Err(Error::Validation {
                        field: "controller_location",
                        message: "SCSI controller location must be 0-63".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// ISO attachment settings.
#[derive(Debug, Clone)]
pub struct IsoAttachment {
    /// Path to ISO file.
    pub iso_path: String,
    /// Controller type (usually IDE for Gen1, SCSI for Gen2).
    pub controller_type: ControllerType,
    /// Controller number.
    pub controller_number: u32,
    /// Controller location.
    pub controller_location: u32,
}

impl IsoAttachment {
    /// Create a new ISO attachment.
    pub fn new(iso_path: impl Into<String>) -> Self {
        Self {
            iso_path: iso_path.into(),
            controller_type: ControllerType::Ide,
            controller_number: 1, // Secondary IDE by default
            controller_location: 0,
        }
    }

    /// Set controller type.
    pub fn controller_type(mut self, controller_type: ControllerType) -> Self {
        self.controller_type = controller_type;
        self
    }

    /// Set controller number.
    pub fn controller_number(mut self, number: u32) -> Self {
        self.controller_number = number;
        self
    }

    /// Set controller location.
    pub fn controller_location(mut self, location: u32) -> Self {
        self.controller_location = location;
        self
    }

    /// Validate the attachment settings.
    pub fn validate(&self) -> Result<()> {
        if self.iso_path.is_empty() {
            return Err(Error::Validation {
                field: "iso_path",
                message: "ISO path cannot be empty".to_string(),
            });
        }

        if !self.iso_path.to_lowercase().ends_with(".iso") {
            return Err(Error::Validation {
                field: "iso_path",
                message: "ISO path must end with .iso".to_string(),
            });
        }

        // Same controller validation as DiskAttachment
        match self.controller_type {
            ControllerType::Ide => {
                if self.controller_number > 1 {
                    return Err(Error::Validation {
                        field: "controller_number",
                        message: "IDE controller number must be 0 or 1".to_string(),
                    });
                }
                if self.controller_location > 1 {
                    return Err(Error::Validation {
                        field: "controller_location",
                        message: "IDE controller location must be 0 or 1".to_string(),
                    });
                }
            }
            ControllerType::Scsi => {
                if self.controller_number > 3 {
                    return Err(Error::Validation {
                        field: "controller_number",
                        message: "SCSI controller number must be 0-3".to_string(),
                    });
                }
                if self.controller_location > 63 {
                    return Err(Error::Validation {
                        field: "controller_location",
                        message: "SCSI controller location must be 0-63".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}
