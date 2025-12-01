//! Disk operations for Hyper-V VMs
//!
//! Provides DVD/ISO attachment, disk initialization, and VHD creation from ISO.

use crate::error::{HvError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

// =============================================================================
// DVD/ISO Operations
// =============================================================================

/// DVD drive information for a VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DvdDrive {
    /// VM name
    pub vm_name: String,
    /// Controller type (IDE or SCSI)
    pub controller_type: String,
    /// Controller number
    pub controller_number: u32,
    /// Controller location
    pub controller_location: u32,
    /// Path to mounted ISO (if any)
    pub path: Option<String>,
}

/// Get DVD drives for a VM
pub fn get_dvd_drives(vm_name: &str) -> Result<Vec<DvdDrive>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                Get-VMDvdDrive -VMName '{}' | ForEach-Object {{
                    [PSCustomObject]@{{
                        VMName = $_.VMName
                        ControllerType = $_.ControllerType.ToString()
                        ControllerNumber = $_.ControllerNumber
                        ControllerLocation = $_.ControllerLocation
                        Path = $_.Path
                    }}
                }} | ConvertTo-Json -Compress
                "#,
                vm_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get DVD drives: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to get DVD drives: {}",
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
    struct DriveJson {
        #[serde(rename = "VMName")]
        vm_name: Option<String>,
        controller_type: Option<String>,
        controller_number: Option<u32>,
        controller_location: Option<u32>,
        path: Option<String>,
    }

    let drives: Vec<DriveJson> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        serde_json::from_str::<DriveJson>(trimmed)
            .map(|d| vec![d])
            .unwrap_or_default()
    };

    Ok(drives
        .into_iter()
        .map(|d| DvdDrive {
            vm_name: d.vm_name.unwrap_or_default(),
            controller_type: d.controller_type.unwrap_or_else(|| "SCSI".to_string()),
            controller_number: d.controller_number.unwrap_or(0),
            controller_location: d.controller_location.unwrap_or(0),
            path: d.path,
        })
        .collect())
}

/// Add a DVD drive to a VM
pub fn add_dvd_drive(
    vm_name: &str,
    controller_number: Option<u32>,
    controller_location: Option<u32>,
) -> Result<()> {
    let mut cmd = format!("Add-VMDvdDrive -VMName '{}'", vm_name);

    if let Some(num) = controller_number {
        cmd.push_str(&format!(" -ControllerNumber {}", num));
    }
    if let Some(loc) = controller_location {
        cmd.push_str(&format!(" -ControllerLocation {}", loc));
    }

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to add DVD drive: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to add DVD drive: {}",
            stderr
        )));
    }

    Ok(())
}

/// Mount an ISO to a VM's DVD drive
pub fn mount_iso(
    vm_name: &str,
    iso_path: &str,
    controller_number: Option<u32>,
    controller_location: Option<u32>,
) -> Result<()> {
    let mut cmd = format!(
        "Set-VMDvdDrive -VMName '{}' -Path '{}'",
        vm_name, iso_path
    );

    if let Some(num) = controller_number {
        cmd.push_str(&format!(" -ControllerNumber {}", num));
    }
    if let Some(loc) = controller_location {
        cmd.push_str(&format!(" -ControllerLocation {}", loc));
    }

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to mount ISO: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to mount ISO: {}",
            stderr
        )));
    }

    Ok(())
}

/// Eject/unmount ISO from a VM's DVD drive
pub fn eject_iso(
    vm_name: &str,
    controller_number: Option<u32>,
    controller_location: Option<u32>,
) -> Result<()> {
    let mut cmd = format!("Set-VMDvdDrive -VMName '{}' -Path $null", vm_name);

    if let Some(num) = controller_number {
        cmd.push_str(&format!(" -ControllerNumber {}", num));
    }
    if let Some(loc) = controller_location {
        cmd.push_str(&format!(" -ControllerLocation {}", loc));
    }

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to eject ISO: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to eject ISO: {}",
            stderr
        )));
    }

    Ok(())
}

