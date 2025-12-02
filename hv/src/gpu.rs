//! GPU enumeration and management for Hyper-V
//!
//! Provides GPU discovery, capability checking, GPU-P partition adapter management,
//! and DDA (Discrete Device Assignment) support.

use crate::error::{HvError, Result};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use windows::core::PCWSTR;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
    SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW, DIGCF_PRESENT,
    HDEVINFO, SP_DEVINFO_DATA, SPDRP_DEVICEDESC, SPDRP_DRIVER, SPDRP_FRIENDLYNAME,
    SPDRP_HARDWAREID, SPDRP_LOCATION_INFORMATION, SPDRP_MFG,
};
use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;

/// GUID for Display Adapters device class
const GUID_DEVCLASS_DISPLAY: windows::core::GUID =
    windows::core::GUID::from_u128(0x4d36e968_e325_11ce_bfc1_08002be10318);

/// Convert Rust string to wide string (UTF-16 null-terminated)
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// RAII wrapper for HDEVINFO
struct DeviceInfoSet(HDEVINFO);

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        unsafe {
            let _ = SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

/// Information about a GPU device on the host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// Device instance ID (used for GPU-P assignment)
    pub device_instance_id: String,
    /// Friendly name (e.g., "NVIDIA GeForce RTX 3080")
    pub name: String,
    /// Device description
    pub description: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Hardware IDs
    pub hardware_ids: Vec<String>,
    /// Driver information
    pub driver: Option<String>,
    /// Location information (e.g., "PCI bus 1, device 0, function 0")
    pub location: Option<String>,
    /// Whether this GPU supports partitioning (GPU-P)
    pub supports_partitioning: bool,
}

/// GPU partition adapter configuration for a VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPartitionAdapter {
    /// VM name
    pub vm_name: String,
    /// Instance path of the GPU
    pub instance_path: Option<String>,
    /// Minimum partition VRAM in bytes
    pub min_partition_vram: Option<u64>,
    /// Maximum partition VRAM in bytes
    pub max_partition_vram: Option<u64>,
    /// Optimal partition VRAM in bytes
    pub optimal_partition_vram: Option<u64>,
    /// Minimum partition encode capacity (0-100)
    pub min_partition_encode: Option<u64>,
    /// Maximum partition encode capacity (0-100)
    pub max_partition_encode: Option<u64>,
    /// Optimal partition encode capacity (0-100)
    pub optimal_partition_encode: Option<u64>,
    /// Minimum partition decode capacity (0-100)
    pub min_partition_decode: Option<u64>,
    /// Maximum partition decode capacity (0-100)
    pub max_partition_decode: Option<u64>,
    /// Optimal partition decode capacity (0-100)
    pub optimal_partition_decode: Option<u64>,
    /// Minimum partition compute capacity (0-100)
    pub min_partition_compute: Option<u64>,
    /// Maximum partition compute capacity (0-100)
    pub max_partition_compute: Option<u64>,
    /// Optimal partition compute capacity (0-100)
    pub optimal_partition_compute: Option<u64>,
}

impl Default for GpuPartitionAdapter {
    fn default() -> Self {
        GpuPartitionAdapter {
            vm_name: String::new(),
            instance_path: None,
            min_partition_vram: None,
            max_partition_vram: None,
            optimal_partition_vram: None,
            min_partition_encode: None,
            max_partition_encode: None,
            optimal_partition_encode: None,
            min_partition_decode: None,
            max_partition_decode: None,
            optimal_partition_decode: None,
            min_partition_compute: None,
            max_partition_compute: None,
            optimal_partition_compute: None,
        }
    }
}

