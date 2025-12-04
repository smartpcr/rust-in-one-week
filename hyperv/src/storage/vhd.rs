use crate::error::{Error, Result};
use crate::wmi::{WbemClassObjectExt, WmiConnection};

/// Virtual hard disk format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VhdFormat {
    /// Legacy VHD format (max 2 TB).
    Vhd,
    /// Modern VHDX format (max 64 TB).
    #[default]
    Vhdx,
}

impl VhdFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VhdFormat::Vhd => "vhd",
            VhdFormat::Vhdx => "vhdx",
        }
    }

    pub fn from_path(path: &str) -> Self {
        if path.to_lowercase().ends_with(".vhd") {
            VhdFormat::Vhd
        } else {
            VhdFormat::Vhdx
        }
    }
}

/// Virtual hard disk type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VhdType {
    /// Fixed size - all space allocated upfront.
    Fixed,
    /// Dynamic - grows as needed.
    #[default]
    Dynamic,
    /// Differencing - based on parent disk.
    Differencing,
}

impl VhdType {
    pub fn to_wmi_value(&self) -> u16 {
        match self {
            VhdType::Fixed => 2,
            VhdType::Dynamic => 3,
            VhdType::Differencing => 4,
        }
    }

    pub fn from_wmi_value(value: u16) -> Self {
        match value {
            2 => VhdType::Fixed,
            3 => VhdType::Dynamic,
            4 => VhdType::Differencing,
            _ => VhdType::Dynamic,
        }
    }
}

/// Represents a virtual hard disk.
#[derive(Debug)]
pub struct Vhd {
    /// Full path to the VHD/VHDX file.
    pub path: String,
    /// Disk format.
    pub format: VhdFormat,
    /// Disk type.
    pub disk_type: VhdType,
    /// Maximum size in bytes.
    pub max_size_bytes: u64,
    /// Current file size in bytes.
    pub file_size_bytes: u64,
    /// Block size in bytes.
    pub block_size_bytes: u32,
    /// Logical sector size.
    pub logical_sector_size: u32,
    /// Physical sector size.
    pub physical_sector_size: u32,
    /// Parent path for differencing disks.
    pub parent_path: Option<String>,
}

/// Settings for creating a new VHD.
#[derive(Debug, Clone)]
pub struct VhdSettings {
    /// Path where the VHD will be created.
    pub path: String,
    /// Disk format.
    pub format: VhdFormat,
    /// Disk type.
    pub disk_type: VhdType,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Block size in bytes (default: 32 MB for VHDX, 2 MB for VHD).
    pub block_size_bytes: Option<u32>,
    /// Logical sector size (512 or 4096).
    pub logical_sector_size: Option<u32>,
    /// Physical sector size (512 or 4096).
    pub physical_sector_size: Option<u32>,
    /// Parent path for differencing disks.
    pub parent_path: Option<String>,
}

impl VhdSettings {
    pub fn builder() -> VhdSettingsBuilder {
        VhdSettingsBuilder::default()
    }