/// Remove a DVD drive from a VM
pub fn remove_dvd_drive(
    vm_name: &str,
    controller_number: u32,
    controller_location: u32,
) -> Result<()> {
    let cmd = format!(
        "Remove-VMDvdDrive -VMName '{}' -ControllerNumber {} -ControllerLocation {}",
        vm_name, controller_number, controller_location
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to remove DVD drive: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to remove DVD drive: {}",
            stderr
        )));
    }

    Ok(())
}

/// Set boot order for a VM (Gen2 only)
pub fn set_boot_order(vm_name: &str, boot_devices: &[&str]) -> Result<()> {
    // Build PowerShell command to set boot order
    // boot_devices can be: "VHD", "DVD", "Network", "File"
    let devices_str = boot_devices
        .iter()
        .map(|d| {
            match *d {
                "DVD" | "CD" => "$(Get-VMDvdDrive -VMName '".to_string() + vm_name + "' | Select-Object -First 1)",
                "VHD" | "HardDrive" => "$(Get-VMHardDiskDrive -VMName '".to_string() + vm_name + "' | Select-Object -First 1)",
                "Network" => "$(Get-VMNetworkAdapter -VMName '".to_string() + vm_name + "' | Select-Object -First 1)",
                _ => d.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let cmd = format!(
        "Set-VMFirmware -VMName '{}' -BootOrder {}",
        vm_name, devices_str
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to set boot order: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to set boot order: {}",
            stderr
        )));
    }

    Ok(())
}

// =============================================================================
// Disk Initialization
// =============================================================================

/// Partition style for disk initialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionStyle {
    /// MBR partition table (legacy, max 2TB)
    Mbr,
    /// GPT partition table (modern, required for UEFI)
    Gpt,
}

impl PartitionStyle {
    fn as_str(&self) -> &str {
        match self {
            PartitionStyle::Mbr => "MBR",
            PartitionStyle::Gpt => "GPT",
        }
    }
}

/// File system type for formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystem {
    Ntfs,
    ReFs,
    Fat32,
    ExFat,
}

impl FileSystem {
    fn as_str(&self) -> &str {
        match self {
            FileSystem::Ntfs => "NTFS",
            FileSystem::ReFs => "ReFS",
            FileSystem::Fat32 => "FAT32",
            FileSystem::ExFat => "exFAT",
        }
    }
}

/// Initialize a VHD with partitions (mounts, initializes, partitions, formats)
///
/// This is useful for preparing a blank VHD to receive an OS installation
/// or to use as a data disk.
pub fn initialize_vhd(
    vhd_path: &str,
    partition_style: PartitionStyle,
    file_system: FileSystem,
    label: Option<&str>,
) -> Result<String> {
    let label_str = label.unwrap_or("Data");

    let cmd = format!(
        r#"
        # Mount the VHD
        $vhd = Mount-VHD -Path '{}' -Passthru
        $diskNumber = $vhd.DiskNumber

        # Initialize the disk
        Initialize-Disk -Number $diskNumber -PartitionStyle {}

        # Create partition using all available space
        $partition = New-Partition -DiskNumber $diskNumber -UseMaximumSize -AssignDriveLetter

        # Format the partition
        Format-Volume -Partition $partition -FileSystem {} -NewFileSystemLabel '{}' -Confirm:$false

        # Get the drive letter
        $driveLetter = $partition.DriveLetter

        # Output the drive letter
        Write-Output $driveLetter
        "#,
        vhd_path,
        partition_style.as_str(),
        file_system.as_str(),
        label_str
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to initialize VHD: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to initialize VHD: {}",
            stderr
        )));
    }

    let drive_letter = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(drive_letter)
}