/// Enumerate all GPU devices on the host
pub fn enumerate_gpus() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();

    unsafe {
        // Get device info set for display adapters
        let device_info_set = SetupDiGetClassDevsW(
            Some(&GUID_DEVCLASS_DISPLAY),
            PCWSTR::null(),
            None,
            DIGCF_PRESENT,
        );

        if device_info_set.is_invalid() {
            return Err(HvError::OperationFailed("Failed to get device info set".to_string()));
        }

        let _guard = DeviceInfoSet(device_info_set);

        let mut index = 0u32;
        loop {
            let mut dev_info_data = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };

            let result = SetupDiEnumDeviceInfo(device_info_set, index, &mut dev_info_data);
            if result.is_err() {
                let err = windows::core::Error::from_win32();
                if err.code() == ERROR_NO_MORE_ITEMS.into() {
                    break;
                }
                // Skip this device and continue
                index += 1;
                continue;
            }

            // Get device instance ID
            let device_instance_id = get_device_instance_id(device_info_set, &dev_info_data)
                .unwrap_or_default();

            // Get device properties
            let name = get_device_property(device_info_set, &dev_info_data, SPDRP_FRIENDLYNAME)
                .or_else(|| get_device_property(device_info_set, &dev_info_data, SPDRP_DEVICEDESC))
                .unwrap_or_else(|| "Unknown GPU".to_string());

            let description = get_device_property(device_info_set, &dev_info_data, SPDRP_DEVICEDESC)
                .unwrap_or_default();

            let manufacturer = get_device_property(device_info_set, &dev_info_data, SPDRP_MFG)
                .unwrap_or_default();

            let hardware_ids = get_device_property(device_info_set, &dev_info_data, SPDRP_HARDWAREID)
                .map(|s| s.split('\0').filter(|s| !s.is_empty()).map(String::from).collect())
                .unwrap_or_default();

            let driver = get_device_property(device_info_set, &dev_info_data, SPDRP_DRIVER);

            let location = get_device_property(device_info_set, &dev_info_data, SPDRP_LOCATION_INFORMATION);

            // Check if GPU supports partitioning
            let supports_partitioning = check_gpu_partitioning_support(&device_instance_id);

            gpus.push(GpuInfo {
                device_instance_id,
                name,
                description,
                manufacturer,
                hardware_ids,
                driver,
                location,
                supports_partitioning,
            });

            index += 1;
        }
    }

    Ok(gpus)
}

/// Get device instance ID
unsafe fn get_device_instance_id(device_info_set: HDEVINFO, dev_info_data: &SP_DEVINFO_DATA) -> Option<String> {
    let mut buffer = vec![0u16; 512];
    let mut required_size = 0u32;

    let result = SetupDiGetDeviceInstanceIdW(
        device_info_set,
        dev_info_data,
        Some(&mut buffer),
        Some(&mut required_size),
    );

    if result.is_ok() {
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..len]))
    } else {
        None
    }
}

/// Get a device registry property as string
unsafe fn get_device_property(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
    property: u32,
) -> Option<String> {
    let mut buffer = vec![0u8; 4096];
    let mut required_size = 0u32;
    let mut reg_type = 0u32;

    let result = SetupDiGetDeviceRegistryPropertyW(
        device_info_set,
        dev_info_data,
        property,
        Some(&mut reg_type),
        Some(&mut buffer),
        Some(&mut required_size),
    );

    if result.is_ok() && required_size > 0 {
        // Convert UTF-16 to string
        let wide_slice: &[u16] = std::slice::from_raw_parts(
            buffer.as_ptr() as *const u16,
            (required_size as usize) / 2,
        );
        let len = wide_slice.iter().position(|&c| c == 0).unwrap_or(wide_slice.len());
        Some(String::from_utf16_lossy(&wide_slice[..len]))
    } else {
        None
    }
}

/// Check if a GPU supports partitioning (GPU-P)
fn check_gpu_partitioning_support(device_instance_id: &str) -> bool {
    // Query via PowerShell if GPU supports partitioning
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                $gpu = Get-WmiObject -Class Msvm_Physical3dGraphicsProcessor -Namespace 'root\virtualization\v2' |
                    Where-Object {{ $_.Name -like '*{}*' }}
                if ($gpu -and $gpu.EnabledForVirtualization) {{ 'true' }} else {{ 'false' }}
                "#,
                device_instance_id.replace('\\', "\\\\")
            ),
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_lowercase() == "true"
        }
        _ => false,
    }
}

