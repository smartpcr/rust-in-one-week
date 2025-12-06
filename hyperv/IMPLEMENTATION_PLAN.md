# Hyper-V Implementation Plan

## Overview

This document provides a detailed implementation plan for extending the `windows-hyperv` crate to achieve feature parity with the C++ `wmiv2` library. The plan covers:

1. **Gap Analysis** - Comprehensive comparison with C++ implementation
2. **Complete API Operation Listings** - All operations by category
3. **Schema Definitions** - WMI class mappings and Rust types
4. **API Method Signatures** - Function signatures with validation
5. **Module Organization** - File structure and dependencies
6. **Validation Rules** - Input validation and error handling
7. **Priority and Dependencies** - Implementation order

---

## Gap Analysis

### Source Reference

- **C++ Implementation**: `/mnt/e/work/hub/agent/rd-agent/src/OS/hyper-v/wvcdll/lib/wmiv2`
- **Rust Implementation**: `hyperv` crate in this repository

### Coverage Summary

| Category | C++ Operations | Rust Coverage | Gap |
|----------|---------------|---------------|-----|
| VM Lifecycle | 15+ | ~40% | Core operations missing state management |
| Migration | 12+ | 0% | **Complete gap** |
| Export/Import | 10+ | 0% | **Complete gap** |
| Security (TPM/SecureBoot) | 8+ | 0% | **Complete gap** |
| Processor (Advanced) | 25+ | ~10% | Only basic CPU count |
| Memory (Advanced) | 10+ | ~20% | Basic memory only |
| NUMA/Topology | 12+ | 0% | **Complete gap** |
| Storage (SCSI/IDE) | 20+ | ~30% | Basic VHD operations |
| NVMe Direct | 15+ | 0% | **Complete gap** |
| Network | 15+ | ~40% | Switch/adapter basics |
| GPU (DDA/GPU-P) | 10+ | 0% | **Complete gap** |
| KVP Exchange | 8+ | 0% | **Complete gap** |
| Serial Console | 4+ | 0% | **Complete gap** |
| Thumbnail/Display | 3+ | 0% | **Complete gap** |
| Validation/Capabilities | 15+ | 0% | **Complete gap** |
| Remote Connection | 5+ | 0% | **Complete gap** |
| Job Handling | 8+ | ~20% | Basic polling only |

**Overall Coverage: ~15-20%**

---

## Complete API Operation Listings by Category

### 1. VM Lifecycle Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
CreateVm(VmName, MemorySize, NumberProcessors, LowMmioGapSize, VmSettings, ...)
GetVm(VmName, VmConfiguration, ResourceReport, AlwaysRefreshCache)
GetVmFromId(VmId, VmConfiguration)
GetPlannedVm(VmName, VmConfiguration, ResourceReport, TimeoutInMillis)
Delete(ErrorString, ErrorStringLength, Timeout)
DeletePlannedVm(ErrorString, ErrorStringLength, Timeout)
Start(Timeout, ErrorString, SerialPortStartRequired)
Stop(Timeout, ErrorString, SerialPortStopRequired)
SetState(State, Timeout, ErrorString, SerialPortStartRequired, SerialPortStopRequired, QueryWithVmId)
GetVmState(State)
GetPlannedVmState(State)
GetVmOperationalStatus(PrimaryStatus, SecondaryStatus, State, QueryWithVmId)
GetVmHeartbeatStatus(Value)
GetVmHeartbeatStatusEx(Value)
EnableHeartbeat(ErrorString, ErrorStringLength)
DisableHeartbeat(ErrorString, ErrorStringLength)
CleanShutdown(ReasonString, ErrorString, ErrorStringLength)
Hibernate(Timeout, ErrorString, ErrorStringLength)
UpdateName(VmName, ErrorString, ErrorStringLength)
GetName(Name)
GetId(Id)
IsFirstBoot(Result, ErrorString, ErrorStringLength)
```

#### Rust Current Coverage
```rust
// hyperv crate - vm module
create_vm(settings: &VmSettings) -> Result<VirtualMachine>
list_vms() -> Result<Vec<VirtualMachine>>
get_vm(name: &str) -> Result<VirtualMachine>
vm.start() -> Result<()>
vm.stop() -> Result<()>
vm.state() -> VmState
vm.delete() -> Result<()>
```

#### Gap: Missing Operations
- `GetVmFromId` - Lookup by GUID
- `GetPlannedVm` - For migration scenarios
- `SetState` - Generic state transition with options
- `GetVmOperationalStatus` - Detailed operational status
- `GetVmHeartbeatStatus/Ex` - Guest heartbeat monitoring
- `EnableHeartbeat/DisableHeartbeat` - IC service control
- `CleanShutdown` - Graceful shutdown via guest
- `Hibernate` - S4 power state
- `IsFirstBoot` - First boot detection

---

### 2. Migration Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
MigrateVm(DestinationHost, VmName, VhdsForStorageMigration, OverWriteExistingVhds,
          AvoidRemovingVhds, CancelIfBlackoutThresholdExceeded, EnableCompression,
          SkipResourceDiskMigration, CpuCappingMagnitude, DestinationPath,
          DestinationVmName, ErrorString, ErrorStringLength, EnableSMBTransport)
MigrateVmToSuspended(DestinationHost, VmName, VhdsForStorageMigration, ...)
MigrateVmAsync(DestinationHost, VmName, JobId, JobIdLength, VhdsForStorageMigration,
               OverWriteExistingVhds, AvoidRemovingVhds, CancelIfBlackoutThresholdExceeded,
               EnableCompression, SkipResourceDiskMigration, LmForNvmeBasedVhds,
               CpuCappingMagnitude, DestinationVmName, DestinationPath, VmalMode,
               ErrorString, ErrorStringLength, EnableSMBTransport, IncludeVMGSFile,
               IsAdvancedOptionEnabled, AdvancedOptions, DestinationVmJbodDiskDetails)
MigrateVmToSuspendedAsync(...)  // Same parameters as MigrateVmAsync
GetMigrationStatus(JobId, JobResult, JobPercentCompleted, JobState, JobElapsedTime,
                   JobStatus, JobStatusLength, ErrorString, ErrorStringLength)
CancelMigration(JobId, ErrorString, ErrorStringLength)
GetOngoingMigrationJobId(VmId, DestinationHost, JobId, JobIdLength, JobTimestamp)
SetMigrationOperationCallback(QueryString, Callback, Operation)
IsLiveMigrationSupported()
StartMigrateSuspendedVm(ErrorString, SerialPortStartRequired, QueryWithVmId,
                        isJobInvoked, SecondaryStatusCheckEnabled)
StopMigrateSuspendedVm(ErrorString, ErrorStringLength)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

#### Required Rust Implementation
```rust
// migration/mod.rs
pub struct MigrationService { ... }
pub struct MigrationSettings { ... }
pub struct MigrationJob { ... }

impl MigrationService {
    fn migrate_vm(&self, settings: &MigrationSettings) -> Result<()>;
    fn migrate_vm_async(&self, settings: &MigrationSettings) -> Result<MigrationJob>;
    fn migrate_vm_to_suspended(&self, settings: &MigrationSettings) -> Result<()>;
    fn get_migration_status(&self, job_id: &str) -> Result<MigrationStatus>;
    fn cancel_migration(&self, job_id: &str) -> Result<()>;
    fn get_ongoing_migration_job(&self, vm_id: &str, dest_host: &str) -> Result<Option<String>>;
    fn is_live_migration_supported(&self) -> Result<bool>;
}
```

---

### 3. Export/Import Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
Export(ExportRootDirectory, ErrorString, ErrorStringLength)
ExportConfig(ExportDirectory, ErrorString, ErrorStringLength)
ExportForLiveMigration(ExportDirectory, ErrorString, ErrorStringLength)
ExportWithoutRuntimeInfo(ExportDirectory, ErrorString, ErrorStringLength)
ExportWithRuntimeInfo(ExportRootDirectory, ErrorString, ErrorStringLength)
Import(VmName, ImportRootDirectory, ErrorString, ErrorStringLength)
CreatePlannedVm(SourceVmName, NewVmName, ConfigPath, VmFolderPath, RetainVmId,
                RdSsdToABC, ErrorString, ErrorStringLength, SerialPortStartRequired,
                TrustedVmPropertyUpdateToRandomValue, LmForNvmeBasedVhds,
                DisableNetworkOffloads, DisableNetworkOffloadsForAccelnetVm,
                FastAttachDetachDiskWithContainerId, IsMfndVM, VmalMode,
                MfndControllerCount, MfndControllersMap, StopMigrateSession,
                IsJbodVm, JbodDiskCount, JbodDiskMap)
ImportAndRealizeSavedVm(SourceVmName, NewVmName, SystemDefinitionFile,
                        OldVhdToNewVhdNameMap, ErrorString, ErrorStringLength, Timeout)
CustomRestore(VmrsFilepath, ErrorString, ErrorStringLength)
RenamePlannedScsiVmVhds(OldVhdToNewVhdNameMap, ErrorString, ErrorStringLength, InLMContext)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 4. Security Operations (TPM, SecureBoot, Isolation)

#### C++ Operations (WindowsVirtualComputer.h)
```
SetVmSecurityProperty(PropertyName, Value, ErrorString, ErrorStringLength)
GetVmSecuritySettings(SecureBootEnabled, VtpmEnabled, GuestIsolationType)
GetSecuritySettings(SecuritySettingData)
GetCvmFirmwareType(CvmFirmwareType)
```

#### C++ Security Types
```cpp
typedef enum _GUEST_STATE_ISOLATION_TYPE {
    IsolationTypeNone = 0,
    IsolationTypeVbs = 1,           // Virtualization-based security
    IsolationTypeSnp = 2,           // AMD SEV-SNP
    IsolationTypeTdx = 3            // Intel TDX
} GUEST_STATE_ISOLATION_TYPE;

typedef enum _CVM_FIRMWARE_TYPE {
    CvmFirmwareTypeUnknown = 0,
    CvmFirmwareTypeLegacy = 1,
    CvmFirmwareTypeUefi = 2
} CVM_FIRMWARE_TYPE;
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 5. Processor Operations (Advanced)

#### C++ Operations (WindowsVirtualComputer.h)
```
UpdateProcessorDevice(Processor, ErrorString, ErrorStringLength)
SetExposeVirtualizationExtensions(Enabled, ErrorString, ErrorStringLength)
SetHierarchicalVirtualization(Enabled, ErrorString, ErrorStringLength)
SetMaxHardwareIsolatedGuests(MaxIsolatedGuests, ErrorString, ErrorStringLength)
SetMaxHierarchicalPartitions(MaxHierarchicalPartitions, HierarchicalVersion, ...)
SetMaxHierarchicalVps(MaxHierarchicalVps, HierarchicalVersion, ...)
SetExtendedVirtualizationExtensions(VirtualizationExtensionValue, ...)
SetHWThreadsPerCore(ThreadCount, ErrorString, ErrorStringLength)
SetL3CacheWays(L3CacheWays, ErrorString, ErrorStringLength)
GetNumaDefault(MaxNumaNodesPerSocketOutput, MaxProcessorsPerNumaNodeOutput)
SetNuma(MaxNumaNodePerSocket, MaxNumaProcPerNode, ErrorString, ErrorStringLength)
SetNumaMemory(MaxMemoryBlocksPerNumaNode, ErrorString, ErrorStringLength)
GetNumaMemoryMinimumConsumable(MinimumConsumableBlocksOutput)
EnableOptimalVmNumaTopology(ErrorString, ErrorStringLength)
GetVmNumaTopology(VmNumaTopology)
SetVmNumaTopology(TopologyMapping, ErrorString, ErrorStringLength)
GetVirtualProcessorsPerChannel(Vppc)
SetVirtualProcessorsPerChannel(Vppc, ErrorString, ErrorStringLength)
GetProcessorSettingData(Settings)  // Map of setting name to value
SetProcessorSettingData(Settings, ErrorString, ErrorStringLength)
  // Settings include: MaxProcessorCountPerL3, MaxClusterCountPerSocket, L3ProcessorDistributionPolicy
SetCpuGroupId(CpuGroupId, ErrorString, ErrorStringLength)
GetCpuGroupId(CpuGroupId, ErrorString, ErrorStringLength)
SupportsCpuGroupIdProperty(Result, ErrorString, ErrorStringLength)
SetVmCpuFrequency(VmCpuFrequency, ErrorString, ErrorStringLength)
SetProcessorLimit(Limit, ErrorString, ErrorStringLength)
SupportsProcessorSettingProperty(PropertyName, Result, ErrorString, ErrorStringLength)
SupportsHWThreadsPerCoreProperty(Result, ErrorString, ErrorStringLength)
SupportsL3CacheWaysProperty(Result, ErrorString, ErrorStringLength)
SupportsExposeVirtualizationExtensionsProperty(Result, ErrorString, ErrorStringLength)
SupportsHierarchicalVirtualizationProperty(Result, ErrorString, ErrorStringLength)
IsHierarchicalVirtualizationEnabled(Result, ErrorString, ErrorStringLength)
SetProcessorPageShatteringValue(Value, ErrorString, ErrorStringLength)
IsProcessorPageShatteringEnabled(Result, ErrorString, ErrorStringLength)
SetEnableSocketTopology(Value, ErrorString, ErrorStringLength)
IsEnableSocketTopologySupported(IsSupported, ErrorString, ErrorStringLength)
SetProcessorFeatures(ProcessorFeatureSetJson, SyntheticProcessorFeatureSetJson, ...)
ExtractVmFeatures(VmSettings, VmVersion, GuestFeatureSet, ProcessorFeatureSetJson, ...)
SetVmFeatures(VmSettings, VmVersionStr)
```

#### Rust Current Coverage
```rust
// vm/types.rs
pub struct ProcessorCount(u32);  // Basic validation 1-2048

// VmSettings builder
.processor_count(count: u32)
```

#### Gap: Missing Advanced Processor Features
- CPU group assignment
- CPU limits/reservations/weights
- NUMA topology configuration
- HWThreadsPerCore (SMT)
- L3 cache ways
- CCX/CCD topology (AMD)
- Nested virtualization
- Processor feature compatibility
- Page shattering mitigation
- Socket topology

---

### 6. Memory Operations (Advanced)

#### C++ Operations (WindowsVirtualComputer.h)
```
UpdateMemory(MemorySize, VmSettings, EnableSgx, SgxSettings, ErrorString, ErrorStringLength)
SetMemoryEncryptionPolicy(MemoryEncryptionPolicy, ErrorString, ErrorStringLength)
UpdateMemorySettingsForHugePageSupport(ErrorString, ErrorStringLength)
UpdateSettingsForHugePageSupport(MemorySize, ErrorString, ErrorStringLength)
AlignMmioSettingsForHugePageSupport(ErrorString, ErrorStringLength)
SetHighMmioGapSize(SizeInMB, ErrorString, ErrorStringLength)
SetHighMmioGapBase(GapBaseInMB, ErrorString, ErrorStringLength)
EnableGuestControlledCacheTypes(Enabled, ErrorString, ErrorStringLength)
SetVmHibernateSetting(ErrorString, ErrorStringLength)
```

