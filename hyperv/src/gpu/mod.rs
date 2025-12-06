//! GPU management for Hyper-V VMs.
//!
//! This module provides support for GPU passthrough technologies:
//!
//! - **GPU-P (GPU Partitioning)**: Share a physical GPU among multiple VMs.
//!   Each VM gets a partition of GPU resources (VRAM, encode/decode engines).
//!   Requires a partitionable GPU (e.g., NVIDIA Tesla/Quadro with vGPU support).
//!
//! - **DDA (Discrete Device Assignment)**: Assign a complete PCI device to a VM
//!   for exclusive use. Provides near-native performance but requires dedicated
//!   hardware per VM.
//!
//! # GPU-P Example
//!
//! ```no_run
//! use windows_hyperv::{HyperV, gpu::{GpuPartitionManager, GpuPartitionSettings, VmGpuPartition}};
//!
//! fn main() -> windows_hyperv::Result<()> {
//!     let hyperv = HyperV::connect()?;
//!     let conn = hyperv.connection();
//!
//!     // List available GPUs
//!     let gpu_manager = GpuPartitionManager::new(&conn);
//!     for gpu in gpu_manager.list_partitionable_gpus()? {
//!         println!("{}: {} partitions available", gpu.name, gpu.available_partitions());
//!     }
//!
//!     // Assign a GPU partition to a VM
//!     let vm = hyperv.get_vm("MyVM")?;
//!     let vm_gpu = VmGpuPartition::new(&conn, vm.id());
//!
//!     let settings = GpuPartitionSettings::builder()
//!         .gpu_id("GPU-0000")
//!         .vram(512, 2048, 1024)  // min, max, optimal MB
//!         .build();
//!
//!     vm_gpu.add_partition(&settings)?;
//!     Ok(())
//! }
//! ```
//!
//! # DDA Example
//!
//! ```no_run
//! use windows_hyperv::{HyperV, gpu::{DdaManager, DdaDeviceSettings, VmDda}};
//!
//! fn main() -> windows_hyperv::Result<()> {
//!     let hyperv = HyperV::connect()?;
//!     let conn = hyperv.connection();
//!
//!     // List available devices
//!     let dda_manager = DdaManager::new(&conn);
//!     for device in dda_manager.list_available_devices()? {
//!         println!("{}: {}", device.friendly_name, device.location_path);
//!     }
//!
//!     // Assign a device to a VM
//!     let vm = hyperv.get_vm("MyVM")?;
//!     let vm_dda = VmDda::new(&conn, vm.id(), vm.name());
//!
//!     // First, configure MMIO space for the VM
//!     vm_dda.configure_mmio(256, 8192)?;  // Low and high MMIO gap in MB
//!
//!     // Then assign the device
//!     let settings = DdaDeviceSettings::new("PCIROOT(0)#PCI(0100)")
//!         .with_mmio(256 * 1024 * 1024, 8 * 1024 * 1024 * 1024);
//!     vm_dda.add_device(&settings)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Requirements
//!
//! ## GPU-P Requirements
//! - Windows Server 2019 or later / Windows 10 1903+
//! - Supported GPU (NVIDIA vGPU-capable, AMD MxGPU, Intel GVT-g)
//! - GPU driver with partitioning support
//!
//! ## DDA Requirements
//! - Windows Server 2016 or later
//! - IOMMU/VT-d enabled in BIOS
//! - PCI device compatible with DDA (check `Get-VMHostAssignableDevice`)
//! - Device dismounted from host before assignment

mod dda;
mod partition;
mod types;

pub use dda::{add_dda_device_to_vm, remove_dda_device_from_vm, DdaManager, VmDda};
pub use partition::{GpuPartitionManager, VmGpuPartition};
pub use types::{
    DdaDevice, DdaDeviceSettings, DdaDeviceStatus, GpuAssignmentType, GpuPartition,
    GpuPartitionSettings, GpuPartitionSettingsBuilder, GpuPartitionStatus, PartitionableGpu,
    VmGpuSummary,
};

#[cfg(windows)]
pub use partition::validate_gpu_partition_settings;