/// Get partitionable GPUs that can be used with GPU-P
pub fn get_partitionable_gpus() -> Result<Vec<GpuInfo>> {
    // First try PowerShell for accurate partitioning info
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
            Get-VMPartitionableGpu | ForEach-Object {
                [PSCustomObject]@{
                    Name = $_.Name
                    ValidPartitionCounts = $_.ValidPartitionCounts -join ','
                }
            } | ConvertTo-Json -Compress
            "#,
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to query partitionable GPUs: {}", e)))?;

    if !output.status.success() {
        // Fallback to enumeration and filter
        let gpus = enumerate_gpus()?;
        return Ok(gpus.into_iter().filter(|g| g.supports_partitioning).collect());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    // Parse JSON and merge with enumerated GPU info
    let gpus = enumerate_gpus()?;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct PartitionableGpu {
        name: Option<String>,
        valid_partition_counts: Option<String>,
    }

    let partitionable: Vec<PartitionableGpu> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        serde_json::from_str::<PartitionableGpu>(trimmed)
            .map(|g| vec![g])
            .unwrap_or_default()
    };

    let partitionable_names: Vec<String> = partitionable
        .iter()
        .filter_map(|p| p.name.clone())
        .collect();

    Ok(gpus
        .into_iter()
        .filter(|g| {
            partitionable_names.iter().any(|name| {
                name.contains(&g.device_instance_id) || g.name.contains(name)
            })
        })
        .map(|mut g| {
            g.supports_partitioning = true;
            g
        })
        .collect())
}

/// Add a GPU partition adapter to a VM
pub fn add_gpu_partition_adapter(vm_name: &str, instance_path: Option<&str>) -> Result<()> {
    let mut cmd = format!("Add-VMGpuPartitionAdapter -VMName '{}'", vm_name);

    if let Some(path) = instance_path {
        cmd.push_str(&format!(" -InstancePath '{}'", path));
    }

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to add GPU partition adapter: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to add GPU partition adapter: {}",
            stderr
        )));
    }

    Ok(())
}

/// Remove a GPU partition adapter from a VM
pub fn remove_gpu_partition_adapter(vm_name: &str) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Remove-VMGpuPartitionAdapter -VMName '{}'", vm_name),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to remove GPU partition adapter: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to remove GPU partition adapter: {}",
            stderr
        )));
    }

    Ok(())
}

/// Get GPU partition adapters for a VM
pub fn get_gpu_partition_adapters(vm_name: &str) -> Result<Vec<GpuPartitionAdapter>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                Get-VMGpuPartitionAdapter -VMName '{}' | ForEach-Object {{
                    [PSCustomObject]@{{
                        VMName = $_.VMName
                        InstancePath = $_.InstancePath
                        MinPartitionVRAM = $_.MinPartitionVRAM
                        MaxPartitionVRAM = $_.MaxPartitionVRAM
                        OptimalPartitionVRAM = $_.OptimalPartitionVRAM
                        MinPartitionEncode = $_.MinPartitionEncode
                        MaxPartitionEncode = $_.MaxPartitionEncode
                        OptimalPartitionEncode = $_.OptimalPartitionEncode
                        MinPartitionDecode = $_.MinPartitionDecode
                        MaxPartitionDecode = $_.MaxPartitionDecode
                        OptimalPartitionDecode = $_.OptimalPartitionDecode
                        MinPartitionCompute = $_.MinPartitionCompute
                        MaxPartitionCompute = $_.MaxPartitionCompute
                        OptimalPartitionCompute = $_.OptimalPartitionCompute
                    }}
                }} | ConvertTo-Json -Compress
                "#,
                vm_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get GPU partition adapters: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to get GPU partition adapters: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct AdapterJson {
        #[serde(rename = "VMName")]
        vm_name: Option<String>,
        instance_path: Option<String>,
        #[serde(rename = "MinPartitionVRAM")]
        min_partition_vram: Option<u64>,
        #[serde(rename = "MaxPartitionVRAM")]
        max_partition_vram: Option<u64>,
        #[serde(rename = "OptimalPartitionVRAM")]
        optimal_partition_vram: Option<u64>,
        min_partition_encode: Option<u64>,
        max_partition_encode: Option<u64>,
        optimal_partition_encode: Option<u64>,
        min_partition_decode: Option<u64>,
        max_partition_decode: Option<u64>,
        optimal_partition_decode: Option<u64>,
        min_partition_compute: Option<u64>,
        max_partition_compute: Option<u64>,
        optimal_partition_compute: Option<u64>,
    }

    let adapters: Vec<AdapterJson> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed)
            .map_err(|e| HvError::JsonError(format!("Failed to parse adapters: {}", e)))?
    } else {
        let single: AdapterJson = serde_json::from_str(trimmed)
            .map_err(|e| HvError::JsonError(format!("Failed to parse adapter: {}", e)))?;
        vec![single]
    };

    Ok(adapters
        .into_iter()
        .map(|a| GpuPartitionAdapter {
            vm_name: a.vm_name.unwrap_or_default(),
            instance_path: a.instance_path,
            min_partition_vram: a.min_partition_vram,
            max_partition_vram: a.max_partition_vram,
            optimal_partition_vram: a.optimal_partition_vram,
            min_partition_encode: a.min_partition_encode,
            max_partition_encode: a.max_partition_encode,
            optimal_partition_encode: a.optimal_partition_encode,
            min_partition_decode: a.min_partition_decode,
            max_partition_decode: a.max_partition_decode,
            optimal_partition_decode: a.optimal_partition_decode,
            min_partition_compute: a.min_partition_compute,
            max_partition_compute: a.max_partition_compute,
            optimal_partition_compute: a.optimal_partition_compute,
        })
        .collect())
}