#### Memory Encryption Policy Values
```cpp
// MemoryEncryptionPolicy values
MKTME_DISABLED = 0
MKTME_ENABLE_IF_SUPPORTED = 1
MKTME_ALWAYS_ENABLED = 2
```

#### Rust Current Coverage
```rust
// vm/types.rs
pub struct MemoryMB(u64);  // Basic validation

// VmSettings builder
.memory_mb(size: u64)
```

---

### 7. Storage Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
AddSyntheticStorageController(ControllerNumber, GuidStr, StorageController, ErrorString, ...)
AddStorageDevice(DeviceType, ControllerType, ControllerNumber, DeviceNumber,
                 PathOrPhysicalObjectPath, ErrorString, IgnoreAlreadyAttachedCheck,
                 VirtualDiskType, VirtualDiskId, IsCacheEnabled)
AddStorageDevices(Disks, DisksSettingData, Retriable, ErrorString, ErrorStringLength)
AddStorageDevicesWithRetries(Disks, ErrorString, ErrorStringLength)
AddPhysicalStorageDevice(DeviceType, ControllerType, ControllerNumber, DeviceNumber, DiskNumber, ...)
RemovePhysicalStorageDevice(DiskNumber, RemoveParentDrive, ErrorString, ErrorStringLength)
RetrievePhysicalDiskAttachStatus(DiskNumber, isAttached)
RemoveVhd(Path, RemoveParentDrive, ErrorString, ErrorStringLength)
RemoveVhdLegacy(Path, RemoveParentDrive, ErrorString, ErrorStringLength)
RemoveVhdImproved(Path, RemoveParentDrive, ErrorString, ErrorStringLength)
RemoveDataVhdParallel(Paths, ErrorString, ErrorStringLength)
RemoveDvdDrive(ControllerType, ControllerNumber, DeviceNumber, ErrorString, ErrorStringLength)
RemoveScsiControllers()
FindIdeSlot(Controller, Slot)
GetAttachedVhdsInfo(AttachedVhdsInfo, ErrorString, ErrorStringLength)
GetAttachedVhdsInfoEx(SystemSettingData, AttachedVhdsInfo, ErrorString, ErrorStringLength)
IsScsiSlotInUse(Controller, Slot, IsInUse)
FindMatchingDisk(Path, MatchingInstancesCount, MatchingInstances, MatchingDiskCount, MatchingDisk)
SetVmDiskType(VmDiskType)
SetVhdDiskType(VhdPath, VmDiskType)
```

#### NVMe Direct Operations
```
AddNvmeDirectDisk(LocationInfo, EnableGuestPolling, DriverHints, PollingQueuePercent,
                  EnableNumaAwarePlacement, PhysicalSerialNumber, VirtualSerialNumber, ...)
AddNvmeDirectV2Disk(NvmeDirectV2Settings, EnableNumaAwarePlacement, ErrorString, ...)
EnumerateNvmeDirectDisks(LocationPaths)
RemoveNvmeDirectDisk(LocationInfo, ErrorString, ErrorStringLength)
GetNVMeNumaAffinity(LocationInfo, NumaAffinity, ErrorString, ErrorStringLength)
AddNvmeController(EmulatorId, NvmeQueueCount, ContainerId, VtlSystem, AsapNvmeControllerType,
                  ControllerVsId, PhysicalNumaNode, ErrorString, ErrorStringLength)
AddNvmeControllerV2(...)
RemoveNvmeVhds(Paths, ErrorString, ErrorStringLength)
EnumerateDisksOverNvme(DiskPaths, AsapBlockDeviceType)
AttachDisksToNvmeController(Disks, AsapBlockDeviceType, ControllerIndex, Controller,
                            UseSoftwareProtocolEngine, ErrorString, ErrorStringLength)
AttachDisksOverNvme(Disks, AsapBlockDeviceType, ErrorString, ErrorStringLength)
HasAsapController(Found, vtl)
HasSCSIVtl2Controller(Found)
FindVirtualSystemIdentifierForNvmeController(VsidVector, isMpf)
CreateUnderhillController(EmulatorId, EmulatorConfig, ErrorString, ErrorStringLength)
UpdateUnderhillConfig(CurrentUpdateId, UnderhillConfig, ErrorString, ErrorStringLength)
GetUnderhillConfig(CurrentUpdateId, UnderhillConfig, ErrorString, ErrorStringLength)
AddAzureStorageGen2FastPathDevice(ErrorString, ErrorStringLength)
HasAzureStorageGen2FastPathDevice(Found)
```

#### DirectDrive Operations
```
CheckDirectDriveDiskOverScsi(DiskPath, IsDiskAttached)
EnumerateDirectDriveDisksOverScsi(DiskPaths)
EnumerateDirectDriveDisksOverIde(DiskPaths)
AttachDirectDriveDiskOverScsi(DiskPath, ControllerId, SlotId)
AttachDirectDriveDiskOverIde(DiskPath, ControllerId, SlotId, ErrorString, ErrorStringLength)
RemoveDirectDriveDiskOverScsi(DiskPath)
RemoveDirectDriveDiskFromIde(DiskPath)
```

#### Persistent Memory Operations
```
AddPmemController(ErrorString, ErrorStringLength)
```

#### Rust Current Coverage
```rust
// storage/mod.rs - Basic VHD operations
VhdManager::create_vhd(settings: &VhdSettings) -> Result<Vhd>
VhdManager::get_vhd_info(path: &Path) -> Result<VhdInfo>
vm.attach_vhd(path: &Path, controller: u32, slot: u32) -> Result<()>
vm.detach_vhd(path: &Path) -> Result<()>
```

---

### 8. Network Operations

#### C++ Switch Management (VirtualSwitchManagementService.h)
```
CreateSwitch(SwitchName, NumLearnableAddresses, EnableIOV, ErrorString, ErrorStringLength)
DeleteSwitch(SwitchName, ErrorString, ErrorStringLength)
CheckSwitchExists(SwitchName)
GetSwitch(SwitchName, Switch)
GetAllSwitches(Switches, SwitchesCount, ErrorString, ErrorStringLength)
SetupSwitch(SwitchName, InternalSwitchPortName, ExternalSwitchPortName, ExternalNicGuid, ...)
CreateSwitchPort(SwitchName, ExternalPortName, InternalPortName, ErrorString, ErrorStringLength)
DeleteSwitchPort(SwitchName, PortName, ErrorString, ErrorStringLength)
CheckSwitchPortExists(SwitchName, PortName)
GetSwitchPort(SwitchName, PortName, SwitchPort)
GetAllSwitchPortNames(SwitchName, PortCount, SwitchPortNames)
GetVirtualEthernetSwitchPath(SwitchName, VirtualSwitchWmiPath, VirtualSwitchWmiPathLength)
EnableSwitchExtension(SwitchName, ExtensionName, ErrorString, ErrorStringLength)
GetSwitchExtensionFeature(SwitchExtensionFeatureGuid, SwitchExtensionFeature)
AddVfpRequiredToSwitch(SwitchName, ErrorString, ErrorStringLength)
GetExternalNicGuid(ExternalNicGuid, ExternalNicGuidLength)
GetExternalNic(ExternalNic)
GetExternalNic(DeviceId, ExternalNic)
```

#### C++ VM Network Operations (WindowsVirtualComputer.h)
```
AddNetworkAdapter(AdapterType, SwitchName, PortName, MacAddress, ErrorString, ErrorStringLength)
AddManaNetworkAdapter(NicProvider, MacAddresses, NumberOfvCPUs, VTL, InstanceIdGuid, ...)
RemoveAllNics(ErrorString, ErrorStringLength)
ConnectSwitchPort(SwitchName, PortName, AdapterMacAddress, ErrorString, ErrorStringLength)
DisconnectSwitchPort(SwitchName, PortName, ErrorString, ErrorStringLength)
DeleteSwitchIfExist(SwitchName, ErrorString, ErrorStringLength)
GetNetworkConnections(NetworkConnections, NetworkConnectionsCount, ErrorString, ErrorStringLength)
GetNicsInfo(NicsInfo, ErrorString, ErrorStringLength)
GetNicsInfoEx(SystemSettingData, NicsInfo, ErrorString, ErrorStringLength, ...)
GetManaNicsInfo(NicsInfo, ErrorString, ErrorStringLength)
GetNetworkConnection_Reservation_Limit_PortName(PortName, Reservation, Limit, ...)
SetNetworkConnection_Reservation_Limit(PortName, Reservation, Limit, ErrorString, ErrorStringLength)
MarkExtensionRequiredOnAllPorts(SwitchFeatureGuid, ErrorString, ErrorStringLength)
DisableNetworkOffloadFeature(DisableNetworkOffloadsForAccelnetVm, ErrorString, ErrorStringLength)
```

#### Rust Current Coverage
```rust
// network/mod.rs
VirtualSwitch::create(settings: &SwitchSettings) -> Result<VirtualSwitch>
VirtualSwitch::list() -> Result<Vec<VirtualSwitch>>
vm.add_network_adapter(settings: &NetworkAdapterSettings) -> Result<NetworkAdapter>
```

---

### 9. GPU Operations

#### C++ DDA Operations (WindowsVirtualComputer.h)
```
AddDdaDeviceToVm(VmName, LocationPath, VirtualBusId, ErrorString, ErrorStringLength)
RemoveDdaDeviceFromVm(VmName, LocationPath, ErrorString, ErrorStringLength)
```

#### C++ GPU-P Operations (GpupResourceManagement.h)
```
GetPartitionableGpuProperty(PropertyName, Value)
GetPartitionableGpuDetails(GpuHwId, numPartitionableGpu, totalPartitionCount)
GetGpuPartitionAvailStatus(numTotalGpuPartitions, numGpuPartitionAdaptersAssigned, numGpuPartitionsInUse)
PGpuIsPartitionCountEqual(GpuHwId, currPartitionCountPerDevice)
SetPGpuPartitionCount(GpuHwId, newPartitionCountValue, numHostGpuPartitions)
ValidateGpupSettingData(settingDataValidationSuccess)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 10. KVP (Key-Value Pair) Exchange Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
GetKvpItemsEx(KvpItems, UseIntrinsicExchange)
GetKvpItemByName(KvpName, KvpValue, UseIntrinsicExchange)
AddKvpItem(KvpName, KvpValue)
ModifyKvpItem(KvpName, KvpValue)
RemoveKvpItem(KvpName)
GetHostOnlyKvpItems(KvpItems)
AddHostOnlyKvpItems(KvpItems)
ModifyHostOnlyKvpItems(KvpItems)
RemoveHostOnlyKvpItems(KvpItems)
ParseItem(xml, KvpItems)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 11. Serial Console Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
SetSerialConsole(Enable, ErrorString, ErrorStringLength)
SetConsoleMode(Mode, ErrorString, ErrorStringLength)  // Mode: 1=COM1, 2=COM2, 3=None
SerialPort_AzureAttach()
SerialPort_AzureDetach()
SetVMConsoleLogFile(VmConsoleLog)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 12. Display/Thumbnail Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
GetThumbnail(WidthPixels, HeightPixels, ThumbnailArray, ErrorString, ErrorStringLength)
SaveThumbnailToBitmapFile(WidthPixels, HeightPixels, ImageData, OutputFilePath)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 13. VM Configuration Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
SetVmSmbiosStrings(VmUniqueId, DeploymentId, ErrorString, ErrorStringLength, ...)
SetVmBootSourceOrder(BootSourceOrderPreferenceListByBootSourceType, BootSourceOrderPreferenceListSize,
                     BootVhdPath, ErrorString, ErrorStringLength)
GetVmBootSourceOrder(VmBootSourceOrder)
SetInitialMachineConfigurationData(ImcData, ImcDataSize, ErrorString, ErrorStringLength)
SetTurnOffOnGuestRestartProperty(PropertyValue, ErrorString, ErrorStringLength)
GetTurnOffOnGuestRestartProperty(PropertyValue, ErrorString, ErrorStringLength)
ApplyAdditionalContainerProperties(Keys, Values, KeyValuePairCount, FailureType, ...)
GetDefaultVmVersion(DefaultVmVersion)
IsSupportedVmVersion(VmVersion, Result)
ProcessAndSetVmVersion(RequestedVmVersionStr, MinimumVmVersionStr, VmName, SystemSettingData)
```

#### Rust Current Coverage
```rust
// vm/types.rs - Basic generation only
pub enum Generation { Gen1, Gen2 }
```

---

### 14. Validation/Capability Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
SupportsProcessorSettingProperty(PropertyName, Result, ErrorString, ErrorStringLength)
SupportsSystemSettingProperty(PropertyName, Result)
SupportsHWThreadsPerCoreProperty(Result, ErrorString, ErrorStringLength)
SupportsL3CacheWaysProperty(Result, ErrorString, ErrorStringLength)
SupportsExposeVirtualizationExtensionsProperty(Result, ErrorString, ErrorStringLength)
SupportsHierarchicalVirtualizationProperty(Result, ErrorString, ErrorStringLength)
SupportsCpuGroupIdProperty(Result, ErrorString, ErrorStringLength)
IsEnableSocketTopologySupported(IsSupported, ErrorString, ErrorStringLength)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

### 15. Host/System Operations

#### C++ Operations
```
GetInstance(MachineName, Domain, Username, Password, WindowsVirtualComputerInstance)
DisableNumaSpanning(ErrorString, ErrorStringLength)
DisableAssociatorWmiQuery()
EnableAssociatorWmiQuery()
```

#### Rust Current Coverage
```rust
// hyperv.rs
HyperV::connect() -> Result<HyperV>  // Local only
```

---

### 16. VPMU (Virtual Performance Monitoring Unit) Operations

#### C++ Operations (WindowsVirtualComputer.h)
```
UpdateSettingsForVPMUSupport(vpmuEnabled, pmuLbrEnabled, pmuPebsEnabled, ErrorString, ...)
UpdateSettingsForPartialVPMU(ErrorString, ErrorStringLength)
```

#### Rust Current Coverage
```rust
// NONE - Complete gap
```

---

## Module Structure

```
hyperv/src/
├── lib.rs                      # Public API exports
├── error.rs                    # Enhanced error types (extend)
├── hyperv.rs                   # Main HyperV facade (extend)
│
├── wmi/
│   ├── mod.rs                  # WMI utilities
│   ├── connection.rs           # Connection handling (extend for remote)
│   ├── variant.rs              # VARIANT conversion
│   └── job.rs                  # NEW: Async job handling with timeout
│
├── vm/
│   ├── mod.rs                  # VM module exports
│   ├── computer_system.rs      # Msvm_ComputerSystem operations
│   ├── settings.rs             # VmSettings builder
│   ├── state.rs                # VM state types
│   └── types.rs                # Strong types
│
├── processor/                   # NEW: Advanced processor settings
│   ├── mod.rs
│   ├── settings.rs             # Msvm_ProcessorSettingData
│   ├── topology.rs             # NUMA, CCX, socket topology
│   └── types.rs                # CpuGroupId, limits, etc.
│
├── memory/                      # NEW: Advanced memory settings
│   ├── mod.rs
│   ├── settings.rs             # Msvm_MemorySettingData
│   └── types.rs                # Dynamic memory types
│
├── security/                    # NEW: Security settings
│   ├── mod.rs
│   ├── settings.rs             # Msvm_SecuritySettingData
│   ├── tpm.rs                  # vTPM operations
│   └── types.rs                # SecureBoot, isolation types
│
├── migration/                   # NEW: Live migration
│   ├── mod.rs
│   ├── service.rs              # Msvm_VirtualSystemMigrationService
│   ├── job.rs                  # Msvm_MigrationJob
│   ├── settings.rs             # Migration settings
│   └── types.rs                # VHD attachment info, options
│
├── export_import/               # NEW: Export/Import
│   ├── mod.rs
│   ├── export.rs               # Export operations
│   ├── import.rs               # Import operations
│   ├── planned_vm.rs           # Planned VM management
│   └── types.rs                # Export/import options
│
├── kvp/                         # NEW: KVP exchange
│   ├── mod.rs
│   ├── exchange.rs             # KVP operations
│   └── types.rs                # KVP types
│
├── serial/                      # NEW: Serial console
│   ├── mod.rs
│   ├── port.rs                 # Serial port settings
│   └── types.rs                # Console modes
│
├── network/
│   ├── mod.rs                  # Network exports (extend)
│   ├── adapter.rs              # Network adapter (extend)
│   ├── switch.rs               # Virtual switch (extend)
│   ├── port.rs                 # NEW: Switch ports
│   ├── extension.rs            # NEW: Switch extensions
│   └── types.rs                # NEW: Extended network types
│
├── storage/
│   ├── mod.rs                  # Storage exports
│   ├── controller.rs           # Storage controllers
│   ├── vhd.rs                  # VHD operations
│   └── nvme.rs                 # NEW: NVMe direct support
│
├── gpu/                         # NEW: GPU management
│   ├── mod.rs
│   ├── partition.rs            # GPU-P operations
│   ├── dda.rs                  # DDA operations
│   ├── resource.rs             # GPU resource management
│   └── types.rs                # GPU types
│
├── checkpoint/
│   ├── mod.rs                  # Checkpoint operations (extend)
│   └── types.rs                # Checkpoint types
│
├── display/                     # NEW: Display/thumbnail
│   ├── mod.rs
│   └── thumbnail.rs            # VM thumbnail capture
│
└── validation/                  # NEW: Property validation
    ├── mod.rs
    ├── capabilities.rs         # Feature capability checks
    └── version.rs              # VM version validation