/// Initialize a VHD with Windows boot partitions (EFI System, MSR, Windows)
///
/// Creates the standard partition layout for a Windows boot disk:
/// - EFI System Partition (ESP): 100MB, FAT32
/// - Microsoft Reserved (MSR): 16MB
/// - Windows: Remaining space, NTFS
pub fn initialize_windows_vhd(vhd_path: &str, windows_label: Option<&str>) -> Result<String> {
    let label = windows_label.unwrap_or("Windows");

    let cmd = format!(
        r#"
        # Mount the VHD
        $vhd = Mount-VHD -Path '{}' -Passthru
        $diskNumber = $vhd.DiskNumber

        # Initialize as GPT
        Initialize-Disk -Number $diskNumber -PartitionStyle GPT

        # Create EFI System Partition (100MB)
        $efiPartition = New-Partition -DiskNumber $diskNumber -Size 100MB -GptType '{{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}}'
        Format-Volume -Partition $efiPartition -FileSystem FAT32 -NewFileSystemLabel 'System' -Confirm:$false

        # Create Microsoft Reserved Partition (16MB)
        New-Partition -DiskNumber $diskNumber -Size 16MB -GptType '{{e3c9e316-0b5c-4db8-817d-f92df00215ae}}' | Out-Null

        # Create Windows partition (remaining space)
        $winPartition = New-Partition -DiskNumber $diskNumber -UseMaximumSize -AssignDriveLetter
        Format-Volume -Partition $winPartition -FileSystem NTFS -NewFileSystemLabel '{}' -Confirm:$false

        # Output the Windows drive letter
        Write-Output $winPartition.DriveLetter
        "#,
        vhd_path, label
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to initialize Windows VHD: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to initialize Windows VHD: {}",
            stderr
        )));
    }

    let drive_letter = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(drive_letter)
}

/// Dismount a VHD that was mounted for initialization
pub fn dismount_vhd(vhd_path: &str) -> Result<()> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Dismount-VHD -Path '{}'", vhd_path),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to dismount VHD: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to dismount VHD: {}",
            stderr
        )));
    }

    Ok(())
}

// =============================================================================
// VHD Creation from ISO (Windows Image)
// =============================================================================

/// Windows edition for image extraction
#[derive(Debug, Clone)]
pub struct WindowsEdition {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub size_bytes: u64,
}

/// Get available Windows editions from an ISO
pub fn get_windows_editions(iso_path: &str) -> Result<Vec<WindowsEdition>> {
    let cmd = format!(
        r#"
        # Mount ISO
        $iso = Mount-DiskImage -ImagePath '{}' -PassThru
        $driveLetter = ($iso | Get-Volume).DriveLetter + ':'
        $wimPath = Join-Path $driveLetter 'sources\install.wim'

        if (-not (Test-Path $wimPath)) {{
            $wimPath = Join-Path $driveLetter 'sources\install.esd'
        }}

        if (-not (Test-Path $wimPath)) {{
            Dismount-DiskImage -ImagePath '{}'
            Write-Error "No install.wim or install.esd found in ISO"
            exit 1
        }}

        # Get image info
        $images = Get-WindowsImage -ImagePath $wimPath

        $result = @()
        foreach ($img in $images) {{
            $result += [PSCustomObject]@{{
                Index = $img.ImageIndex
                Name = $img.ImageName
                Description = $img.ImageDescription
                SizeBytes = $img.ImageSize
            }}
        }}

        # Dismount ISO
        Dismount-DiskImage -ImagePath '{}'

        $result | ConvertTo-Json -Compress
        "#,
        iso_path, iso_path, iso_path
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get Windows editions: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to get Windows editions: {}",
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
    struct EditionJson {
        index: Option<u32>,
        name: Option<String>,
        description: Option<String>,
        size_bytes: Option<u64>,
    }

    let editions: Vec<EditionJson> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        serde_json::from_str::<EditionJson>(trimmed)
            .map(|e| vec![e])
            .unwrap_or_default()
    };

    Ok(editions
        .into_iter()
        .map(|e| WindowsEdition {
            index: e.index.unwrap_or(1),
            name: e.name.unwrap_or_default(),
            description: e.description.unwrap_or_default(),
            size_bytes: e.size_bytes.unwrap_or(0),
        })
        .collect())
}