/// Set GPU partition adapter properties for a VM
pub fn set_gpu_partition_adapter(
    vm_name: &str,
    min_vram: Option<u64>,
    max_vram: Option<u64>,
    optimal_vram: Option<u64>,
    min_encode: Option<u64>,
    max_encode: Option<u64>,
    optimal_encode: Option<u64>,
    min_decode: Option<u64>,
    max_decode: Option<u64>,
    optimal_decode: Option<u64>,
    min_compute: Option<u64>,
    max_compute: Option<u64>,
    optimal_compute: Option<u64>,
) -> Result<()> {
    let mut cmd = format!("Set-VMGpuPartitionAdapter -VMName '{}'", vm_name);

    if let Some(v) = min_vram {
        cmd.push_str(&format!(" -MinPartitionVRAM {}", v));
    }
    if let Some(v) = max_vram {
        cmd.push_str(&format!(" -MaxPartitionVRAM {}", v));
    }
    if let Some(v) = optimal_vram {
        cmd.push_str(&format!(" -OptimalPartitionVRAM {}", v));
    }
    if let Some(v) = min_encode {
        cmd.push_str(&format!(" -MinPartitionEncode {}", v));
    }
    if let Some(v) = max_encode {
        cmd.push_str(&format!(" -MaxPartitionEncode {}", v));
    }
    if let Some(v) = optimal_encode {
        cmd.push_str(&format!(" -OptimalPartitionEncode {}", v));
    }
    if let Some(v) = min_decode {
        cmd.push_str(&format!(" -MinPartitionDecode {}", v));
    }
    if let Some(v) = max_decode {
        cmd.push_str(&format!(" -MaxPartitionDecode {}", v));
    }
    if let Some(v) = optimal_decode {
        cmd.push_str(&format!(" -OptimalPartitionDecode {}", v));
    }
    if let Some(v) = min_compute {
        cmd.push_str(&format!(" -MinPartitionCompute {}", v));
    }
    if let Some(v) = max_compute {
        cmd.push_str(&format!(" -MaxPartitionCompute {}", v));
    }
    if let Some(v) = optimal_compute {
        cmd.push_str(&format!(" -OptimalPartitionCompute {}", v));
    }

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to set GPU partition adapter: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to set GPU partition adapter: {}",
            stderr
        )));
    }

    Ok(())
}

/// Configure VM settings required for GPU-P
pub fn configure_vm_for_gpu(vm_name: &str, low_mmio_gb: u32, high_mmio_gb: u32) -> Result<()> {
    // Set GuestControlledCacheTypes and memory mapped IO space
    let cmd = format!(
        r#"
        Set-VM -Name '{}' -GuestControlledCacheTypes $true -LowMemoryMappedIoSpace {}GB -HighMemoryMappedIoSpace {}GB
        "#,
        vm_name, low_mmio_gb, high_mmio_gb
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to configure VM for GPU: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to configure VM for GPU: {}",
            stderr
        )));
    }

    Ok(())
}