```

---

## 1. Enhanced Error Types

### File: `hyperv/src/error.rs`

```rust
use core::fmt;
use std::time::Duration;

#[cfg(windows)]
use windows::core::Error as WinError;

/// Failure type classification for error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureType {
    /// Transient error, operation can be retried.
    Transient,
    /// Permanent error, operation cannot succeed.
    Permanent,
    /// Configuration error, settings need to be changed.
    Configuration,
    /// Resource error, resources unavailable.
    Resource,
    /// Validation error, input is invalid.
    Validation,
}

/// VM enabled state for error context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmStateError {
    Unknown,
    Running,
    Off,
    ShuttingDown,
    Paused,
    Suspended,
    Starting,
    Stopping,
    Saving,
    Restoring,
    Migrating,
    Other(u16),
}

/// Hyper-V operation errors with typed context.
#[derive(Debug)]
pub enum Error {
    // === Connection Errors ===

    /// Failed to connect to WMI.
    #[cfg(windows)]
    WmiConnection(WinError),

    /// Failed to connect to remote host.
    RemoteConnection {
        host: String,
        source: Box<Error>,
    },

    /// Authentication failed.
    AuthenticationFailed {
        host: String,
        user: Option<String>,
    },

    // === Query Errors ===

    /// Failed to execute WMI query.
    #[cfg(windows)]
    WmiQuery {
        query: String,
        source: WinError
    },

    /// Failed to invoke WMI method.
    #[cfg(windows)]
    WmiMethod {
        class: &'static str,
        method: &'static str,
        source: WinError,
    },

    // === Not Found Errors ===

    /// VM not found by name or ID.
    VmNotFound(String),

    /// Virtual switch not found.
    SwitchNotFound(String),

    /// VHD/VHDX file not found.
    VhdNotFound(String),

    /// Checkpoint not found.
    CheckpointNotFound { vm: String, checkpoint: String },

    /// Migration job not found.
    MigrationJobNotFound(String),

    /// GPU device not found.
    GpuNotFound(String),

    // === State Errors ===

    /// Operation invalid for current VM state.
    InvalidState {
        vm_name: String,
        current: VmStateError,
        operation: &'static str,
        required_states: &'static [VmStateError],
    },

    /// VM is in a transitional state.
    TransitionalState {
        vm_name: String,
        current: VmStateError,
    },

    // === Validation Errors ===

    /// Property validation failed.
    Validation {
        field: &'static str,
        message: String,
        failure_type: FailureType,
    },

    /// Required property missing.
    MissingRequired(&'static str),

    /// Property not supported on this version.
    PropertyNotSupported {
        property: &'static str,
        min_version: &'static str,
    },

    /// VM version not supported.
    UnsupportedVmVersion {
        version: String,
        supported_versions: Vec<String>,
    },

    // === Operation Errors ===

    /// WMI operation returned failure code.
    OperationFailed {
        operation: &'static str,
        return_value: u32,
        message: String,
        failure_type: FailureType,
    },

    /// Failed to convert WMI VARIANT to expected type.
    TypeConversion {
        property: &'static str,
        expected: &'static str,
    },

    // === Job Errors ===

    /// Job failed during async operation.
    JobFailed {
        operation: &'static str,
        job_id: String,
        error_code: u32,
        error_description: String,
    },

    /// Job timed out.
    JobTimeout {
        operation: &'static str,
        job_id: String,
        timeout: Duration,
        last_progress: u16,
    },

    /// Job was cancelled.
    JobCancelled {
        operation: &'static str,
        job_id: String,
    },

    // === Migration Errors ===

    /// Migration failed.
    MigrationFailed {
        vm_name: String,
        destination: String,
        error_code: u32,
        message: String,
    },

    /// Blackout threshold exceeded.
    BlackoutThresholdExceeded {
        vm_name: String,
        threshold_ms: u64,
        actual_ms: u64,
    },

    // === Security Errors ===

    /// Security operation failed.
    SecurityError {
        operation: &'static str,
        message: String,
    },

    /// TPM operation failed.
    TpmError {
        operation: &'static str,
        message: String,
    },

    // === Resource Errors ===

    /// Insufficient resources.
    InsufficientResources {
        resource_type: &'static str,
        required: String,
        available: String,
    },

    /// GPU partition not available.
    GpuPartitionUnavailable {
        gpu_id: String,
        message: String,
    },

    // === IO Errors ===

    /// File IO error.
    IoError {
        path: String,
        source: std::io::Error,
    },
}

impl Error {
    /// Get the failure type for this error.
    pub fn failure_type(&self) -> FailureType {
        match self {
            Error::WmiConnection(_) => FailureType::Transient,
            Error::RemoteConnection { .. } => FailureType::Transient,
            Error::AuthenticationFailed { .. } => FailureType::Configuration,
            Error::VmNotFound(_) => FailureType::Permanent,
            Error::InvalidState { .. } => FailureType::Transient,
            Error::TransitionalState { .. } => FailureType::Transient,
            Error::Validation { failure_type, .. } => *failure_type,
            Error::OperationFailed { failure_type, .. } => *failure_type,
            Error::JobTimeout { .. } => FailureType::Transient,
            Error::InsufficientResources { .. } => FailureType::Resource,
            _ => FailureType::Permanent,
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self.failure_type(), FailureType::Transient)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::WmiConnection(e) => write!(f, "WMI connection failed: {e}"),
            Error::RemoteConnection { host, source } => {
                write!(f, "Failed to connect to remote host '{host}': {source}")
            }
            Error::AuthenticationFailed { host, user } => {
                write!(f, "Authentication failed for host '{host}'")?;
                if let Some(u) = user {
                    write!(f, " as user '{u}'")?;
                }
                Ok(())
            }
            Error::WmiQuery { query, source } => {
                write!(f, "WMI query failed: {query} - {source}")
            }
            Error::WmiMethod { class, method, source } => {
                write!(f, "WMI method {class}.{method} failed: {source}")
            }
            Error::VmNotFound(name) => write!(f, "VM not found: {name}"),
            Error::SwitchNotFound(name) => write!(f, "Virtual switch not found: {name}"),
            Error::VhdNotFound(path) => write!(f, "VHD not found: {path}"),
            Error::CheckpointNotFound { vm, checkpoint } => {
                write!(f, "Checkpoint '{checkpoint}' not found for VM '{vm}'")
            }
            Error::MigrationJobNotFound(id) => write!(f, "Migration job not found: {id}"),
            Error::GpuNotFound(id) => write!(f, "GPU not found: {id}"),
            Error::InvalidState { vm_name, current, operation, required_states } => {
                write!(f, "Cannot {operation} VM '{vm_name}' in state {current:?}. ")?;
                write!(f, "Required states: {required_states:?}")
            }
            Error::TransitionalState { vm_name, current } => {
                write!(f, "VM '{vm_name}' is in transitional state {current:?}")
            }
            Error::Validation { field, message, .. } => {
                write!(f, "Validation failed for '{field}': {message}")
            }
            Error::MissingRequired(field) => {
                write!(f, "Required field missing: {field}")
            }
            Error::PropertyNotSupported { property, min_version } => {
                write!(f, "Property '{property}' requires VM version {min_version} or later")
            }
            Error::UnsupportedVmVersion { version, supported_versions } => {
                write!(f, "VM version '{version}' not supported. Supported: {supported_versions:?}")
            }
            Error::OperationFailed { operation, return_value, message, .. } => {
                write!(f, "Operation '{operation}' failed with code {return_value}: {message}")
            }
            Error::TypeConversion { property, expected } => {
                write!(f, "Cannot convert property '{property}' to {expected}")
            }
            Error::JobFailed { operation, job_id, error_code, error_description } => {
                write!(f, "Job {job_id} failed for '{operation}' (code {error_code}): {error_description}")
            }
            Error::JobTimeout { operation, job_id, timeout, last_progress } => {
                write!(f, "Job {job_id} timed out after {timeout:?} for '{operation}' (progress: {last_progress}%)")
            }
            Error::JobCancelled { operation, job_id } => {
                write!(f, "Job {job_id} was cancelled for '{operation}'")
            }
            Error::MigrationFailed { vm_name, destination, error_code, message } => {
                write!(f, "Migration of VM '{vm_name}' to '{destination}' failed (code {error_code}): {message}")
            }
            Error::BlackoutThresholdExceeded { vm_name, threshold_ms, actual_ms } => {
                write!(f, "Migration of VM '{vm_name}' exceeded blackout threshold: {actual_ms}ms > {threshold_ms}ms")
            }
            Error::SecurityError { operation, message } => {
                write!(f, "Security operation '{operation}' failed: {message}")
            }
            Error::TpmError { operation, message } => {
                write!(f, "TPM operation '{operation}' failed: {message}")
            }
            Error::InsufficientResources { resource_type, required, available } => {
                write!(f, "Insufficient {resource_type}: required {required}, available {available}")
            }
            Error::GpuPartitionUnavailable { gpu_id, message } => {
                write!(f, "GPU partition unavailable for '{gpu_id}': {message}")
            }
            Error::IoError { path, source } => {
                write!(f, "IO error for '{path}': {source}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(windows)]
            Error::WmiConnection(e) => Some(e),
            #[cfg(windows)]
            Error::WmiQuery { source, .. } => Some(source),
            #[cfg(windows)]
            Error::WmiMethod { source, .. } => Some(source),
            Error::RemoteConnection { source, .. } => Some(source.as_ref()),
            Error::IoError { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Result type for Hyper-V operations.
pub type Result<T> = core::result::Result<T, Error>;
```

---

## 2. WMI Connection (Remote Support)

### File: `hyperv/src/wmi/connection.rs`

```rust
use crate::error::{Error, Result};
use std::time::Duration;

/// Credentials for remote WMI connection.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Domain name (optional).
    pub domain: Option<String>,
    /// Username.
    pub username: String,
    /// Password (securely stored).
    password: secrecy::SecretString,
}

impl Credentials {
    /// Create new credentials.
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            domain: None,
            username: username.into(),
            password: secrecy::SecretString::new(password.into()),
        }
    }

    /// Create credentials with domain.
    pub fn with_domain(
        domain: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            domain: Some(domain.into()),
            username: username.into(),
            password: secrecy::SecretString::new(password.into()),
        }
    }

    /// Get password for WMI connection (internal use only).
    pub(crate) fn password_str(&self) -> &str {
        use secrecy::ExposeSecret;
        self.password.expose_secret()
    }
}

/// WMI connection configuration.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Target machine name (None for local).
    pub machine_name: Option<String>,
    /// Credentials for remote connection.
    pub credentials: Option<Credentials>,
    /// Connection timeout.
    pub timeout: Duration,
    /// WMI namespace (default: root\virtualization\v2).
    pub namespace: String,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            machine_name: None,
            credentials: None,
            timeout: Duration::from_secs(30),
            namespace: "root\\virtualization\\v2".to_string(),
        }
    }
}

impl ConnectionConfig {
    /// Create local connection config.
    pub fn local() -> Self {
        Self::default()
    }

    /// Create remote connection config.
    pub fn remote(machine_name: impl Into<String>) -> Self {
        Self {
            machine_name: Some(machine_name.into()),
            ..Default::default()
        }
    }

    /// Add credentials.
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// WMI connection for Hyper-V management.
pub struct WmiConnection {
    #[cfg(windows)]
    services: windows::Win32::System::Wmi::IWbemServices,
    config: ConnectionConfig,
    /// Cached VM name to path mapping.
    vm_cache: std::collections::HashMap<String, String>,
    /// Whether to use associator query optimization.
    use_associator_optimization: bool,
}

impl WmiConnection {
    /// Connect to local Hyper-V WMI.
    pub fn connect_local() -> Result<Self> {
        Self::connect(ConnectionConfig::local())
    }

    /// Connect to remote Hyper-V WMI.
    pub fn connect_remote(
        machine_name: impl Into<String>,
        credentials: Credentials,
    ) -> Result<Self> {
        Self::connect(
            ConnectionConfig::remote(machine_name)
                .with_credentials(credentials)
        )
    }