/// Create a bootable Windows VHDX from an ISO
///
/// This creates a new VHDX, initializes it with Windows boot partitions,
/// and applies a Windows image from the ISO.
///
/// # Arguments
/// * `iso_path` - Path to Windows ISO file
/// * `vhdx_path` - Path for the new VHDX file
/// * `size_gb` - Size of the VHDX in GB
/// * `edition_index` - Index of Windows edition (use `get_windows_editions` to list)
pub fn create_vhdx_from_iso(
    iso_path: &str,
    vhdx_path: &str,
    size_gb: u64,
    edition_index: u32,
) -> Result<()> {
    let size_bytes = size_gb * 1024 * 1024 * 1024;

    let cmd = format!(
        r#"
        $ErrorActionPreference = 'Stop'

        # Create VHDX
        $vhdx = New-VHD -Path '{}' -SizeBytes {} -Dynamic

        # Mount VHDX
        $vhd = Mount-VHD -Path '{}' -Passthru
        $diskNumber = $vhd.DiskNumber

        # Initialize as GPT
        Initialize-Disk -Number $diskNumber -PartitionStyle GPT

        # Create EFI System Partition
        $efiPartition = New-Partition -DiskNumber $diskNumber -Size 100MB -GptType '{{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}}'
        Format-Volume -Partition $efiPartition -FileSystem FAT32 -NewFileSystemLabel 'System' -Confirm:$false
        $efiPartition | Add-PartitionAccessPath -AssignDriveLetter
        $efiLetter = $efiPartition.DriveLetter

        # Create MSR Partition
        New-Partition -DiskNumber $diskNumber -Size 16MB -GptType '{{e3c9e316-0b5c-4db8-817d-f92df00215ae}}' | Out-Null

        # Create Windows Partition
        $winPartition = New-Partition -DiskNumber $diskNumber -UseMaximumSize
        Format-Volume -Partition $winPartition -FileSystem NTFS -NewFileSystemLabel 'Windows' -Confirm:$false
        $winPartition | Add-PartitionAccessPath -AssignDriveLetter
        $winLetter = $winPartition.DriveLetter

        # Mount ISO
        $iso = Mount-DiskImage -ImagePath '{}' -PassThru
        $isoLetter = ($iso | Get-Volume).DriveLetter + ':'

        # Find WIM file
        $wimPath = Join-Path $isoLetter 'sources\install.wim'
        if (-not (Test-Path $wimPath)) {{
            $wimPath = Join-Path $isoLetter 'sources\install.esd'
        }}

        # Apply Windows image
        Write-Host "Applying Windows image (this may take several minutes)..."
        Expand-WindowsImage -ImagePath $wimPath -Index {} -ApplyPath "${{winLetter}}:\"

        # Configure boot files
        Write-Host "Configuring boot files..."
        bcdboot "${{winLetter}}:\Windows" /s "${{efiLetter}}:" /f UEFI

        # Cleanup
        Dismount-DiskImage -ImagePath '{}'
        Dismount-VHD -Path '{}'

        Write-Host "VHDX created successfully: {}"
        "#,
        vhdx_path, size_bytes, vhdx_path,
        iso_path, edition_index,
        iso_path, vhdx_path, vhdx_path
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to create VHDX from ISO: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to create VHDX from ISO: {}",
            stderr
        )));
    }

    Ok(())
}