/// Copy GPU driver files from host to VM (required for GPU-PV)
///
/// Note: This copies the driver files from the host to the VM's virtual disk.
/// The VM must be stopped and the VHD must be accessible.
pub fn copy_gpu_drivers_to_vm(
    vm_vhd_path: &str,
    host_driver_store: Option<&str>,
) -> Result<()> {
    let driver_store = host_driver_store.unwrap_or("C:\\Windows\\System32\\DriverStore\\FileRepository");

    // This is a simplified implementation - in practice you'd need to:
    // 1. Mount the VHD
    // 2. Copy driver files
    // 3. Update registry entries
    // 4. Unmount the VHD

    let cmd = format!(
        r#"
        # Get GPU driver folders
        $driverFolders = Get-ChildItem '{}' | Where-Object {{ $_.Name -like 'nv*' -or $_.Name -like 'amd*' -or $_.Name -like 'igdlh*' }}

        if ($driverFolders.Count -eq 0) {{
            Write-Error "No GPU drivers found in driver store"
            exit 1
        }}

        # Mount VHD
        $vhd = Mount-VHD -Path '{}' -Passthru
        $disk = $vhd | Get-Disk
        $partition = $disk | Get-Partition | Where-Object {{ $_.Type -eq 'Basic' }} | Select-Object -First 1
        $driveLetter = $partition.DriveLetter

        if (-not $driveLetter) {{
            $partition | Add-PartitionAccessPath -AssignDriveLetter
            $driveLetter = ($partition | Get-Partition).DriveLetter
        }}

        $destPath = "${{driveLetter}}:\Windows\System32\DriverStore\FileRepository"

        foreach ($folder in $driverFolders) {{
            $destFolder = Join-Path $destPath $folder.Name
            if (-not (Test-Path $destFolder)) {{
                Copy-Item -Path $folder.FullName -Destination $destFolder -Recurse -Force
            }}
        }}

        # Also copy System32 driver files
        $sys32Drivers = @('nvapi64.dll', 'nvd3dumx.dll', 'aticfx64.dll', 'igdumdim64.dll')
        foreach ($driver in $sys32Drivers) {{
            $src = Join-Path 'C:\Windows\System32' $driver
            if (Test-Path $src) {{
                $dest = "${{driveLetter}}:\Windows\System32\$driver"
                Copy-Item -Path $src -Destination $dest -Force -ErrorAction SilentlyContinue
            }}
        }}

        Dismount-VHD -Path '{}'
        Write-Output "GPU drivers copied successfully"
        "#,
        driver_store, vm_vhd_path, vm_vhd_path
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to copy GPU drivers: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to copy GPU drivers: {}",
            stderr
        )));
    }

    Ok(())
}

// =============================================================================
// DDA (Discrete Device Assignment) Support
// =============================================================================

/// Information about a device that can be assigned via DDA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignableDevice {
    /// Device instance path
    pub instance_id: String,
    /// Device friendly name
    pub name: String,
    /// Location path (required for DDA assignment)
    pub location_path: String,
    /// Whether the device is currently assigned to a VM
    pub is_assigned: bool,
    /// VM name if assigned
    pub assigned_vm: Option<String>,
    /// Whether the device is dismounted from host
    pub is_dismounted: bool,
    /// Device status
    pub status: String,
}

/// DDA assignment mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdaAssignmentMode {
    /// Direct assignment - device is exclusively assigned to VM
    Direct,
    /// Instance path assignment
    InstancePath,
    /// Location path assignment
    LocationPath,
}

/// Get all devices that can be assigned via DDA
///
/// Note: DDA requires Windows Server. This will return an empty list on client Windows.
pub fn get_assignable_devices() -> Result<Vec<AssignableDevice>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
            $devices = @()

            # Get host assignable devices
            $hostDevices = Get-VMHostAssignableDevice -ErrorAction SilentlyContinue

            if ($hostDevices) {
                foreach ($dev in $hostDevices) {
                    $devices += [PSCustomObject]@{
                        InstanceId = $dev.InstanceId
                        Name = $dev.Name
                        LocationPath = $dev.LocationPath
                        IsAssigned = $false
                        AssignedVM = $null
                        IsDismounted = $true
                        Status = 'Dismounted'
                    }
                }
            }

            # Get devices assigned to VMs
            $vms = Get-VM -ErrorAction SilentlyContinue
            foreach ($vm in $vms) {
                $vmDevices = Get-VMAssignableDevice -VM $vm -ErrorAction SilentlyContinue
                foreach ($dev in $vmDevices) {
                    $devices += [PSCustomObject]@{
                        InstanceId = $dev.InstanceId
                        Name = $dev.Name
                        LocationPath = $dev.LocationPath
                        IsAssigned = $true
                        AssignedVM = $vm.Name
                        IsDismounted = $true
                        Status = 'Assigned'
                    }
                }
            }

            $devices | ConvertTo-Json -Compress
            "#,
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get assignable devices: {}", e)))?;

    if !output.status.success() {
        // DDA not available (likely client Windows)
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct DeviceJson {
        instance_id: Option<String>,
        name: Option<String>,
        location_path: Option<String>,
        is_assigned: Option<bool>,
        #[serde(rename = "AssignedVM")]
        assigned_vm: Option<String>,
        is_dismounted: Option<bool>,
        status: Option<String>,
    }

    let devices: Vec<DeviceJson> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        serde_json::from_str::<DeviceJson>(trimmed)
            .map(|d| vec![d])
            .unwrap_or_default()
    };

    Ok(devices
        .into_iter()
        .map(|d| AssignableDevice {
            instance_id: d.instance_id.unwrap_or_default(),
            name: d.name.unwrap_or_default(),
            location_path: d.location_path.unwrap_or_default(),
            is_assigned: d.is_assigned.unwrap_or(false),
            assigned_vm: d.assigned_vm,
            is_dismounted: d.is_dismounted.unwrap_or(false),
            status: d.status.unwrap_or_default(),
        })
        .collect())
}