    /// Connect with configuration.
    pub fn connect(config: ConnectionConfig) -> Result<Self> {
        #[cfg(windows)]
        {
            use windows::Win32::System::Com::*;
            use windows::Win32::System::Wmi::*;
            use windows::core::*;

            unsafe {
                // Initialize COM
                CoInitializeEx(None, COINIT_MULTITHREADED)?;

                // Set security
                CoInitializeSecurity(
                    None,
                    -1,
                    None,
                    None,
                    RPC_C_AUTHN_LEVEL_DEFAULT,
                    RPC_C_IMP_LEVEL_IMPERSONATE,
                    None,
                    EOAC_NONE,
                    None,
                )?;

                // Create locator
                let locator: IWbemLocator = CoCreateInstance(
                    &WbemLocator,
                    None,
                    CLSCTX_INPROC_SERVER,
                )?;

                // Build namespace path
                let namespace = if let Some(ref machine) = config.machine_name {
                    format!("\\\\{}\\{}", machine, config.namespace)
                } else {
                    config.namespace.clone()
                };

                // Connect
                let services = if let Some(ref creds) = config.credentials {
                    let user = if let Some(ref domain) = creds.domain {
                        format!("{}\\{}", domain, creds.username)
                    } else {
                        creds.username.clone()
                    };

                    locator.ConnectServer(
                        &BSTR::from(&namespace),
                        &BSTR::from(&user),
                        &BSTR::from(creds.password_str()),
                        &BSTR::new(),
                        WBEM_FLAG_CONNECT_USE_MAX_WAIT.0 as i32,
                        &BSTR::new(),
                        None,
                    )?
                } else {
                    locator.ConnectServer(
                        &BSTR::from(&namespace),
                        &BSTR::new(),
                        &BSTR::new(),
                        &BSTR::new(),
                        0,
                        &BSTR::new(),
                        None,
                    )?
                };

                // Set proxy security
                CoSetProxyBlanket(
                    &services,
                    RPC_C_AUTHN_WINNT,
                    RPC_C_AUTHZ_NONE,
                    None,
                    RPC_C_AUTHN_LEVEL_CALL,
                    RPC_C_IMP_LEVEL_IMPERSONATE,
                    None,
                    EOAC_NONE,
                )?;

                Ok(Self {
                    services,
                    config,
                    vm_cache: std::collections::HashMap::new(),
                    use_associator_optimization: true,
                })
            }
        }

        #[cfg(not(windows))]
        Err(Error::OperationFailed {
            operation: "connect",
            return_value: 0,
            message: "WMI only available on Windows".to_string(),
            failure_type: crate::error::FailureType::Permanent,
        })
    }

    /// Enable or disable associator query optimization.
    pub fn set_associator_optimization(&mut self, enabled: bool) {
        self.use_associator_optimization = enabled;
    }

    /// Clear VM name cache.
    pub fn clear_cache(&mut self) {
        self.vm_cache.clear();
    }

    /// Get connection configuration.
    pub fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    /// Check if connected to remote machine.
    pub fn is_remote(&self) -> bool {
        self.config.machine_name.is_some()
    }
}
```

---

## 3. Async Job Handling with Timeout

### File: `hyperv/src/wmi/job.rs`

```rust
use crate::error::{Error, Result};
use std::time::{Duration, Instant};

/// WMI job state values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum JobState {
    /// Job is queued.
    New = 2,
    /// Job is starting.
    Starting = 3,
    /// Job is running.
    Running = 4,
    /// Job is suspended.
    Suspended = 5,
    /// Job is shutting down.
    ShuttingDown = 6,
    /// Job completed successfully.
    Completed = 7,
    /// Job was terminated.
    Terminated = 8,
    /// Job was killed.
    Killed = 9,
    /// Job failed with exception.
    Exception = 10,
    /// Job is in service mode.
    Service = 11,
    /// Unknown state.
    Unknown = 0,
}

impl From<u16> for JobState {
    fn from(value: u16) -> Self {
        match value {
            2 => JobState::New,
            3 => JobState::Starting,
            4 => JobState::Running,
            5 => JobState::Suspended,
            6 => JobState::ShuttingDown,
            7 => JobState::Completed,
            8 => JobState::Terminated,
            9 => JobState::Killed,
            10 => JobState::Exception,
            11 => JobState::Service,
            _ => JobState::Unknown,
        }
    }
}

impl JobState {
    /// Check if job is still running.
    pub fn is_running(&self) -> bool {
        matches!(self,
            JobState::New |
            JobState::Starting |
            JobState::Running |
            JobState::Suspended |
            JobState::ShuttingDown
        )
    }

    /// Check if job completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, JobState::Completed)
    }

    /// Check if job failed.
    pub fn is_failed(&self) -> bool {
        matches!(self,
            JobState::Terminated |
            JobState::Killed |
            JobState::Exception
        )
    }
}

/// Job status information.
#[derive(Debug, Clone)]
pub struct JobStatus {
    /// Job instance ID.
    pub job_id: String,
    /// Current state.
    pub state: JobState,
    /// Percent complete (0-100).
    pub percent_complete: u16,
    /// Error code (0 = success).
    pub error_code: u32,
    /// Error description.
    pub error_description: String,
    /// Status string.
    pub status: String,
    /// Elapsed time in milliseconds.
    pub elapsed_time_ms: u64,
}

/// Job options for async operations.
#[derive(Debug, Clone)]
pub struct JobOptions {
    /// Timeout for job completion.
    pub timeout: Duration,
    /// Poll interval.
    pub poll_interval: Duration,
    /// Progress callback.
    pub on_progress: Option<Box<dyn Fn(u16) + Send + Sync>>,
}

impl Default for JobOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(20 * 60), // 20 minutes
            poll_interval: Duration::from_millis(100),
            on_progress: None,
        }
    }
}

impl JobOptions {
    /// Create with timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }

    /// Set poll interval.
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Set progress callback.
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u16) + Send + Sync + 'static,
    {
        self.on_progress = Some(Box::new(callback));
        self
    }
}

/// Wait for a WMI job to complete.
pub fn wait_for_job(
    conn: &WmiConnection,
    job_path: &str,
    operation: &'static str,
    options: &JobOptions,
) -> Result<JobStatus> {
    let start = Instant::now();
    let mut last_progress = 0u16;

    loop {
        // Check timeout
        if start.elapsed() > options.timeout {
            return Err(Error::JobTimeout {
                operation,
                job_id: job_path.to_string(),
                timeout: options.timeout,
                last_progress,
            });
        }

        // Query job status
        let status = get_job_status(conn, job_path)?;

        // Report progress
        if status.percent_complete != last_progress {
            last_progress = status.percent_complete;
            if let Some(ref callback) = options.on_progress {
                callback(last_progress);
            }
        }

        // Check completion
        if status.state.is_success() {
            return Ok(status);
        }

        if status.state.is_failed() {
            return Err(Error::JobFailed {
                operation,
                job_id: job_path.to_string(),
                error_code: status.error_code,
                error_description: status.error_description,
            });
        }

        // Wait before next poll
        std::thread::sleep(options.poll_interval);
    }
}

/// Get job status without waiting.
pub fn get_job_status(conn: &WmiConnection, job_path: &str) -> Result<JobStatus> {
    #[cfg(windows)]
    {
        use windows::core::BSTR;

        let obj = conn.get_object(job_path)?;

        Ok(JobStatus {
            job_id: obj.get_string("InstanceID")?.unwrap_or_default(),
            state: JobState::from(obj.get_u16("JobState")?.unwrap_or(0)),
            percent_complete: obj.get_u16("PercentComplete")?.unwrap_or(0),
            error_code: obj.get_u32("ErrorCode")?.unwrap_or(0),
            error_description: obj.get_string("ErrorDescription")?.unwrap_or_default(),
            status: obj.get_string("JobStatus")?.unwrap_or_default(),
            elapsed_time_ms: obj.get_u64("ElapsedTime")?.unwrap_or(0),
        })
    }

    #[cfg(not(windows))]
    Err(Error::OperationFailed {
        operation: "get_job_status",
        return_value: 0,
        message: "Not available on this platform".to_string(),
        failure_type: crate::error::FailureType::Permanent,
    })
}

/// Cancel a running job.
pub fn cancel_job(conn: &WmiConnection, job_path: &str) -> Result<()> {
    #[cfg(windows)]
    {
        conn.exec_method(job_path, "RequestStateChange", |params| {
            params.put_u16("RequestedState", 3)?; // 3 = Terminate
            Ok(())
        })?;
        Ok(())
    }

    #[cfg(not(windows))]
    Err(Error::OperationFailed {
        operation: "cancel_job",
        return_value: 0,
        message: "Not available on this platform".to_string(),
        failure_type: crate::error::FailureType::Permanent,
    })
}
```

---

## 4. Migration Module

### File: `hyperv/src/migration/mod.rs`

```rust
//! Live migration operations for Hyper-V VMs.
//!
//! Supports:
//! - Live migration (VM keeps running)
//! - Quick migration (VM briefly pauses)
//! - Storage migration (VHDs only)
//! - Planned migration (stopped VM)

mod service;
mod job;
mod settings;
mod types;

pub use service::MigrationService;
pub use job::{MigrationJob, MigrationJobStatus};
pub use settings::{MigrationSettings, MigrationSettingsBuilder};
pub use types::*;
```

### File: `hyperv/src/migration/types.rs`

```rust
use crate::error::{Error, Result};
use std::path::PathBuf;

/// Migration type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationType {
    /// Live migration - VM stays running.
    Live,
    /// Quick migration - VM briefly pauses.
    Quick,
    /// Storage migration - Move VHDs only.
    Storage,
    /// Offline migration - VM is stopped.
    Offline,
}

impl MigrationType {
    /// Get WMI migration type value.
    pub fn to_wmi_value(&self) -> u16 {
        match self {
            MigrationType::Live => 32768,      // VirtualSystemAndStorage
            MigrationType::Quick => 32769,     // VirtualSystem
            MigrationType::Storage => 32770,   // Storage
            MigrationType::Offline => 32771,   // PlannedVirtualSystem
        }
    }
}

/// Migration transport type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportType {
    /// TCP transport (default).
    #[default]
    Tcp,
    /// SMB transport (faster for storage).
    Smb,
    /// Compression enabled.
    Compression,
}

impl TransportType {
    /// Get WMI transport type value.
    pub fn to_wmi_value(&self) -> u16 {
        match self {
            TransportType::Tcp => 0,
            TransportType::Smb => 1,
            TransportType::Compression => 2,
        }
    }
}

/// CPU capping magnitude during migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CpuCappingMagnitude {
    /// Normal capping.
    #[default]
    Normal = 0,
    /// Low capping (less aggressive).
    Low = 1,
    /// High capping (more aggressive, faster migration).
    High = 2,
}

/// VHD attachment info for storage migration.
#[derive(Debug, Clone)]
pub struct VhdAttachmentInfo {
    /// Source VHD path.
    pub source_path: PathBuf,
    /// Destination VHD path.
    pub destination_path: PathBuf,
    /// Controller type (IDE/SCSI).
    pub controller_type: ControllerType,
    /// Controller number.
    pub controller_number: u32,
    /// Controller location.
    pub controller_location: u32,
    /// Overwrite existing VHD at destination.
    pub overwrite_existing: bool,
}

/// Controller type for VHD attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerType {
    Ide,
    Scsi,
    Nvme,
}

/// Migration capability information.
#[derive(Debug, Clone)]
pub struct MigrationCapability {
    /// Whether live migration is available.
    pub live_migration_available: bool,
    /// Whether storage migration is available.
    pub storage_migration_available: bool,
    /// Maximum concurrent migrations.
    pub max_concurrent_migrations: u32,
    /// Supported transport types.
    pub supported_transports: Vec<TransportType>,
}
```

### File: `hyperv/src/migration/settings.rs`

```rust
use super::types::*;
use crate::error::{Error, Result, FailureType};
use std::path::PathBuf;
use std::time::Duration;

/// Migration settings for a VM migration operation.
#[derive(Debug, Clone)]
pub struct MigrationSettings {
    /// Destination host name.
    pub destination_host: String,
    /// Migration type.
    pub migration_type: MigrationType,
    /// Transport type.
    pub transport_type: TransportType,
    /// VHDs to migrate (for storage migration).
    pub vhds: Vec<VhdAttachmentInfo>,
    /// Destination path for VHDs.
    pub destination_path: Option<PathBuf>,
    /// New VM name at destination.
    pub destination_vm_name: Option<String>,
    /// Overwrite existing VHDs.
    pub overwrite_existing_vhds: bool,
    /// Avoid removing source VHDs after migration.
    pub retain_source_vhds: bool,
    /// Cancel if blackout threshold exceeded.
    pub cancel_on_blackout_threshold: bool,
    /// Blackout threshold in milliseconds.
    pub blackout_threshold_ms: Option<u64>,
    /// Enable compression.
    pub enable_compression: bool,
    /// Enable SMB transport.
    pub enable_smb_transport: bool,
    /// Skip resource disk migration.
    pub skip_resource_disk: bool,
    /// CPU capping magnitude.
    pub cpu_capping: CpuCappingMagnitude,
    /// Include VMGS file in migration.
    pub include_vmgs_file: bool,
    /// Operation timeout.
    pub timeout: Duration,
}

impl Default for MigrationSettings {
    fn default() -> Self {
        Self {
            destination_host: String::new(),
            migration_type: MigrationType::Live,
            transport_type: TransportType::Tcp,
            vhds: Vec::new(),
            destination_path: None,
            destination_vm_name: None,
            overwrite_existing_vhds: false,
            retain_source_vhds: false,
            cancel_on_blackout_threshold: false,
            blackout_threshold_ms: None,
            enable_compression: true,
            enable_smb_transport: false,
            skip_resource_disk: false,
            cpu_capping: CpuCappingMagnitude::Normal,
            include_vmgs_file: false,
            timeout: Duration::from_secs(60 * 60), // 1 hour default
        }
    }
}

/// Builder for migration settings.
#[derive(Debug, Clone, Default)]
pub struct MigrationSettingsBuilder {
    settings: MigrationSettings,
}

impl MigrationSettingsBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set destination host (required).
    pub fn destination_host(mut self, host: impl Into<String>) -> Self {
        self.settings.destination_host = host.into();
        self
    }

    /// Set migration type.
    pub fn migration_type(mut self, migration_type: MigrationType) -> Self {
        self.settings.migration_type = migration_type;
        self
    }

    /// Use live migration.
    pub fn live(self) -> Self {
        self.migration_type(MigrationType::Live)
    }

    /// Use quick migration.
    pub fn quick(self) -> Self {
        self.migration_type(MigrationType::Quick)
    }

    /// Use storage-only migration.
    pub fn storage_only(self) -> Self {
        self.migration_type(MigrationType::Storage)
    }

    /// Set transport type.
    pub fn transport(mut self, transport: TransportType) -> Self {
        self.settings.transport_type = transport;
        self
    }

    /// Add VHD to migrate.
    pub fn add_vhd(mut self, vhd: VhdAttachmentInfo) -> Self {
        self.settings.vhds.push(vhd);
        self
    }

    /// Set destination path for VHDs.
    pub fn destination_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.settings.destination_path = Some(path.into());
        self
    }

    /// Set new VM name at destination.
    pub fn destination_vm_name(mut self, name: impl Into<String>) -> Self {
        self.settings.destination_vm_name = Some(name.into());
        self
    }

    /// Overwrite existing VHDs at destination.
    pub fn overwrite_existing(mut self, overwrite: bool) -> Self {
        self.settings.overwrite_existing_vhds = overwrite;
        self
    }

    /// Retain source VHDs after migration.
    pub fn retain_source_vhds(mut self, retain: bool) -> Self {
        self.settings.retain_source_vhds = retain;
        self
    }

    /// Cancel if blackout threshold exceeded.
    pub fn cancel_on_blackout(mut self, threshold_ms: u64) -> Self {
        self.settings.cancel_on_blackout_threshold = true;
        self.settings.blackout_threshold_ms = Some(threshold_ms);
        self
    }

    /// Enable compression.
    pub fn compression(mut self, enabled: bool) -> Self {
        self.settings.enable_compression = enabled;
        self
    }

    /// Enable SMB transport.
    pub fn smb_transport(mut self, enabled: bool) -> Self {
        self.settings.enable_smb_transport = enabled;
        self
    }

    /// Skip resource disk migration.
    pub fn skip_resource_disk(mut self, skip: bool) -> Self {
        self.settings.skip_resource_disk = skip;
        self
    }

    /// Set CPU capping magnitude.
    pub fn cpu_capping(mut self, capping: CpuCappingMagnitude) -> Self {
        self.settings.cpu_capping = capping;
        self
    }

    /// Include VMGS file.
    pub fn include_vmgs(mut self, include: bool) -> Self {
        self.settings.include_vmgs_file = include;
        self
    }

    /// Set operation timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.settings.timeout = timeout;
        self
    }

    /// Build and validate settings.
    pub fn build(self) -> Result<MigrationSettings> {
        // Validate destination host
        if self.settings.destination_host.is_empty() {
            return Err(Error::MissingRequired("destination_host"));
        }

        // Validate storage migration has VHDs
        if self.settings.migration_type == MigrationType::Storage
            && self.settings.vhds.is_empty()
        {
            return Err(Error::Validation {
                field: "vhds",
                message: "Storage migration requires at least one VHD".to_string(),
                failure_type: FailureType::Validation,
            });
        }

        Ok(self.settings)
    }
}