    pub fn validate(&self) -> Result<()> {
        if self.path.is_empty() {
            return Err(Error::Validation {
                field: "path",
                message: "VHD path cannot be empty".to_string(),
            });
        }

        // Validate extension matches format
        let path_lower = self.path.to_lowercase();
        match self.format {
            VhdFormat::Vhd => {
                if !path_lower.ends_with(".vhd") {
                    return Err(Error::Validation {
                        field: "path",
                        message: "VHD format requires .vhd extension".to_string(),
                    });
                }
            }
            VhdFormat::Vhdx => {
                if !path_lower.ends_with(".vhdx") {
                    return Err(Error::Validation {
                        field: "path",
                        message: "VHDX format requires .vhdx extension".to_string(),
                    });
                }
            }
        }

        // Size validation
        if self.size_bytes == 0 && self.disk_type != VhdType::Differencing {
            return Err(Error::Validation {
                field: "size_bytes",
                message: "Size must be greater than 0".to_string(),
            });
        }

        // VHD max size is 2 TB
        if self.format == VhdFormat::Vhd && self.size_bytes > 2 * 1024 * 1024 * 1024 * 1024 {
            return Err(Error::Validation {
                field: "size_bytes",
                message: "VHD format supports maximum 2 TB".to_string(),
            });
        }

        // VHDX max size is 64 TB
        if self.format == VhdFormat::Vhdx && self.size_bytes > 64 * 1024 * 1024 * 1024 * 1024 {
            return Err(Error::Validation {
                field: "size_bytes",
                message: "VHDX format supports maximum 64 TB".to_string(),
            });
        }

        // Differencing disk requires parent
        if self.disk_type == VhdType::Differencing && self.parent_path.is_none() {
            return Err(Error::Validation {
                field: "parent_path",
                message: "Differencing disk requires parent path".to_string(),
            });
        }

        // Sector size validation
        if let Some(sector) = self.logical_sector_size {
            if sector != 512 && sector != 4096 {
                return Err(Error::Validation {
                    field: "logical_sector_size",
                    message: "Logical sector size must be 512 or 4096".to_string(),
                });
            }
        }

        if let Some(sector) = self.physical_sector_size {
            if sector != 512 && sector != 4096 {
                return Err(Error::Validation {
                    field: "physical_sector_size",
                    message: "Physical sector size must be 512 or 4096".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Builder for VHD settings.
#[derive(Default)]
pub struct VhdSettingsBuilder {
    path: Option<String>,
    format: VhdFormat,
    disk_type: VhdType,
    size_bytes: Option<u64>,
    block_size_bytes: Option<u32>,
    logical_sector_size: Option<u32>,
    physical_sector_size: Option<u32>,
    parent_path: Option<String>,
}

impl VhdSettingsBuilder {
    /// Set VHD file path (required).
    pub fn path(mut self, path: impl Into<String>) -> Self {
        let p = path.into();
        self.format = VhdFormat::from_path(&p);
        self.path = Some(p);
        self
    }

    /// Set disk format. Auto-detected from path if not specified.
    pub fn format(mut self, format: VhdFormat) -> Self {
        self.format = format;
        self
    }

    /// Set disk type.
    pub fn disk_type(mut self, disk_type: VhdType) -> Self {
        self.disk_type = disk_type;
        self
    }

    /// Set size in bytes.
    pub fn size_bytes(mut self, size: u64) -> Self {
        self.size_bytes = Some(size);
        self
    }

    /// Set size in gigabytes.
    pub fn size_gb(mut self, gb: u64) -> Self {
        self.size_bytes = Some(gb * 1024 * 1024 * 1024);
        self
    }

    /// Set block size in bytes.
    pub fn block_size_bytes(mut self, size: u32) -> Self {
        self.block_size_bytes = Some(size);
        self
    }

    /// Set logical sector size (512 or 4096).
    pub fn logical_sector_size(mut self, size: u32) -> Self {
        self.logical_sector_size = Some(size);
        self
    }

    /// Set physical sector size (512 or 4096).
    pub fn physical_sector_size(mut self, size: u32) -> Self {
        self.physical_sector_size = Some(size);
        self
    }

    /// Set parent path for differencing disk.
    pub fn parent_path(mut self, path: impl Into<String>) -> Self {
        self.parent_path = Some(path.into());
        self.disk_type = VhdType::Differencing;
        self
    }

    /// Build and validate the settings.
    pub fn build(self) -> Result<VhdSettings> {
        let settings = VhdSettings {
            path: self.path.ok_or(Error::MissingRequired("path"))?,
            format: self.format,
            disk_type: self.disk_type,
            size_bytes: self.size_bytes.unwrap_or(0),
            block_size_bytes: self.block_size_bytes,
            logical_sector_size: self.logical_sector_size,
            physical_sector_size: self.physical_sector_size,
            parent_path: self.parent_path,
        };

        settings.validate()?;
        Ok(settings)
    }
}

/// VHD management operations.
pub struct VhdManager {
    connection: std::sync::Arc<WmiConnection>,
}

impl VhdManager {
    pub(crate) fn new(connection: std::sync::Arc<WmiConnection>) -> Self {
        Self { connection }
    }

    /// Create a new VHD/VHDX.
    pub fn create(&self, settings: &VhdSettings) -> Result<Vhd> {
        settings.validate()?;

        // Get the ImageManagementService
        let service = self.get_image_service()?;
        let service_path = service.get_path()?;

        // Create VirtualHardDiskSettingData
        let vhd_settings = self.connection.spawn_instance("Msvm_VirtualHardDiskSettingData")?;
        vhd_settings.put_string("Path", &settings.path)?;
        vhd_settings.put_u16("Type", settings.disk_type.to_wmi_value())?;
        vhd_settings.put_u16("Format", if settings.format == VhdFormat::Vhdx { 3 } else { 2 })?;
        vhd_settings.put_u64("MaxInternalSize", settings.size_bytes)?;

        if let Some(block_size) = settings.block_size_bytes {
            vhd_settings.put_u32("BlockSize", block_size)?;
        }
        if let Some(logical) = settings.logical_sector_size {
            vhd_settings.put_u32("LogicalSectorSize", logical)?;
        }
        if let Some(physical) = settings.physical_sector_size {
            vhd_settings.put_u32("PhysicalSectorSize", physical)?;
        }
        if let Some(ref parent) = settings.parent_path {
            vhd_settings.put_string("ParentPath", parent)?;
        }

        let settings_text = vhd_settings.get_text()?;

        // Call CreateVirtualHardDisk
        let in_params = self.connection.get_method_params(
            "Msvm_ImageManagementService",
            "CreateVirtualHardDisk",
        )?;
        in_params.put_string("VirtualDiskSettingData", &settings_text)?;

        let out_params = self.connection.exec_method(
            &service_path,
            "CreateVirtualHardDisk",
            Some(&in_params),
        )?;

        self.handle_job_result(&out_params, "CreateVirtualHardDisk")?;

        // Return info about created VHD
        self.get_info(&settings.path)
    }

    /// Get information about an existing VHD.
    pub fn get_info(&self, path: &str) -> Result<Vhd> {
        let service = self.get_image_service()?;
        let service_path = service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_ImageManagementService",
            "GetVirtualHardDiskSettingData",
        )?;
        in_params.put_string("Path", path)?;

        let out_params = self.connection.exec_method(
            &service_path,
            "GetVirtualHardDiskSettingData",
            Some(&in_params),
        )?;

        self.handle_job_result(&out_params, "GetVirtualHardDiskSettingData")?;

        // For now, return basic info - full implementation would parse embedded object
        // TODO: Parse the returned embedded settings from GetVirtualHardDiskSettingData
        Ok(Vhd {
            path: path.to_string(),
            format: VhdFormat::from_path(path),
            disk_type: VhdType::Dynamic,
            max_size_bytes: 0,
            file_size_bytes: 0,
            block_size_bytes: 0,
            logical_sector_size: 512,
            physical_sector_size: 4096,
            parent_path: None,
        })
    }

    /// Resize a VHD.
    pub fn resize(&self, path: &str, new_size_bytes: u64) -> Result<()> {
        let service = self.get_image_service()?;
        let service_path = service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_ImageManagementService",
            "ResizeVirtualHardDisk",
        )?;
        in_params.put_string("Path", path)?;
        in_params.put_u64("MaxInternalSize", new_size_bytes)?;

        let out_params = self.connection.exec_method(
            &service_path,
            "ResizeVirtualHardDisk",
            Some(&in_params),
        )?;

        self.handle_job_result(&out_params, "ResizeVirtualHardDisk")
    }

    /// Convert VHD between formats or types.
    pub fn convert(&self, source_path: &str, dest_settings: &VhdSettings) -> Result<Vhd> {
        dest_settings.validate()?;

        let service = self.get_image_service()?;
        let service_path = service.get_path()?;

        let vhd_settings = self.connection.spawn_instance("Msvm_VirtualHardDiskSettingData")?;
        vhd_settings.put_string("Path", &dest_settings.path)?;
        vhd_settings.put_u16("Type", dest_settings.disk_type.to_wmi_value())?;
        vhd_settings.put_u16("Format", if dest_settings.format == VhdFormat::Vhdx { 3 } else { 2 })?;

        let settings_text = vhd_settings.get_text()?;

        let in_params = self.connection.get_method_params(
            "Msvm_ImageManagementService",
            "ConvertVirtualHardDisk",
        )?;
        in_params.put_string("SourcePath", source_path)?;
        in_params.put_string("VirtualDiskSettingData", &settings_text)?;

        let out_params = self.connection.exec_method(
            &service_path,
            "ConvertVirtualHardDisk",
            Some(&in_params),
        )?;

        self.handle_job_result(&out_params, "ConvertVirtualHardDisk")?;

        self.get_info(&dest_settings.path)
    }

    /// Compact a dynamic VHD.
    pub fn compact(&self, path: &str) -> Result<()> {
        let service = self.get_image_service()?;
        let service_path = service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_ImageManagementService",
            "CompactVirtualHardDisk",
        )?;
        in_params.put_string("Path", path)?;
        in_params.put_u16("Mode", 0)?; // Full mode

        let out_params = self.connection.exec_method(
            &service_path,
            "CompactVirtualHardDisk",
            Some(&in_params),
        )?;

        self.handle_job_result(&out_params, "CompactVirtualHardDisk")
    }

    /// Merge a differencing disk into its parent.
    pub fn merge(&self, path: &str) -> Result<()> {
        let service = self.get_image_service()?;
        let service_path = service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_ImageManagementService",
            "MergeVirtualHardDisk",
        )?;
        in_params.put_string("SourcePath", path)?;

        let out_params = self.connection.exec_method(
            &service_path,
            "MergeVirtualHardDisk",
            Some(&in_params),
        )?;

        self.handle_job_result(&out_params, "MergeVirtualHardDisk")
    }

    fn get_image_service(&self) -> Result<windows::Win32::System::Wmi::IWbemClassObject> {
        self.connection
            .query_first("SELECT * FROM Msvm_ImageManagementService")?
            .ok_or_else(|| Error::WmiQuery {
                query: "Msvm_ImageManagementService".to_string(),
                source: windows_core::Error::from_hresult(windows_core::HRESULT(-1)),
            })
    }

    fn handle_job_result(
        &self,
        out_params: &windows::Win32::System::Wmi::IWbemClassObject,
        operation: &'static str,
    ) -> Result<()> {
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

        match return_value {
            0 => Ok(()),
            4096 => {
                // Job started
                if let Some(job_path) = out_params.get_string_prop("Job")? {
                    self.wait_for_job(&job_path, operation)
                } else {
                    Ok(())
                }
            }
            code => Err(Error::OperationFailed {
                operation,
                return_value: code,
                message: format!("{} failed", operation),
            }),
        }
    }

    fn wait_for_job(&self, job_path: &str, operation: &'static str) -> Result<()> {
        loop {
            let job = self.connection.get_object(job_path)?;
            let job_state = job.get_u16("JobState")?.unwrap_or(0);

            match job_state {
                7 => return Ok(()), // Completed
                8 | 9 | 10 | 11 => {
                    let error_code = job.get_u32("ErrorCode")?.unwrap_or(0);
                    let error_desc = job.get_string_prop("ErrorDescription")?.unwrap_or_default();
                    return Err(Error::JobFailed {
                        operation,
                        error_code,
                        error_description: error_desc,
                    });
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
}