/// Get PCI location path for a device by instance ID
///
/// The location path is required for DDA assignment.
pub fn get_device_location_path(instance_id: &str) -> Result<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                $dev = Get-PnpDeviceProperty -InstanceId '{}' -KeyName 'DEVPKEY_Device_LocationPaths' -ErrorAction SilentlyContinue
                if ($dev) {{
                    $dev.Data | Where-Object {{ $_ -like 'PCIROOT*' }} | Select-Object -First 1
                }}
                "#,
                instance_id
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get location path: {}", e)))?;

    if !output.status.success() {
        return Err(HvError::OperationFailed(
            "Failed to get device location path".to_string(),
        ));
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err(HvError::OperationFailed(format!(
            "No PCI location path found for device: {}",
            instance_id
        )));
    }

    Ok(path)
}

/// Dismount a device from the host for DDA assignment
///
/// The device must be dismounted before it can be assigned to a VM.
/// This will disable the device on the host.
pub fn dismount_device_from_host(location_path: &str) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Dismount-VMHostAssignableDevice -LocationPath '{}' -Force",
                location_path
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to dismount device: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to dismount device from host: {}",
            stderr
        )));
    }

    Ok(())
}

/// Mount a device back to the host after DDA removal
///
/// This re-enables the device on the host after it was dismounted.
pub fn mount_device_to_host(location_path: &str) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Mount-VMHostAssignableDevice -LocationPath '{}'",
                location_path
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to mount device: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to mount device to host: {}",
            stderr
        )));
    }

    Ok(())
}

/// Assign a device to a VM via DDA
///
/// The device must be dismounted from the host first using `dismount_device_from_host`.
/// The VM must be stopped.
pub fn add_assignable_device_to_vm(vm_name: &str, location_path: &str) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Add-VMAssignableDevice -VMName '{}' -LocationPath '{}'",
                vm_name, location_path
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to add assignable device: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to add assignable device to VM: {}",
            stderr
        )));
    }

    Ok(())
}

/// Remove an assigned device from a VM
///
/// The VM must be stopped. After removal, you can mount the device back to host.
pub fn remove_assignable_device_from_vm(vm_name: &str, location_path: &str) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Remove-VMAssignableDevice -VMName '{}' -LocationPath '{}'",
                vm_name, location_path
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to remove assignable device: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to remove assignable device from VM: {}",
            stderr
        )));
    }

    Ok(())
}

/// Get devices assigned to a VM via DDA
pub fn get_vm_assignable_devices(vm_name: &str) -> Result<Vec<AssignableDevice>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                Get-VMAssignableDevice -VMName '{}' | ForEach-Object {{
                    [PSCustomObject]@{{
                        InstanceId = $_.InstanceId
                        Name = $_.Name
                        LocationPath = $_.LocationPath
                        IsAssigned = $true
                        AssignedVM = '{}'
                        IsDismounted = $true
                        Status = 'Assigned'
                    }}
                }} | ConvertTo-Json -Compress
                "#,
                vm_name, vm_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get VM assignable devices: {}", e)))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct DeviceJson {
        instance_id: Option<String>,
        name: Option<String>,
        location_path: Option<String>,
        is_assigned: Option<bool>,
        #[serde(rename = "AssignedVM")]
        assigned_vm: Option<String>,
        is_dismounted: Option<bool>,
        status: Option<String>,
    }

    let devices: Vec<DeviceJson> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        serde_json::from_str::<DeviceJson>(trimmed)
            .map(|d| vec![d])
            .unwrap_or_default()
    };

    Ok(devices
        .into_iter()
        .map(|d| AssignableDevice {
            instance_id: d.instance_id.unwrap_or_default(),
            name: d.name.unwrap_or_default(),
            location_path: d.location_path.unwrap_or_default(),
            is_assigned: d.is_assigned.unwrap_or(true),
            assigned_vm: d.assigned_vm,
            is_dismounted: d.is_dismounted.unwrap_or(true),
            status: d.status.unwrap_or_default(),
        })
        .collect())
}