impl MigrationSettings {
    /// Create a builder.
    pub fn builder() -> MigrationSettingsBuilder {
        MigrationSettingsBuilder::new()
    }
}
```

### File: `hyperv/src/migration/service.rs`

```rust
use super::{MigrationSettings, MigrationJob, MigrationJobStatus};
use crate::error::{Error, Result};
use crate::wmi::WmiConnection;
use crate::wmi::job::{JobOptions, wait_for_job};

/// Migration service for performing VM migrations.
pub struct MigrationService<'a> {
    conn: &'a WmiConnection,
    vsms_path: String,
}

impl<'a> MigrationService<'a> {
    /// Create a new migration service.
    pub fn new(conn: &'a WmiConnection) -> Result<Self> {
        // Get Msvm_VirtualSystemMigrationService singleton
        let vsms_path = conn.get_singleton_path(
            "Msvm_VirtualSystemMigrationService"
        )?;

        Ok(Self { conn, vsms_path })
    }

    /// Migrate a VM synchronously (waits for completion).
    pub fn migrate_vm(
        &self,
        vm_name: &str,
        settings: &MigrationSettings,
    ) -> Result<()> {
        let job = self.migrate_vm_async(vm_name, settings)?;

        wait_for_job(
            self.conn,
            &job.job_path,
            "MigrateVirtualSystemToHost",
            &JobOptions::with_timeout(settings.timeout),
        )?;

        Ok(())
    }

    /// Migrate a VM asynchronously (returns job).
    pub fn migrate_vm_async(
        &self,
        vm_name: &str,
        settings: &MigrationSettings,
    ) -> Result<MigrationJob> {
        #[cfg(windows)]
        {
            // Get VM computer system path
            let vm_path = self.conn.get_vm_path(vm_name)?;

            // Build migration setting data
            let migration_setting_data = self.build_migration_setting_data(settings)?;

            // Execute migration method
            let result = self.conn.exec_method(
                &self.vsms_path,
                "MigrateVirtualSystemToHost",
                |params| {
                    params.put_reference("ComputerSystem", &vm_path)?;
                    params.put_string("DestinationHost", &settings.destination_host)?;
                    params.put_string("MigrationSettingData", &migration_setting_data)?;

                    // Build new resource setting data for VHDs
                    if !settings.vhds.is_empty() {
                        let resource_settings = self.build_resource_setting_data(settings)?;
                        params.put_string_array("NewResourceSettingData", &resource_settings)?;
                    }

                    Ok(())
                },
            )?;

            // Check return value
            let return_value = result.get_u32("ReturnValue")?.unwrap_or(0);

            match return_value {
                0 => {
                    // Completed synchronously
                    Ok(MigrationJob {
                        job_path: String::new(),
                        vm_name: vm_name.to_string(),
                        destination: settings.destination_host.clone(),
                        completed: true,
                    })
                }
                4096 => {
                    // Job started
                    let job_path = result.get_reference("Job")?;
                    Ok(MigrationJob {
                        job_path,
                        vm_name: vm_name.to_string(),
                        destination: settings.destination_host.clone(),
                        completed: false,
                    })
                }
                code => {
                    let message = get_migration_error_message(code);
                    Err(Error::MigrationFailed {
                        vm_name: vm_name.to_string(),
                        destination: settings.destination_host.clone(),
                        error_code: code,
                        message,
                    })
                }
            }
        }

        #[cfg(not(windows))]
        Err(Error::OperationFailed {
            operation: "migrate_vm",
            return_value: 0,
            message: "Not available on this platform".to_string(),
            failure_type: crate::error::FailureType::Permanent,
        })
    }

    /// Migrate VM to suspended state at destination.
    pub fn migrate_vm_to_suspended(
        &self,
        vm_name: &str,
        settings: &MigrationSettings,
    ) -> Result<MigrationJob> {
        // Same as migrate_vm_async but with different migration type
        let mut settings = settings.clone();
        settings.migration_type = super::MigrationType::Quick;
        self.migrate_vm_async(vm_name, &settings)
    }

    /// Get migration job status.
    pub fn get_migration_status(&self, job_id: &str) -> Result<MigrationJobStatus> {
        #[cfg(windows)]
        {
            let job_path = format!(
                "Msvm_MigrationJob.InstanceID=\"{}\"",
                job_id
            );

            let obj = self.conn.get_object(&job_path)?;

            Ok(MigrationJobStatus {
                job_id: job_id.to_string(),
                state: obj.get_u16("JobState")?.unwrap_or(0).into(),
                percent_complete: obj.get_u16("PercentComplete")?.unwrap_or(0),
                error_code: obj.get_u32("ErrorCode")?.unwrap_or(0),
                error_description: obj.get_string("ErrorDescription")?.unwrap_or_default(),
                elapsed_time_ms: obj.get_u64("ElapsedTime")?.unwrap_or(0),
                vm_name: obj.get_string("VirtualSystemName")?.unwrap_or_default(),
                destination_host: obj.get_string("DestinationHost")?.unwrap_or_default(),
            })
        }

        #[cfg(not(windows))]
        Err(Error::MigrationJobNotFound(job_id.to_string()))
    }

    /// Cancel an ongoing migration.
    pub fn cancel_migration(&self, job_id: &str) -> Result<()> {
        #[cfg(windows)]
        {
            let job_path = format!(
                "Msvm_MigrationJob.InstanceID=\"{}\"",
                job_id
            );

            self.conn.exec_method(&job_path, "RequestStateChange", |params| {
                params.put_u16("RequestedState", 3)?; // Terminate
                Ok(())
            })?;

            Ok(())
        }

        #[cfg(not(windows))]
        Err(Error::MigrationJobNotFound(job_id.to_string()))
    }

    /// Get ongoing migration job for a VM.
    pub fn get_ongoing_migration(
        &self,
        vm_id: &str,
        destination_host: &str,
    ) -> Result<Option<MigrationJob>> {
        #[cfg(windows)]
        {
            let query = format!(
                "SELECT * FROM Msvm_MigrationJob WHERE \
                 VirtualSystemName='{}' AND DestinationHost='{}' AND \
                 (JobState=2 OR JobState=3 OR JobState=4)",
                vm_id, destination_host
            );

            let jobs: Vec<_> = self.conn.query(&query)?.collect();

            if let Some(job) = jobs.first() {
                Ok(Some(MigrationJob {
                    job_path: job.path()?,
                    vm_name: job.get_string("VirtualSystemName")?.unwrap_or_default(),
                    destination: destination_host.to_string(),
                    completed: false,
                }))
            } else {
                Ok(None)
            }
        }

        #[cfg(not(windows))]
        Ok(None)
    }

    /// Check migration capability.
    pub fn get_capability(&self) -> Result<super::MigrationCapability> {
        #[cfg(windows)]
        {
            let obj = self.conn.get_object(&self.vsms_path)?;

            Ok(super::MigrationCapability {
                live_migration_available: true, // Check from settings
                storage_migration_available: true,
                max_concurrent_migrations: 2, // Default
                supported_transports: vec![
                    super::TransportType::Tcp,
                    super::TransportType::Compression,
                ],
            })
        }

        #[cfg(not(windows))]
        Err(Error::OperationFailed {
            operation: "get_capability",
            return_value: 0,
            message: "Not available on this platform".to_string(),
            failure_type: crate::error::FailureType::Permanent,
        })
    }

    #[cfg(windows)]
    fn build_migration_setting_data(&self, settings: &MigrationSettings) -> Result<String> {
        // Create Msvm_VirtualSystemMigrationSettingData instance
        let setting_class = self.conn.get_class("Msvm_VirtualSystemMigrationSettingData")?;
        let setting_obj = setting_class.spawn_instance()?;

        setting_obj.put_u16("MigrationType", settings.migration_type.to_wmi_value())?;
        setting_obj.put_u16("TransportType", settings.transport_type.to_wmi_value())?;
        setting_obj.put_bool("EnableCompression", settings.enable_compression)?;

        if let Some(ref path) = settings.destination_path {
            setting_obj.put_string("DestinationPlannedVirtualSystemPath",
                path.to_string_lossy().as_ref())?;
        }

        setting_obj.get_text()
    }

    #[cfg(windows)]
    fn build_resource_setting_data(&self, settings: &MigrationSettings) -> Result<Vec<String>> {
        let mut result = Vec::new();

        for vhd in &settings.vhds {
            let storage_class = self.conn.get_class("Msvm_StorageAllocationSettingData")?;
            let storage_obj = storage_class.spawn_instance()?;

            storage_obj.put_string_array("HostResource", &[
                vhd.destination_path.to_string_lossy().as_ref()
            ])?;

            result.push(storage_obj.get_text()?);
        }

        Ok(result)
    }
}

fn get_migration_error_message(code: u32) -> String {
    match code {
        0 => "Success".to_string(),
        1 => "Not supported".to_string(),
        2 => "Failed".to_string(),
        3 => "Timeout".to_string(),
        4 => "Invalid parameter".to_string(),
        5 => "Invalid state".to_string(),
        6 => "Incompatible parameters".to_string(),
        4096 => "Job started".to_string(),
        32768 => "VM not found".to_string(),
        32769 => "Destination host unreachable".to_string(),
        32770 => "Insufficient resources".to_string(),
        _ => format!("Unknown error code: {}", code),
    }
}
```

---

## 5. Security Module

### File: `hyperv/src/security/mod.rs`

```rust
//! Security settings for Hyper-V VMs.
//!
//! Supports:
//! - Secure Boot
//! - vTPM (Virtual Trusted Platform Module)
//! - Guest state isolation (Shielded VMs)

mod settings;
mod tpm;
mod types;

pub use settings::{SecuritySettings, SecuritySettingsBuilder};
pub use tpm::{TpmOperations, TpmState};
pub use types::*;
```

### File: `hyperv/src/security/types.rs`

```rust
/// Guest state isolation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GuestIsolationType {
    /// No isolation.
    #[default]
    None = 0,
    /// VBS (Virtualization-based Security) isolation.
    Vbs = 1,
    /// SNP (AMD SEV-SNP) isolation.
    Snp = 2,
    /// TDX (Intel TDX) isolation.
    Tdx = 3,
}

impl From<u16> for GuestIsolationType {
    fn from(value: u16) -> Self {
        match value {
            0 => GuestIsolationType::None,
            1 => GuestIsolationType::Vbs,
            2 => GuestIsolationType::Snp,
            3 => GuestIsolationType::Tdx,
            _ => GuestIsolationType::None,
        }
    }
}

/// Secure boot template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureBootTemplate {
    /// Microsoft Windows template.
    MicrosoftWindows,
    /// Microsoft UEFI Certificate Authority.
    MicrosoftUefiCa,
    /// Open Source Shielded VM template.
    OpenSourceShieldedVm,
}

impl SecureBootTemplate {
    /// Get template GUID.
    pub fn to_guid(&self) -> &'static str {
        match self {
            SecureBootTemplate::MicrosoftWindows =>
                "{1734c6e8-3154-4dda-ba5f-a874cc483422}",
            SecureBootTemplate::MicrosoftUefiCa =>
                "{272e7447-90a4-4563-a4b9-8e4ab00526ce}",
            SecureBootTemplate::OpenSourceShieldedVm =>
                "{5c5b03be-6e38-4d00-a6a8-62b9b5f3aa72}",
        }
    }
}

/// Firmware type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareType {
    /// Legacy BIOS.
    Bios,
    /// UEFI.
    Uefi,
}

impl From<u16> for FirmwareType {
    fn from(value: u16) -> Self {
        match value {
            1 => FirmwareType::Uefi,
            _ => FirmwareType::Bios,
        }
    }
}

/// Key protector type for shielded VMs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyProtectorType {
    /// No key protector.
    None,
    /// Local key protector.
    Local,
    /// Host Guardian Service key protector.
    Hgs,
}
```

### File: `hyperv/src/security/settings.rs`

```rust
use super::types::*;
use crate::error::{Error, Result, FailureType};
use crate::wmi::WmiConnection;

/// Security settings for a VM.
#[derive(Debug, Clone)]
pub struct SecuritySettings {
    /// Enable Secure Boot.
    pub secure_boot_enabled: bool,
    /// Secure Boot template.
    pub secure_boot_template: Option<SecureBootTemplate>,
    /// Enable vTPM.
    pub tpm_enabled: bool,
    /// Guest state isolation type.
    pub guest_isolation_type: GuestIsolationType,
    /// Encrypt VM state and migration traffic.
    pub encrypt_state_and_migration: bool,
    /// Enable shielding.
    pub shielding_requested: bool,
    /// Data encryption enabled.
    pub data_encryption_enabled: bool,
    /// Key protector type.
    pub key_protector_type: KeyProtectorType,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            secure_boot_enabled: false,
            secure_boot_template: None,
            tpm_enabled: false,
            guest_isolation_type: GuestIsolationType::None,
            encrypt_state_and_migration: false,
            shielding_requested: false,
            data_encryption_enabled: false,
            key_protector_type: KeyProtectorType::None,
        }
    }
}

/// Builder for security settings.
#[derive(Debug, Clone, Default)]
pub struct SecuritySettingsBuilder {
    settings: SecuritySettings,
}