/// Quick create a VM with Windows from ISO
///
/// This is a convenience function that:
/// 1. Creates a bootable VHDX from the ISO
/// 2. Creates a Gen2 VM with the VHDX attached
pub fn quick_create_windows_vm(
    vm_name: &str,
    iso_path: &str,
    vhdx_path: &str,
    size_gb: u64,
    memory_mb: u64,
    cpu_count: u32,
    edition_index: u32,
) -> Result<()> {
    // Create the VHDX from ISO
    create_vhdx_from_iso(iso_path, vhdx_path, size_gb, edition_index)?;

    // Create the VM
    let cmd = format!(
        r#"
        New-VM -Name '{}' -Generation 2 -MemoryStartupBytes {}MB -VHDPath '{}'
        Set-VM -Name '{}' -ProcessorCount {}
        "#,
        vm_name, memory_mb, vhdx_path,
        vm_name, cpu_count
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to create VM: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to create VM: {}",
            stderr
        )));
    }

    Ok(())
}

// =============================================================================
// Hard Disk Drive Operations
// =============================================================================

/// Hard disk drive info for a VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardDiskDrive {
    /// VM name
    pub vm_name: String,
    /// Controller type (IDE or SCSI)
    pub controller_type: String,
    /// Controller number
    pub controller_number: u32,
    /// Controller location
    pub controller_location: u32,
    /// Path to VHD/VHDX file
    pub path: Option<String>,
}

/// Get hard disk drives for a VM
pub fn get_hard_disk_drives(vm_name: &str) -> Result<Vec<HardDiskDrive>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                Get-VMHardDiskDrive -VMName '{}' | ForEach-Object {{
                    [PSCustomObject]@{{
                        VMName = $_.VMName
                        ControllerType = $_.ControllerType.ToString()
                        ControllerNumber = $_.ControllerNumber
                        ControllerLocation = $_.ControllerLocation
                        Path = $_.Path
                    }}
                }} | ConvertTo-Json -Compress
                "#,
                vm_name
            ),
        ])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to get hard disk drives: {}", e)))?;

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
    struct DriveJson {
        #[serde(rename = "VMName")]
        vm_name: Option<String>,
        controller_type: Option<String>,
        controller_number: Option<u32>,
        controller_location: Option<u32>,
        path: Option<String>,
    }

    let drives: Vec<DriveJson> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        serde_json::from_str::<DriveJson>(trimmed)
            .map(|d| vec![d])
            .unwrap_or_default()
    };

    Ok(drives
        .into_iter()
        .map(|d| HardDiskDrive {
            vm_name: d.vm_name.unwrap_or_default(),
            controller_type: d.controller_type.unwrap_or_else(|| "SCSI".to_string()),
            controller_number: d.controller_number.unwrap_or(0),
            controller_location: d.controller_location.unwrap_or(0),
            path: d.path,
        })
        .collect())
}

/// Add a hard disk drive to a VM
pub fn add_hard_disk_drive(
    vm_name: &str,
    vhd_path: &str,
    controller_type: Option<&str>,
    controller_number: Option<u32>,
    controller_location: Option<u32>,
) -> Result<()> {
    let mut cmd = format!(
        "Add-VMHardDiskDrive -VMName '{}' -Path '{}'",
        vm_name, vhd_path
    );

    if let Some(ct) = controller_type {
        cmd.push_str(&format!(" -ControllerType {}", ct));
    }
    if let Some(num) = controller_number {
        cmd.push_str(&format!(" -ControllerNumber {}", num));
    }
    if let Some(loc) = controller_location {
        cmd.push_str(&format!(" -ControllerLocation {}", loc));
    }

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to add hard disk drive: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to add hard disk drive: {}",
            stderr
        )));
    }

    Ok(())
}

/// Remove a hard disk drive from a VM
pub fn remove_hard_disk_drive(
    vm_name: &str,
    controller_number: u32,
    controller_location: u32,
) -> Result<()> {
    let cmd = format!(
        "Remove-VMHardDiskDrive -VMName '{}' -ControllerType SCSI -ControllerNumber {} -ControllerLocation {}",
        vm_name, controller_number, controller_location
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .map_err(|e| HvError::OperationFailed(format!("Failed to remove hard disk drive: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HvError::OperationFailed(format!(
            "Failed to remove hard disk drive: {}",
            stderr
        )));
    }

    Ok(())
}