/// Configure VM for DDA device assignment
///
/// Sets automatic stop action and enables write-combining on MMIO.
pub fn configure_vm_for_dda(
    vm_name: &str,
    automatic_stop_action: Option<&str>,
) -> Result<()> {
    let stop_action = automatic_stop_action.unwrap_or("TurnOff");

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                Set-VM -Name '{}' -AutomaticStopAction {}
                Set-VM -Name '{}' -GuestControlledCacheTypes $true
                "#,
                vm_name, stop_action, vm_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to configure VM for DDA: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to configure VM for DDA: {}",
            stderr
        )));
    }

    Ok(())
}

/// Set MMIO (Memory Mapped I/O) space for a VM
///
/// Required for DDA devices that need large MMIO space (like GPUs).
pub fn set_vm_mmio_space(
    vm_name: &str,
    low_mmio_mb: u64,
    high_mmio_gb: u64,
) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Set-VM -Name '{}' -LowMemoryMappedIoSpace {}MB -HighMemoryMappedIoSpace {}GB",
                vm_name, low_mmio_mb, high_mmio_gb
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to set MMIO space: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to set VM MMIO space: {}",
            stderr
        )));
    }

    Ok(())
}

/// Check if DDA is supported on this host
///
/// DDA requires:
/// - Windows Server 2016 or later
/// - Hardware IOMMU (Intel VT-d or AMD-Vi)
/// - Compatible motherboard with ACS support
pub fn check_dda_support() -> Result<DdaSupportInfo> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
            $info = @{
                IsServer = (Get-CimInstance Win32_OperatingSystem).ProductType -ne 1
                HasIommu = $false
                HostAssignableDevicesAvailable = $false
            }

            # Check for IOMMU
            $iommu = Get-WmiObject -Class Win32_Processor | Select-Object -First 1
            if ($iommu.VirtualizationFirmwareEnabled) {
                $info.HasIommu = $true
            }

            # Check if Get-VMHostAssignableDevice cmdlet exists
            if (Get-Command Get-VMHostAssignableDevice -ErrorAction SilentlyContinue) {
                $info.HostAssignableDevicesAvailable = $true
            }

            $info | ConvertTo-Json -Compress
            "#,
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to check DDA support: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct SupportJson {
        is_server: Option<bool>,
        has_iommu: Option<bool>,
        host_assignable_devices_available: Option<bool>,
    }

    let support: SupportJson = serde_json::from_str(trimmed).unwrap_or(SupportJson {
        is_server: Some(false),
        has_iommu: Some(false),
        host_assignable_devices_available: Some(false),
    });

    let is_server = support.is_server.unwrap_or(false);
    let has_iommu = support.has_iommu.unwrap_or(false);
    let cmdlet_available = support.host_assignable_devices_available.unwrap_or(false);

    Ok(DdaSupportInfo {
        is_supported: is_server && cmdlet_available,
        is_server,
        has_iommu,
        cmdlet_available,
        reason: if !is_server {
            Some("DDA requires Windows Server".to_string())
        } else if !cmdlet_available {
            Some("DDA cmdlets not available".to_string())
        } else {
            None
        },
    })
}

/// Information about DDA support on this host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdaSupportInfo {
    /// Whether DDA is fully supported
    pub is_supported: bool,
    /// Whether this is Windows Server
    pub is_server: bool,
    /// Whether IOMMU is detected
    pub has_iommu: bool,
    /// Whether DDA cmdlets are available
    pub cmdlet_available: bool,
    /// Reason if not supported
    pub reason: Option<String>,
}

/// Move a DDA device from one VM to another
///
/// This is a convenience function that:
/// 1. Removes the device from the source VM
/// 2. Assigns it to the target VM
///
/// Both VMs must be stopped.
pub fn move_assignable_device(
    source_vm: &str,
    target_vm: &str,
    location_path: &str,
) -> Result<()> {
    // Remove from source
    remove_assignable_device_from_vm(source_vm, location_path)?;

    // Add to target
    add_assignable_device_to_vm(target_vm, location_path)?;

    Ok(())
}