impl SecuritySettingsBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable Secure Boot.
    pub fn secure_boot(mut self, enabled: bool) -> Self {
        self.settings.secure_boot_enabled = enabled;
        self
    }

    /// Set Secure Boot template.
    pub fn secure_boot_template(mut self, template: SecureBootTemplate) -> Self {
        self.settings.secure_boot_template = Some(template);
        self.settings.secure_boot_enabled = true;
        self
    }

    /// Enable Secure Boot for Windows.
    pub fn secure_boot_windows(self) -> Self {
        self.secure_boot_template(SecureBootTemplate::MicrosoftWindows)
    }

    /// Enable Secure Boot for Linux.
    pub fn secure_boot_linux(self) -> Self {
        self.secure_boot_template(SecureBootTemplate::MicrosoftUefiCa)
    }

    /// Enable vTPM.
    pub fn tpm(mut self, enabled: bool) -> Self {
        self.settings.tpm_enabled = enabled;
        self
    }

    /// Set guest isolation type.
    pub fn guest_isolation(mut self, isolation: GuestIsolationType) -> Self {
        self.settings.guest_isolation_type = isolation;
        self
    }

    /// Enable VBS isolation.
    pub fn vbs_isolation(self) -> Self {
        self.guest_isolation(GuestIsolationType::Vbs)
    }

    /// Encrypt state and migration traffic.
    pub fn encrypt_state_and_migration(mut self, enabled: bool) -> Self {
        self.settings.encrypt_state_and_migration = enabled;
        self
    }

    /// Enable shielding.
    pub fn shielding(mut self, enabled: bool) -> Self {
        self.settings.shielding_requested = enabled;
        self
    }

    /// Build settings.
    pub fn build(self) -> Result<SecuritySettings> {
        // Validate: Secure Boot requires template
        if self.settings.secure_boot_enabled && self.settings.secure_boot_template.is_none() {
            return Err(Error::Validation {
                field: "secure_boot_template",
                message: "Secure Boot requires a template".to_string(),
                failure_type: FailureType::Validation,
            });
        }

        // Validate: Shielding requires TPM
        if self.settings.shielding_requested && !self.settings.tpm_enabled {
            return Err(Error::Validation {
                field: "tpm_enabled",
                message: "Shielding requires vTPM to be enabled".to_string(),
                failure_type: FailureType::Validation,
            });
        }

        Ok(self.settings)
    }
}

impl SecuritySettings {
    /// Create a builder.
    pub fn builder() -> SecuritySettingsBuilder {
        SecuritySettingsBuilder::new()
    }

    /// Get security settings for a VM.
    pub fn get(conn: &WmiConnection, vm_id: &str) -> Result<Self> {
        #[cfg(windows)]
        {
            // Query Msvm_SecuritySettingData
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_SystemSettingDataComponent \
                 ResultClass=Msvm_SecuritySettingData",
                vm_id
            );

            let results: Vec<_> = conn.query(&query)?.collect();

            if let Some(obj) = results.first() {
                Ok(SecuritySettings {
                    secure_boot_enabled: obj.get_bool("SecureBootEnabled")?.unwrap_or(false),
                    secure_boot_template: None, // Need separate query
                    tpm_enabled: obj.get_bool("TpmEnabled")?.unwrap_or(false),
                    guest_isolation_type: GuestIsolationType::from(
                        obj.get_u16("GuestStateIsolationType")?.unwrap_or(0)
                    ),
                    encrypt_state_and_migration: obj.get_bool("EncryptStateAndVmMigrationTraffic")?.unwrap_or(false),
                    shielding_requested: obj.get_bool("ShieldingRequested")?.unwrap_or(false),
                    data_encryption_enabled: obj.get_bool("DataEncryptionEnabled")?.unwrap_or(false),
                    key_protector_type: KeyProtectorType::None,
                })
            } else {
                Ok(SecuritySettings::default())
            }
        }

        #[cfg(not(windows))]
        Ok(SecuritySettings::default())
    }

    /// Apply security settings to a VM.
    pub fn apply(&self, conn: &WmiConnection, vm_id: &str) -> Result<()> {
        #[cfg(windows)]
        {
            // Get VSMS
            let vsms_path = conn.get_singleton_path("Msvm_VirtualSystemManagementService")?;

            // Get current security setting data
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_SystemSettingDataComponent \
                 ResultClass=Msvm_SecuritySettingData",
                vm_id
            );

            let results: Vec<_> = conn.query(&query)?.collect();

            if let Some(security_data) = results.first() {
                // Modify existing
                security_data.put_bool("TpmEnabled", self.tpm_enabled)?;
                security_data.put_bool("SecureBootEnabled", self.secure_boot_enabled)?;
                security_data.put_bool("EncryptStateAndVmMigrationTraffic",
                    self.encrypt_state_and_migration)?;
                security_data.put_bool("ShieldingRequested", self.shielding_requested)?;
                security_data.put_u16("GuestStateIsolationType",
                    self.guest_isolation_type as u16)?;

                // Apply via ModifySecuritySettings
                conn.exec_method(&vsms_path, "ModifySecuritySettings", |params| {
                    params.put_string("SecuritySettingData", &security_data.get_text()?)?;
                    Ok(())
                })?;
            }

            Ok(())
        }

        #[cfg(not(windows))]
        Ok(())
    }
}
```

---

## 6. Processor Module (Advanced Settings)

### File: `hyperv/src/processor/mod.rs`

```rust
//! Advanced processor settings for Hyper-V VMs.
//!
//! Supports:
//! - CPU count and limits
//! - NUMA topology
//! - AMD CCX/CCD configuration
//! - CPU groups
//! - Hardware thread configuration

mod settings;
mod topology;
mod types;

pub use settings::{ProcessorSettings, ProcessorSettingsBuilder};
pub use topology::*;
pub use types::*;
```

### File: `hyperv/src/processor/types.rs`

```rust
use crate::error::{Error, Result, FailureType};

/// CPU limit (0-100000, representing 0-100% in units of 0.001%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuLimit(u64);

impl CpuLimit {
    /// No limit (100%).
    pub const NONE: Self = Self(100000);

    /// Create from percentage (0.0 - 100.0).
    pub fn from_percent(percent: f64) -> Option<Self> {
        if percent >= 0.0 && percent <= 100.0 {
            Some(Self((percent * 1000.0) as u64))
        } else {
            None
        }
    }

    /// Get as percentage.
    pub fn as_percent(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Get raw value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Default for CpuLimit {
    fn default() -> Self {
        Self::NONE
    }
}

/// CPU reservation (0-100000, representing 0-100%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CpuReservation(u64);

impl CpuReservation {
    /// No reservation.
    pub const NONE: Self = Self(0);

    /// Create from percentage (0.0 - 100.0).
    pub fn from_percent(percent: f64) -> Option<Self> {
        if percent >= 0.0 && percent <= 100.0 {
            Some(Self((percent * 1000.0) as u64))
        } else {
            None
        }
    }

    /// Get as percentage.
    pub fn as_percent(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Get raw value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// CPU weight for relative priority (0-10000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuWeight(u32);

impl CpuWeight {
    /// Default weight (100).
    pub const DEFAULT: Self = Self(100);
    /// Minimum weight.
    pub const MIN: u32 = 0;
    /// Maximum weight.
    pub const MAX: u32 = 10000;

    /// Create a new weight.
    pub fn new(weight: u32) -> Option<Self> {
        if weight <= Self::MAX {
            Some(Self(weight))
        } else {
            None
        }
    }

    /// Get weight value.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl Default for CpuWeight {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// CPU group ID for CPU affinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuGroupId(uuid::Uuid);

impl CpuGroupId {
    /// No CPU group.
    pub fn none() -> Self {
        Self(uuid::Uuid::nil())
    }

    /// Create from GUID string.
    pub fn from_guid(guid: &str) -> Result<Self> {
        uuid::Uuid::parse_str(guid)
            .map(Self)
            .map_err(|_| Error::Validation {
                field: "cpu_group_id",
                message: format!("Invalid GUID: {}", guid),
                failure_type: FailureType::Validation,
            })
    }

    /// Get as GUID string.
    pub fn to_guid(&self) -> String {
        self.0.to_string()
    }

    /// Check if no group assigned.
    pub fn is_none(&self) -> bool {
        self.0.is_nil()
    }
}

impl Default for CpuGroupId {
    fn default() -> Self {
        Self::none()
    }
}

/// Hardware threads per core configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwThreadsPerCore {
    /// Use host default.
    Default,
    /// Specific count (1 = no SMT, 2 = SMT enabled).
    Count(u32),
}

impl Default for HwThreadsPerCore {
    fn default() -> Self {
        Self::Default
    }
}

/// Page shattering mitigation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageShatteringState {
    /// Use default setting.
    #[default]
    Default = 0,
    /// Always enabled.
    AlwaysEnabled = 1,
    /// Always disabled (allows large heap allocations).
    AlwaysDisabled = 2,
}

/// Virtual processors per channel (VPPC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VppcValue(u16);

impl VppcValue {
    /// Default VPPC.
    pub const DEFAULT: Self = Self(0);

    /// Create new VPPC value.
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Get value.
    pub fn get(&self) -> u16 {
        self.0
    }
}

impl Default for VppcValue {
    fn default() -> Self {
        Self::DEFAULT
    }
}
```

### File: `hyperv/src/processor/settings.rs`

```rust
use super::types::*;
use crate::error::{Error, Result, FailureType};
use crate::vm::ProcessorCount;
use crate::wmi::WmiConnection;

/// Processor settings for a VM.
#[derive(Debug, Clone)]
pub struct ProcessorSettings {
    /// Number of virtual processors.
    pub count: ProcessorCount,
    /// CPU limit.
    pub limit: CpuLimit,
    /// CPU reservation.
    pub reservation: CpuReservation,
    /// CPU weight.
    pub weight: CpuWeight,
    /// CPU group ID.
    pub cpu_group_id: CpuGroupId,
    /// Hardware threads per core.
    pub hw_threads_per_core: HwThreadsPerCore,
    /// Enable nested virtualization.
    pub expose_virtualization_extensions: bool,
    /// Enable hierarchical virtualization.
    pub hierarchical_virtualization: bool,
    /// Page shattering mitigation.
    pub page_shattering: PageShatteringState,
    /// Enable socket topology exposure.
    pub enable_socket_topology: bool,
    /// Virtual processors per channel.
    pub vppc: VppcValue,
    /// L3 cache ways allocation (0 = default).
    pub l3_cache_ways: u32,
    /// AMD: Max processors per L3 cache.
    pub max_processors_per_l3: u32,
    /// AMD: Max cluster count per socket.
    pub max_cluster_count_per_socket: u32,
    /// AMD: L3 processor distribution policy.
    pub l3_processor_distribution_policy: u32,
    /// CPU frequency cap in MHz (0 = no cap).
    pub cpu_frequency_cap_mhz: u32,
}

impl Default for ProcessorSettings {
    fn default() -> Self {
        Self {
            count: ProcessorCount::one(),
            limit: CpuLimit::NONE,
            reservation: CpuReservation::NONE,
            weight: CpuWeight::DEFAULT,
            cpu_group_id: CpuGroupId::none(),
            hw_threads_per_core: HwThreadsPerCore::Default,
            expose_virtualization_extensions: false,
            hierarchical_virtualization: false,
            page_shattering: PageShatteringState::Default,
            enable_socket_topology: false,
            vppc: VppcValue::DEFAULT,
            l3_cache_ways: 0,
            max_processors_per_l3: 0,
            max_cluster_count_per_socket: 0,
            l3_processor_distribution_policy: 0,
            cpu_frequency_cap_mhz: 0,
        }
    }
}

/// Builder for processor settings.
#[derive(Debug, Clone, Default)]
pub struct ProcessorSettingsBuilder {
    settings: ProcessorSettings,
}

impl ProcessorSettingsBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set processor count.
    pub fn count(mut self, count: ProcessorCount) -> Self {
        self.settings.count = count;
        self
    }

    /// Set processor count from value.
    pub fn count_value(self, count: u32) -> Result<Self> {
        let count = ProcessorCount::new(count)
            .ok_or_else(|| Error::Validation {
                field: "processor_count",
                message: format!("Invalid processor count: {}", count),
                failure_type: FailureType::Validation,
            })?;
        Ok(self.count(count))
    }

    /// Set CPU limit.
    pub fn limit(mut self, limit: CpuLimit) -> Self {
        self.settings.limit = limit;
        self
    }

    /// Set CPU limit from percentage.
    pub fn limit_percent(self, percent: f64) -> Result<Self> {
        let limit = CpuLimit::from_percent(percent)
            .ok_or_else(|| Error::Validation {
                field: "cpu_limit",
                message: format!("Invalid CPU limit: {}%", percent),
                failure_type: FailureType::Validation,
            })?;
        Ok(self.limit(limit))
    }

    /// Set CPU reservation.
    pub fn reservation(mut self, reservation: CpuReservation) -> Self {
        self.settings.reservation = reservation;
        self
    }

    /// Set CPU reservation from percentage.
    pub fn reservation_percent(self, percent: f64) -> Result<Self> {
        let reservation = CpuReservation::from_percent(percent)
            .ok_or_else(|| Error::Validation {
                field: "cpu_reservation",
                message: format!("Invalid CPU reservation: {}%", percent),
                failure_type: FailureType::Validation,
            })?;
        Ok(self.reservation(reservation))
    }

    /// Set CPU weight.
    pub fn weight(mut self, weight: CpuWeight) -> Self {
        self.settings.weight = weight;
        self
    }

    /// Set CPU group ID.
    pub fn cpu_group(mut self, group_id: CpuGroupId) -> Self {
        self.settings.cpu_group_id = group_id;
        self
    }

    /// Set CPU group ID from GUID string.
    pub fn cpu_group_guid(self, guid: &str) -> Result<Self> {
        let group_id = CpuGroupId::from_guid(guid)?;
        Ok(self.cpu_group(group_id))
    }

    /// Enable nested virtualization.
    pub fn nested_virtualization(mut self, enabled: bool) -> Self {
        self.settings.expose_virtualization_extensions = enabled;
        self
    }

    /// Enable hierarchical virtualization.
    pub fn hierarchical_virtualization(mut self, enabled: bool) -> Self {
        self.settings.hierarchical_virtualization = enabled;
        self
    }

    /// Set hardware threads per core.
    pub fn hw_threads_per_core(mut self, threads: HwThreadsPerCore) -> Self {
        self.settings.hw_threads_per_core = threads;
        self
    }

    /// Set page shattering mitigation.
    pub fn page_shattering(mut self, state: PageShatteringState) -> Self {
        self.settings.page_shattering = state;
        self
    }

    /// Enable socket topology exposure.
    pub fn socket_topology(mut self, enabled: bool) -> Self {
        self.settings.enable_socket_topology = enabled;
        self
    }

    /// Set AMD CCX configuration.
    pub fn amd_ccx(
        mut self,
        max_per_l3: u32,
        max_clusters_per_socket: u32,
        distribution_policy: u32,
    ) -> Self {
        self.settings.max_processors_per_l3 = max_per_l3;
        self.settings.max_cluster_count_per_socket = max_clusters_per_socket;
        self.settings.l3_processor_distribution_policy = distribution_policy;
        self
    }

    /// Set CPU frequency cap.
    pub fn frequency_cap_mhz(mut self, mhz: u32) -> Self {
        self.settings.cpu_frequency_cap_mhz = mhz;
        self
    }

    /// Build settings.
    pub fn build(self) -> Result<ProcessorSettings> {
        Ok(self.settings)
    }
}

impl ProcessorSettings {
    /// Create a builder.
    pub fn builder() -> ProcessorSettingsBuilder {
        ProcessorSettingsBuilder::new()
    }

    /// Get processor settings for a VM.
    pub fn get(conn: &WmiConnection, vm_id: &str) -> Result<Self> {
        #[cfg(windows)]
        {
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
                 ResultClass=Msvm_ProcessorSettingData",
                vm_id
            );

            let results: Vec<_> = conn.query(&query)?.collect();

            if let Some(obj) = results.first() {
                Ok(ProcessorSettings {
                    count: ProcessorCount::new(
                        obj.get_u32("VirtualQuantity")?.unwrap_or(1)
                    ).unwrap_or(ProcessorCount::one()),
                    limit: CpuLimit(obj.get_u64("Limit")?.unwrap_or(100000)),
                    reservation: CpuReservation(obj.get_u64("Reservation")?.unwrap_or(0)),
                    weight: CpuWeight::new(obj.get_u32("Weight")?.unwrap_or(100))
                        .unwrap_or(CpuWeight::DEFAULT),
                    cpu_group_id: obj.get_string("CpuGroupId")?
                        .and_then(|s| CpuGroupId::from_guid(&s).ok())
                        .unwrap_or_default(),
                    hw_threads_per_core: match obj.get_u32("HwThreadsPerCore")? {
                        Some(0) | None => HwThreadsPerCore::Default,
                        Some(n) => HwThreadsPerCore::Count(n),
                    },
                    expose_virtualization_extensions: obj.get_bool("ExposeVirtualizationExtensions")?.unwrap_or(false),
                    hierarchical_virtualization: obj.get_bool("HierarchicalVirtualization")?.unwrap_or(false),
                    page_shattering: PageShatteringState::Default, // Read separately
                    enable_socket_topology: obj.get_bool("EnableSocketTopology")?.unwrap_or(false),
                    vppc: VppcValue::new(obj.get_u16("VirtualProcessorsPerChannel")?.unwrap_or(0)),
                    l3_cache_ways: obj.get_u32("L3CacheWays")?.unwrap_or(0),
                    max_processors_per_l3: obj.get_u32("MaxProcessorCountPerL3")?.unwrap_or(0),
                    max_cluster_count_per_socket: obj.get_u32("MaxClusterCountPerSocket")?.unwrap_or(0),
                    l3_processor_distribution_policy: obj.get_u32("L3ProcessorDistributionPolicy")?.unwrap_or(0),
                    cpu_frequency_cap_mhz: obj.get_u32("PerfCpuFreqCapMhz")?.unwrap_or(0),
                })
            } else {
                Ok(ProcessorSettings::default())
            }
        }

        #[cfg(not(windows))]
        Ok(ProcessorSettings::default())
    }

    /// Apply processor settings to a VM.
    pub fn apply(&self, conn: &WmiConnection, vm_id: &str) -> Result<()> {
        #[cfg(windows)]
        {
            // Get VSMS
            let vsms_path = conn.get_singleton_path("Msvm_VirtualSystemManagementService")?;

            // Get current processor setting data
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
                 ResultClass=Msvm_ProcessorSettingData",
                vm_id
            );

            let results: Vec<_> = conn.query(&query)?.collect();

            if let Some(proc_data) = results.first() {
                // Modify settings
                proc_data.put_u32("VirtualQuantity", self.count.get())?;
                proc_data.put_u64("Limit", self.limit.raw())?;
                proc_data.put_u64("Reservation", self.reservation.raw())?;
                proc_data.put_u32("Weight", self.weight.get())?;

                if !self.cpu_group_id.is_none() {
                    proc_data.put_string("CpuGroupId", &self.cpu_group_id.to_guid())?;
                }

                proc_data.put_bool("ExposeVirtualizationExtensions",
                    self.expose_virtualization_extensions)?;
                proc_data.put_bool("EnableSocketTopology", self.enable_socket_topology)?;

                // AMD CCX settings
                if self.max_processors_per_l3 > 0 {
                    proc_data.put_u32("MaxProcessorCountPerL3", self.max_processors_per_l3)?;
                }
                if self.max_cluster_count_per_socket > 0 {
                    proc_data.put_u32("MaxClusterCountPerSocket", self.max_cluster_count_per_socket)?;
                }
                if self.cpu_frequency_cap_mhz > 0 {
                    proc_data.put_u32("PerfCpuFreqCapMhz", self.cpu_frequency_cap_mhz)?;
                }

                // Apply via ModifyResourceSettings
                conn.exec_method(&vsms_path, "ModifyResourceSettings", |params| {
                    params.put_string_array("ResourceSettings", &[&proc_data.get_text()?])?;
                    Ok(())
                })?;
            }

            Ok(())
        }

        #[cfg(not(windows))]
        Ok(())
    }
}
```

---

## 7. Validation Module

### File: `hyperv/src/validation/mod.rs`

```rust
//! Property validation and capability checking.
//!
//! Provides:
//! - Property existence checks
//! - VM version validation
//! - Feature capability detection

mod capabilities;
mod version;

pub use capabilities::*;
pub use version::*;
```

### File: `hyperv/src/validation/capabilities.rs`

```rust
use crate::error::{Error, Result};
use crate::wmi::WmiConnection;

/// Property support checker.
pub struct PropertySupport<'a> {
    conn: &'a WmiConnection,
}

impl<'a> PropertySupport<'a> {
    /// Create a new property support checker.
    pub fn new(conn: &'a WmiConnection) -> Self {
        Self { conn }
    }

    /// Check if a processor setting property is supported.
    pub fn supports_processor_property(&self, property: &str) -> Result<bool> {
        self.check_class_property("Msvm_ProcessorSettingData", property)
    }

    /// Check if a system setting property is supported.
    pub fn supports_system_property(&self, property: &str) -> Result<bool> {
        self.check_class_property("Msvm_VirtualSystemSettingData", property)
    }

    /// Check if a security setting property is supported.
    pub fn supports_security_property(&self, property: &str) -> Result<bool> {
        self.check_class_property("Msvm_SecuritySettingData", property)
    }

    /// Check if CPU group ID property is supported.
    pub fn supports_cpu_group_id(&self) -> Result<bool> {
        self.supports_processor_property("CpuGroupId")
    }

    /// Check if HW threads per core property is supported.
    pub fn supports_hw_threads_per_core(&self) -> Result<bool> {
        self.supports_processor_property("HwThreadsPerCore")
    }

    /// Check if L3 cache ways property is supported.
    pub fn supports_l3_cache_ways(&self) -> Result<bool> {
        self.supports_processor_property("L3CacheWays")
    }

    /// Check if nested virtualization is supported.
    pub fn supports_nested_virtualization(&self) -> Result<bool> {
        self.supports_processor_property("ExposeVirtualizationExtensions")
    }

    /// Check if socket topology property is supported.
    pub fn supports_socket_topology(&self) -> Result<bool> {
        self.supports_processor_property("EnableSocketTopology")
    }

    /// Check if hierarchical virtualization is supported.
    pub fn supports_hierarchical_virtualization(&self) -> Result<bool> {
        self.supports_processor_property("HierarchicalVirtualization")
    }

    fn check_class_property(&self, class: &str, property: &str) -> Result<bool> {
        #[cfg(windows)]
        {
            let class_def = self.conn.get_class(class)?;
            Ok(class_def.has_property(property))
        }

        #[cfg(not(windows))]
        Ok(false)
    }
}

/// Host capability information.
#[derive(Debug, Clone)]
pub struct HostCapabilities {
    /// Live migration available.
    pub live_migration: bool,
    /// Storage migration available.
    pub storage_migration: bool,
    /// vTPM available.
    pub vtpm: bool,
    /// Shielded VMs available.
    pub shielded_vms: bool,
    /// GPU-P available.
    pub gpu_partition: bool,
    /// DDA available.
    pub dda: bool,
    /// Nested virtualization available.
    pub nested_virtualization: bool,
    /// Maximum VM version supported.
    pub max_vm_version: String,
    /// Supported VM versions.
    pub supported_vm_versions: Vec<String>,
}

impl HostCapabilities {
    /// Query host capabilities.
    pub fn query(conn: &WmiConnection) -> Result<Self> {
        #[cfg(windows)]
        {
            // Get Msvm_VirtualSystemManagementServiceSettingData
            let query = "SELECT * FROM Msvm_VirtualSystemManagementServiceSettingData";
            let results: Vec<_> = conn.query(query)?.collect();

            let (max_version, supported_versions) = if let Some(obj) = results.first() {
                let max = obj.get_string("MaximumMachineSupportedVersion")?.unwrap_or_default();
                let supported = obj.get_string_array("SupportedVirtualMachineVersions")?;
                (max, supported)
            } else {
                (String::new(), Vec::new())
            };

            // Check for migration service
            let migration_available = conn.get_singleton_path("Msvm_VirtualSystemMigrationService").is_ok();

            // Check property support
            let prop_support = PropertySupport::new(conn);

            Ok(HostCapabilities {
                live_migration: migration_available,
                storage_migration: migration_available,
                vtpm: prop_support.supports_security_property("TpmEnabled").unwrap_or(false),
                shielded_vms: prop_support.supports_security_property("ShieldingRequested").unwrap_or(false),
                gpu_partition: conn.get_singleton_path("Msvm_PartitionableGpu").is_ok(),
                dda: conn.get_singleton_path("Msvm_AssignableDeviceService").is_ok(),
                nested_virtualization: prop_support.supports_nested_virtualization().unwrap_or(false),
                max_vm_version: max_version,
                supported_vm_versions: supported_versions,
            })
        }

        #[cfg(not(windows))]
        Ok(HostCapabilities {
            live_migration: false,
            storage_migration: false,
            vtpm: false,
            shielded_vms: false,
            gpu_partition: false,
            dda: false,
            nested_virtualization: false,
            max_vm_version: String::new(),
            supported_vm_versions: Vec::new(),
        })
    }
}
```

### File: `hyperv/src/validation/version.rs`

```rust
use crate::error::{Error, Result, FailureType};
use crate::wmi::WmiConnection;

/// VM version information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VmVersion {
    major: u32,
    minor: u32,
}

impl VmVersion {
    /// Windows Server 2016 / Windows 10 version.
    pub const V8_0: Self = Self { major: 8, minor: 0 };
    /// Windows Server 2019 version.
    pub const V9_0: Self = Self { major: 9, minor: 0 };
    /// Windows Server 2022 version.
    pub const V10_0: Self = Self { major: 10, minor: 0 };
    /// Windows Server 2025 version.
    pub const V11_0: Self = Self { major: 11, minor: 0 };

    /// Parse from version string (e.g., "9.0").
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            Some(Self { major, minor })
        } else {
            None
        }
    }

    /// Format as version string.
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Check if this version supports a feature.
    pub fn supports(&self, required: &VmVersion) -> bool {
        self >= required
    }
}

impl std::fmt::Display for VmVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// VM version validator.
pub struct VmVersionValidator<'a> {
    conn: &'a WmiConnection,
    supported_versions: Vec<VmVersion>,
    default_version: Option<VmVersion>,
}

impl<'a> VmVersionValidator<'a> {
    /// Create a new validator.
    pub fn new(conn: &'a WmiConnection) -> Result<Self> {
        #[cfg(windows)]
        {
            let query = "SELECT * FROM Msvm_VirtualSystemManagementServiceSettingData";
            let results: Vec<_> = conn.query(query)?.collect();

            let (default, supported) = if let Some(obj) = results.first() {
                let default_str = obj.get_string("DefaultVirtualMachinePath")?;
                let supported_strs = obj.get_string_array("SupportedVirtualMachineVersions")?;

                let default = default_str.and_then(|s| VmVersion::parse(&s));
                let supported: Vec<_> = supported_strs
                    .iter()
                    .filter_map(|s| VmVersion::parse(s))
                    .collect();

                (default, supported)
            } else {
                (None, Vec::new())
            };

            Ok(Self {
                conn,
                supported_versions: supported,
                default_version: default,
            })
        }

        #[cfg(not(windows))]
        Ok(Self {
            conn,
            supported_versions: Vec::new(),
            default_version: None,
        })
    }

    /// Check if a version is supported.
    pub fn is_supported(&self, version: &VmVersion) -> bool {
        self.supported_versions.contains(version)
    }

    /// Get the default VM version.
    pub fn default_version(&self) -> Option<&VmVersion> {
        self.default_version.as_ref()
    }

    /// Get all supported versions.
    pub fn supported_versions(&self) -> &[VmVersion] {
        &self.supported_versions
    }

    /// Validate that a version is supported, returning error if not.
    pub fn validate(&self, version: &str) -> Result<VmVersion> {
        let ver = VmVersion::parse(version)
            .ok_or_else(|| Error::Validation {
                field: "vm_version",
                message: format!("Invalid version format: {}", version),
                failure_type: FailureType::Validation,
            })?;

        if !self.is_supported(&ver) {
            return Err(Error::UnsupportedVmVersion {
                version: version.to_string(),
                supported_versions: self.supported_versions
                    .iter()
                    .map(|v| v.to_string())
                    .collect(),
            });
        }

        Ok(ver)
    }

    /// Get minimum version required for a feature.
    pub fn min_version_for_feature(&self, feature: &str) -> Option<VmVersion> {
        match feature {
            "nested_virtualization" => Some(VmVersion::V8_0),
            "vtpm" => Some(VmVersion::V8_0),
            "shielded_vm" => Some(VmVersion::V8_0),
            "gpu_partition" => Some(VmVersion::V9_0),
            "socket_topology" => Some(VmVersion::V10_0),
            _ => None,
        }
    }
}
```

---

## 8. KVP (Key-Value Pair) Module

### File: `hyperv/src/kvp/mod.rs`

```rust
//! KVP (Key-Value Pair) exchange for guest-host communication.

mod exchange;
mod types;

pub use exchange::KvpExchange;
pub use types::*;
```

### File: `hyperv/src/kvp/types.rs`

```rust
/// KVP item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvpItem {
    /// Key name.
    pub name: String,
    /// Value data.
    pub data: String,
}

impl KvpItem {
    /// Create a new KVP item.
    pub fn new(name: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
        }
    }
}

/// KVP source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvpSource {
    /// Host-only items (set by host, visible to guest).
    HostOnly,
    /// Guest items (set by guest, visible to host).
    Guest,
    /// External items.
    External,
}
```

### File: `hyperv/src/kvp/exchange.rs`

```rust
use super::{KvpItem, KvpSource};
use crate::error::{Error, Result};
use crate::wmi::WmiConnection;

/// KVP exchange operations.
pub struct KvpExchange<'a> {
    conn: &'a WmiConnection,
    vm_id: String,
}

impl<'a> KvpExchange<'a> {
    /// Create KVP exchange for a VM.
    pub fn new(conn: &'a WmiConnection, vm_id: impl Into<String>) -> Self {
        Self {
            conn,
            vm_id: vm_id.into(),
        }
    }

    /// Get host-only KVP items.
    pub fn get_host_only_items(&self) -> Result<Vec<KvpItem>> {
        self.get_items(KvpSource::HostOnly)
    }

    /// Get guest KVP items.
    pub fn get_guest_items(&self) -> Result<Vec<KvpItem>> {
        self.get_items(KvpSource::Guest)
    }

    /// Add host-only KVP items.
    pub fn add_host_only_items(&self, items: &[KvpItem]) -> Result<()> {
        self.modify_items("AddKvpItems", items)
    }

    /// Modify host-only KVP items.
    pub fn modify_host_only_items(&self, items: &[KvpItem]) -> Result<()> {
        self.modify_items("ModifyKvpItems", items)
    }

    /// Remove host-only KVP items by key name.
    pub fn remove_host_only_items(&self, keys: &[&str]) -> Result<()> {
        let items: Vec<_> = keys.iter()
            .map(|k| KvpItem::new(*k, ""))
            .collect();
        self.modify_items("RemoveKvpItems", &items)
    }

    fn get_items(&self, source: KvpSource) -> Result<Vec<KvpItem>> {
        #[cfg(windows)]
        {
            let class_name = match source {
                KvpSource::HostOnly => "Msvm_KvpExchangeDataItem",
                KvpSource::Guest => "Msvm_KvpExchangeDataItem",
                KvpSource::External => "Msvm_KvpExchangeDataItem",
            };

            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_KvpExchangeComponentSettingData \
                 ResultClass={}",
                self.vm_id, class_name
            );

            let results: Vec<_> = self.conn.query(&query)?.collect();

            let mut items = Vec::new();
            for obj in results {
                if let (Some(name), Some(data)) = (
                    obj.get_string("Name")?,
                    obj.get_string("Data")?
                ) {
                    items.push(KvpItem { name, data });
                }
            }

            Ok(items)
        }

        #[cfg(not(windows))]
        Ok(Vec::new())
    }

    fn modify_items(&self, method: &str, items: &[KvpItem]) -> Result<()> {
        #[cfg(windows)]
        {
            let vsms_path = self.conn.get_singleton_path("Msvm_VirtualSystemManagementService")?;
            let vm_path = self.conn.get_vm_path_by_id(&self.vm_id)?;

            // Build KVP data items
            let mut data_items = Vec::new();
            for item in items {
                let kvp_class = self.conn.get_class("Msvm_KvpExchangeDataItem")?;
                let kvp_obj = kvp_class.spawn_instance()?;
                kvp_obj.put_string("Name", &item.name)?;
                kvp_obj.put_string("Data", &item.data)?;
                kvp_obj.put_u16("Source", 0)?; // 0 = Host
                data_items.push(kvp_obj.get_text()?);
            }

            self.conn.exec_method(&vsms_path, method, |params| {
                params.put_reference("TargetSystem", &vm_path)?;
                params.put_string_array("DataItems",
                    &data_items.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
                Ok(())
            })?;

            Ok(())
        }

        #[cfg(not(windows))]
        Ok(())
    }
}
```

---

## 9. Serial Console Module

### File: `hyperv/src/serial/mod.rs`

```rust
//! Serial console management for Hyper-V VMs.

mod port;
mod types;

pub use port::SerialPort;
pub use types::*;
```

### File: `hyperv/src/serial/types.rs`

```rust
/// Console mode for VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMode {
    /// None - serial console disabled.
    None = 3,
    /// COM1 serial port.
    Com1 = 1,
    /// COM2 serial port.
    Com2 = 2,
}

impl From<u16> for ConsoleMode {
    fn from(value: u16) -> Self {
        match value {
            1 => ConsoleMode::Com1,
            2 => ConsoleMode::Com2,
            _ => ConsoleMode::None,
        }
    }
}

/// Serial port connection type.
#[derive(Debug, Clone)]
pub enum SerialPortConnection {
    /// Not connected.
    None,
    /// Named pipe.
    NamedPipe(String),
    /// File output.
    File(String),
    /// Azure serial console.
    Azure,
}
```

### File: `hyperv/src/serial/port.rs`

```rust
use super::{ConsoleMode, SerialPortConnection};
use crate::error::{Error, Result};
use crate::wmi::WmiConnection;

/// Serial port operations.
pub struct SerialPort<'a> {
    conn: &'a WmiConnection,
    vm_id: String,
}

impl<'a> SerialPort<'a> {
    /// Create serial port manager for a VM.
    pub fn new(conn: &'a WmiConnection, vm_id: impl Into<String>) -> Self {
        Self {
            conn,
            vm_id: vm_id.into(),
        }
    }

    /// Enable serial console.
    pub fn enable(&self, enabled: bool) -> Result<()> {
        self.set_console_mode(if enabled { ConsoleMode::Com1 } else { ConsoleMode::None })
    }

    /// Set console mode.
    pub fn set_console_mode(&self, mode: ConsoleMode) -> Result<()> {
        #[cfg(windows)]
        {
            let vsms_path = self.conn.get_singleton_path("Msvm_VirtualSystemManagementService")?;

            // Get VM settings
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_SettingsDefineState \
                 ResultClass=Msvm_VirtualSystemSettingData",
                self.vm_id
            );

            let results: Vec<_> = self.conn.query(&query)?.collect();

            if let Some(settings) = results.first() {
                settings.put_u16("ConsoleMode", mode as u16)?;

                self.conn.exec_method(&vsms_path, "ModifySystemSettings", |params| {
                    params.put_string("SystemSettings", &settings.get_text()?)?;
                    Ok(())
                })?;
            }

            Ok(())
        }

        #[cfg(not(windows))]
        Ok(())
    }

    /// Get current console mode.
    pub fn get_console_mode(&self) -> Result<ConsoleMode> {
        #[cfg(windows)]
        {
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_SettingsDefineState \
                 ResultClass=Msvm_VirtualSystemSettingData",
                self.vm_id
            );

            let results: Vec<_> = self.conn.query(&query)?.collect();

            if let Some(settings) = results.first() {
                let mode = settings.get_u16("ConsoleMode")?.unwrap_or(3);
                return Ok(ConsoleMode::from(mode));
            }

            Ok(ConsoleMode::None)
        }

        #[cfg(not(windows))]
        Ok(ConsoleMode::None)
    }

    /// Connect serial port to named pipe.
    pub fn connect_to_pipe(&self, port: u32, pipe_name: &str) -> Result<()> {
        self.set_connection(port, SerialPortConnection::NamedPipe(pipe_name.to_string()))
    }

    /// Connect serial port to file.
    pub fn connect_to_file(&self, port: u32, file_path: &str) -> Result<()> {
        self.set_connection(port, SerialPortConnection::File(file_path.to_string()))
    }

    /// Disconnect serial port.
    pub fn disconnect(&self, port: u32) -> Result<()> {
        self.set_connection(port, SerialPortConnection::None)
    }

    fn set_connection(&self, port: u32, connection: SerialPortConnection) -> Result<()> {
        #[cfg(windows)]
        {
            let vsms_path = self.conn.get_singleton_path("Msvm_VirtualSystemManagementService")?;

            // Get serial port setting data
            let query = format!(
                "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
                 WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
                 ResultClass=Msvm_SerialPortSettingData",
                self.vm_id
            );

            let results: Vec<_> = self.conn.query(&query)?.collect();

            // Find the matching port
            for port_data in results {
                // Check if this is the right port (based on InstanceID or similar)
                let connection_str = match &connection {
                    SerialPortConnection::None => String::new(),
                    SerialPortConnection::NamedPipe(name) => name.clone(),
                    SerialPortConnection::File(path) => path.clone(),
                    SerialPortConnection::Azure => "Azure".to_string(),
                };

                port_data.put_string("Connection", &connection_str)?;

                self.conn.exec_method(&vsms_path, "ModifyResourceSettings", |params| {
                    params.put_string_array("ResourceSettings", &[&port_data.get_text()?])?;
                    Ok(())
                })?;

                break;
            }

            Ok(())
        }

        #[cfg(not(windows))]
        Ok(())
    }
}
```

---

## 10. Export/Import Module

### File: `hyperv/src/export_import/mod.rs`

```rust
//! Export and import operations for Hyper-V VMs.

mod export;
mod import;
mod planned_vm;
mod types;

pub use export::ExportService;
pub use import::ImportService;
pub use planned_vm::PlannedVm;
pub use types::*;
```

### File: `hyperv/src/export_import/types.rs`

```rust
use std::path::PathBuf;

/// Export type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    /// Full export (configuration + data).
    Full,
    /// Configuration only.
    ConfigOnly,
    /// For live migration.
    ForLiveMigration,
    /// Without runtime info (for RequestCustomRestore).
    WithoutRuntimeInfo,
    /// With runtime info (for saved VM export).
    WithRuntimeInfo,
}

/// Export options.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export type.
    pub export_type: ExportType,
    /// Destination directory.
    pub destination_path: PathBuf,
    /// Create subfolder with VM name.
    pub create_vm_subfolder: bool,
    /// Copy snapshots.
    pub copy_snapshots: bool,
    /// Copy VHDs.
    pub copy_vhds: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            export_type: ExportType::Full,
            destination_path: PathBuf::new(),
            create_vm_subfolder: true,
            copy_snapshots: true,
            copy_vhds: true,
        }
    }
}

/// Import options.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Source configuration path.
    pub config_path: PathBuf,
    /// VM folder path (destination).
    pub vm_folder_path: Option<PathBuf>,
    /// New VM name (None = keep original).
    pub new_vm_name: Option<String>,
    /// Retain original VM ID.
    pub retain_vm_id: bool,
    /// Generate new VM ID.
    pub generate_new_id: bool,
    /// Copy VHDs to new location.
    pub copy_vhds: bool,
    /// VHD destination path.
    pub vhd_destination_path: Option<PathBuf>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            config_path: PathBuf::new(),
            vm_folder_path: None,
            new_vm_name: None,
            retain_vm_id: false,
            generate_new_id: true,
            copy_vhds: false,
            vhd_destination_path: None,
        }
    }
}
```

### File: `hyperv/src/export_import/export.rs`

```rust
use super::{ExportOptions, ExportType};
use crate::error::{Error, Result};
use crate::wmi::WmiConnection;
use crate::wmi::job::{JobOptions, wait_for_job};

/// Export service for VM exports.
pub struct ExportService<'a> {
    conn: &'a WmiConnection,
    vsms_path: String,
}

impl<'a> ExportService<'a> {
    /// Create a new export service.
    pub fn new(conn: &'a WmiConnection) -> Result<Self> {
        let vsms_path = conn.get_singleton_path("Msvm_VirtualSystemManagementService")?;
        Ok(Self { conn, vsms_path })
    }

    /// Export a VM.
    pub fn export_vm(&self, vm_name: &str, options: &ExportOptions) -> Result<()> {
        #[cfg(windows)]
        {
            let vm_path = self.conn.get_vm_path(vm_name)?;
            let dest_path = options.destination_path.to_string_lossy();

            // Build export setting data
            let export_setting = self.build_export_setting_data(options)?;

            let result = self.conn.exec_method(&self.vsms_path, "ExportSystemDefinition", |params| {
                params.put_reference("ComputerSystem", &vm_path)?;
                params.put_string("ExportDirectory", &dest_path)?;
                params.put_string("ExportSettingData", &export_setting)?;
                Ok(())
            })?;

            let return_value = result.get_u32("ReturnValue")?.unwrap_or(0);

            match return_value {
                0 => Ok(()),
                4096 => {
                    let job_path = result.get_reference("Job")?;
                    wait_for_job(
                        self.conn,
                        &job_path,
                        "ExportSystemDefinition",
                        &JobOptions::default(),
                    )?;
                    Ok(())
                }
                code => Err(Error::OperationFailed {
                    operation: "ExportSystemDefinition",
                    return_value: code,
                    message: format!("Export failed with code {}", code),
                    failure_type: crate::error::FailureType::Permanent,
                }),
            }
        }

        #[cfg(not(windows))]
        Ok(())
    }

    /// Export VM configuration only.
    pub fn export_config(&self, vm_name: &str, destination: &std::path::Path) -> Result<()> {
        let options = ExportOptions {
            export_type: ExportType::ConfigOnly,
            destination_path: destination.to_path_buf(),
            copy_vhds: false,
            copy_snapshots: false,
            ..Default::default()
        };
        self.export_vm(vm_name, &options)
    }

    /// Export for live migration.
    pub fn export_for_migration(&self, vm_name: &str, destination: &std::path::Path) -> Result<()> {
        let options = ExportOptions {
            export_type: ExportType::ForLiveMigration,
            destination_path: destination.to_path_buf(),
            ..Default::default()
        };
        self.export_vm(vm_name, &options)
    }

    #[cfg(windows)]
    fn build_export_setting_data(&self, options: &ExportOptions) -> Result<String> {
        let export_class = self.conn.get_class("Msvm_VirtualSystemExportSettingData")?;
        let export_obj = export_class.spawn_instance()?;

        // Set export type
        let copy_vm_storage = matches!(
            options.export_type,
            ExportType::Full | ExportType::WithRuntimeInfo
        );
        let copy_vm_runtime = matches!(
            options.export_type,
            ExportType::Full | ExportType::WithRuntimeInfo | ExportType::ForLiveMigration
        );
        let copy_snapshots = options.copy_snapshots;

        export_obj.put_bool("CopyVmStorage", copy_vm_storage)?;
        export_obj.put_bool("CopyVmRuntimeInformation", copy_vm_runtime)?;
        export_obj.put_bool("CopySnapshotConfiguration", copy_snapshots)?;
        export_obj.put_bool("CreateVmExportSubdirectory", options.create_vm_subfolder)?;

        export_obj.get_text()
    }
}
```

---

## Implementation Priority and Dependencies

### Phase 1: Core Infrastructure (Week 1-2)
1. **Error types** - Extended error handling
2. **WMI Connection** - Remote support, connection pooling
3. **Job handling** - Timeout support, progress callbacks

### Phase 2: Essential Features (Week 3-4)
1. **Validation module** - Property and version validation
2. **Processor settings** - Advanced CPU configuration
3. **Security settings** - TPM, Secure Boot

### Phase 3: Migration Support (Week 5-6)
1. **Migration service** - Live/quick migration
2. **Export/Import** - VM export/import
3. **Planned VM** - For brownout scenarios

### Phase 4: Auxiliary Features (Week 7-8)
1. **KVP exchange** - Guest-host communication
2. **Serial console** - Console access
3. **GPU module** - GPU-P and DDA improvements
4. **Network extensions** - Switch ports, extensions

### Dependencies Graph

```
error.rs (base)
    └─> wmi/connection.rs
        └─> wmi/job.rs
            └─> validation/mod.rs
                ├─> processor/mod.rs
                ├─> security/mod.rs
                └─> migration/mod.rs
                    └─> export_import/mod.rs

network/mod.rs
    └─> network/port.rs
        └─> network/extension.rs

kvp/mod.rs (standalone)
serial/mod.rs (standalone)
```

---

## Testing Strategy

Each module should include:

1. **Unit tests** - Schema parsing, validation logic
2. **Integration tests** - Require Hyper-V, feature-gated
3. **Mock tests** - WMI response mocking for CI

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_settings_validation() {
        // Unit test - no WMI needed
        let settings = MigrationSettings::builder()
            .destination_host("server1")
            .live()
            .build();

        assert!(settings.is_ok());
    }

    #[test]
    fn test_missing_destination_host() {
        let settings = MigrationSettings::builder()
            .live()
            .build();

        assert!(matches!(settings, Err(Error::MissingRequired("destination_host"))));
    }
}

#[cfg(all(test, windows, feature = "integration"))]
mod integration_tests {
    use super::*;

    #[test]
    fn test_migration_capability() {
        let conn = WmiConnection::connect_local().unwrap();
        let service = MigrationService::new(&conn).unwrap();
        let cap = service.get_capability().unwrap();

        assert!(cap.live_migration_available);
    }
}
```

---

This plan provides a comprehensive roadmap for implementing feature parity with the C++ wmiv2 library. Each module includes explicit schema definitions, type-safe API signatures, and validation rules.
