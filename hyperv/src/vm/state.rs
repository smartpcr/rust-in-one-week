use core::fmt;

/// VM enabled state (Msvm_ComputerSystem.EnabledState).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum VmState {
    /// Unknown state.
    Unknown = 0,
    /// VM is running.
    Running = 2,
    /// VM is powered off.
    Off = 3,
    /// VM is in the process of shutting down.
    ShuttingDown = 4,
    /// Not applicable.
    NotApplicable = 5,
    /// VM is disabled.
    Disabled = 6,
    /// VM is paused.
    Paused = 32768,
    /// VM is suspended/saved.
    Suspended = 32769,
    /// VM is starting.
    Starting = 32770,
    /// VM is in a saved snapshot state.
    Snapshotting = 32771,
    /// VM is saving state.
    Saving = 32773,
    /// VM is stopping.
    Stopping = 32774,
    /// VM is pausing.
    Pausing = 32776,
    /// VM is resuming.
    Resuming = 32777,
    /// VM is saved/hibernated (WMI v2 state = 32779).
    /// This maps to EnabledStateSaved in WMI v2.
    SavedV2 = 32779,
    /// VM is hibernated (S4 power state, WMI v2 state = 32783).
    Hibernated = 32783,
}

impl VmState {
    /// Parse from WMI EnabledState value.
    pub fn from_enabled_state(value: u16) -> Self {
        match value {
            2 => VmState::Running,
            3 => VmState::Off,
            4 => VmState::ShuttingDown,
            5 => VmState::NotApplicable,
            6 => VmState::Disabled,
            32768 => VmState::Paused,
            32769 => VmState::Suspended,
            32770 => VmState::Starting,
            32771 => VmState::Snapshotting,
            32773 => VmState::Saving,
            32774 => VmState::Stopping,
            32776 => VmState::Pausing,
            32777 => VmState::Resuming,
            32779 => VmState::SavedV2,
            32783 => VmState::Hibernated,
            _ => VmState::Unknown,
        }
    }

    /// Check if VM can be started.
    pub fn can_start(&self) -> bool {
        matches!(
            self,
            VmState::Off
                | VmState::Suspended
                | VmState::Paused
                | VmState::SavedV2
                | VmState::Hibernated
        )
    }

    /// Check if VM can be stopped.
    pub fn can_stop(&self) -> bool {
        matches!(
            self,
            VmState::Running | VmState::Paused | VmState::Suspended
        )
    }

    /// Check if VM can be paused.
    pub fn can_pause(&self) -> bool {
        matches!(self, VmState::Running)
    }

    /// Check if VM can be saved.
    pub fn can_save(&self) -> bool {
        matches!(self, VmState::Running | VmState::Paused)
    }

    /// Check if VM can be hibernated (S4 power state).
    ///
    /// Hibernate requires the VM to be running and have hibernate enabled in VM settings.
    pub fn can_hibernate(&self) -> bool {
        matches!(self, VmState::Running)
    }

    /// Check if VM is in a transitional state.
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            VmState::Starting
                | VmState::Stopping
                | VmState::Saving
                | VmState::Pausing
                | VmState::Resuming
                | VmState::ShuttingDown
                | VmState::Snapshotting
        )
    }

    /// Check if VM is in a saved/hibernated state.
    pub fn is_saved(&self) -> bool {
        matches!(
            self,
            VmState::Suspended | VmState::SavedV2 | VmState::Hibernated
        )
    }
}

impl fmt::Display for VmState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VmState::Unknown => "Unknown",
            VmState::Running => "Running",
            VmState::Off => "Off",
            VmState::ShuttingDown => "Shutting Down",
            VmState::NotApplicable => "Not Applicable",
            VmState::Disabled => "Disabled",
            VmState::Paused => "Paused",
            VmState::Suspended => "Saved",
            VmState::Starting => "Starting",
            VmState::Snapshotting => "Taking Snapshot",
            VmState::Saving => "Saving",
            VmState::Stopping => "Stopping",
            VmState::Pausing => "Pausing",
            VmState::Resuming => "Resuming",
            VmState::SavedV2 => "Saved",
            VmState::Hibernated => "Hibernated",
        };
        write!(f, "{}", s)
    }
}

/// VM generation (Gen1 = BIOS, Gen2 = UEFI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Generation {
    /// Generation 1 VM (BIOS-based, IDE boot).
    #[default]
    Gen1,
    /// Generation 2 VM (UEFI-based, Secure Boot capable).
    Gen2,
}

impl Generation {
    /// Get the WMI VirtualSystemSubType value.
    pub fn to_subtype(&self) -> &'static str {
        match self {
            Generation::Gen1 => "Microsoft:Hyper-V:SubType:1",
            Generation::Gen2 => "Microsoft:Hyper-V:SubType:2",
        }
    }

    /// Parse from WMI VirtualSystemSubType value.
    pub fn from_subtype(subtype: &str) -> Self {
        if subtype.contains(":2") {
            Generation::Gen2
        } else {
            Generation::Gen1
        }
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Generation::Gen1 => write!(f, "Generation 1"),
            Generation::Gen2 => write!(f, "Generation 2"),
        }
    }
}

/// VM operational status (primary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum OperationalStatus {
    Unknown = 0,
    Ok = 2,
    Degraded = 3,
    Stressed = 4,
    PredictiveFailure = 5,
    Error = 6,
    NonRecoverableError = 7,
    Starting = 8,
    Stopping = 9,
    Stopped = 10,
    InService = 11,
    NoContact = 12,
    LostCommunication = 13,
    Aborted = 14,
    Dormant = 15,
    SupportingEntity = 16,
    Completed = 17,
    PowerMode = 18,
    ProtocolVersionMismatch = 32775,
    ApplicationCriticalState = 32782,
    CommunicationTimedOut = 32783,
    CommunicationFailed = 32784,
}

/// VM operational status (secondary).
///
/// The secondary operational status provides additional context about
/// ongoing operations on the VM, particularly during migrations and snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum OperationalStatusSecondary {
    /// No secondary status.
    None = 0,
    /// Creating a snapshot.
    CreatingSnapshot = 32768,
    /// Applying a snapshot.
    ApplyingSnapshot = 32769,
    /// Deleting a snapshot.
    DeletingSnapshot = 32770,
    /// Waiting to start.
    WaitingToStart = 32771,
    /// Merging disks (after snapshot delete).
    MergingDisks = 32772,
    /// Exporting VM.
    ExportingVm = 32773,
    /// VM is being migrated (live migration).
    MigratingVm = 32774,
    /// VM is being migrated to suspended state.
    MigratingVmToSuspended = 32796,
}

impl OperationalStatusSecondary {
    pub fn from_value(value: u16) -> Self {
        match value {
            32768 => OperationalStatusSecondary::CreatingSnapshot,
            32769 => OperationalStatusSecondary::ApplyingSnapshot,
            32770 => OperationalStatusSecondary::DeletingSnapshot,
            32771 => OperationalStatusSecondary::WaitingToStart,
            32772 => OperationalStatusSecondary::MergingDisks,
            32773 => OperationalStatusSecondary::ExportingVm,
            32774 => OperationalStatusSecondary::MigratingVm,
            32796 => OperationalStatusSecondary::MigratingVmToSuspended,
            _ => OperationalStatusSecondary::None,
        }
    }

    /// Check if the VM is currently being migrated.
    pub fn is_migrating(&self) -> bool {
        matches!(
            self,
            OperationalStatusSecondary::MigratingVm
                | OperationalStatusSecondary::MigratingVmToSuspended
        )
    }
}

impl OperationalStatus {
    pub fn from_value(value: u16) -> Self {
        match value {
            2 => OperationalStatus::Ok,
            3 => OperationalStatus::Degraded,
            4 => OperationalStatus::Stressed,
            5 => OperationalStatus::PredictiveFailure,
            6 => OperationalStatus::Error,
            7 => OperationalStatus::NonRecoverableError,
            8 => OperationalStatus::Starting,
            9 => OperationalStatus::Stopping,
            10 => OperationalStatus::Stopped,
            11 => OperationalStatus::InService,
            12 => OperationalStatus::NoContact,
            13 => OperationalStatus::LostCommunication,
            14 => OperationalStatus::Aborted,
            15 => OperationalStatus::Dormant,
            16 => OperationalStatus::SupportingEntity,
            17 => OperationalStatus::Completed,
            18 => OperationalStatus::PowerMode,
            32775 => OperationalStatus::ProtocolVersionMismatch,
            32782 => OperationalStatus::ApplicationCriticalState,
            32783 => OperationalStatus::CommunicationTimedOut,
            32784 => OperationalStatus::CommunicationFailed,
            _ => OperationalStatus::Unknown,
        }
    }
}

/// Requested state for VM state change operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RequestedState {
    /// Start the VM.
    Running = 2,
    /// Power off the VM (hard stop).
    Off = 3,
    /// Pause the VM.
    Paused = 32768,
    /// Save (suspend) the VM.
    Saved = 32769,
    /// Reset the VM.
    Reset = 11,
    /// Hibernate the VM (S4 power state).
    /// This triggers a guest-initiated hibernate/save to disk.
    Hibernated = 32783,
}

/// Shutdown type for graceful shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownType {
    /// Graceful shutdown through guest integration services.
    Graceful,
    /// Force power off.
    Force,
    /// Graceful shutdown, fall back to force if needed.
    GracefulWithForce,
}

/// Checkpoint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckpointType {
    /// Disabled - no checkpoints.
    Disabled,
    /// Production checkpoint (application-consistent, preferred).
    #[default]
    Production,
    /// Production checkpoint only (fail if not possible).
    ProductionOnly,
    /// Standard checkpoint (crash-consistent).
    Standard,
}

impl CheckpointType {
    pub fn to_value(&self) -> u16 {
        match self {
            CheckpointType::Disabled => 0,
            CheckpointType::Production => 1,
            CheckpointType::ProductionOnly => 2,
            CheckpointType::Standard => 3,
        }
    }

    pub fn from_value(value: u16) -> Self {
        match value {
            0 => CheckpointType::Disabled,
            1 => CheckpointType::Production,
            2 => CheckpointType::ProductionOnly,
            3 => CheckpointType::Standard,
            _ => CheckpointType::Production,
        }
    }
}

/// Automatic start action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomaticStartAction {
    /// Do nothing on host start.
    #[default]
    Nothing,
    /// Automatically start if VM was running.
    StartIfRunning,
    /// Always start the VM.
    AlwaysStart,
}

impl AutomaticStartAction {
    pub fn to_value(&self) -> u16 {
        match self {
            AutomaticStartAction::Nothing => 0,
            AutomaticStartAction::StartIfRunning => 1,
            AutomaticStartAction::AlwaysStart => 2,
        }
    }

    pub fn from_value(value: u16) -> Self {
        match value {
            0 => AutomaticStartAction::Nothing,
            1 => AutomaticStartAction::StartIfRunning,
            2 => AutomaticStartAction::AlwaysStart,
            _ => AutomaticStartAction::Nothing,
        }
    }
}

/// Startup delay for automatic VM start.
///
/// Represents a time interval for delaying VM startup after host boot.
/// Maximum delay is 24 hours (86400 seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StartupDelay(u32);

impl StartupDelay {
    /// Maximum delay in seconds (24 hours).
    pub const MAX_SECONDS: u32 = 86400;

    /// Create a new startup delay from seconds.
    ///
    /// Returns `None` if the delay exceeds 24 hours.
    pub fn from_secs(seconds: u32) -> Option<Self> {
        if seconds <= Self::MAX_SECONDS {
            Some(Self(seconds))
        } else {
            None
        }
    }

    /// Create a startup delay from minutes.
    ///
    /// Returns `None` if the delay exceeds 24 hours.
    pub fn from_mins(minutes: u32) -> Option<Self> {
        Self::from_secs(minutes.saturating_mul(60))
    }

    /// Create a startup delay from hours.
    ///
    /// Returns `None` if the delay exceeds 24 hours.
    pub fn from_hours(hours: u32) -> Option<Self> {
        Self::from_secs(hours.saturating_mul(3600))
    }

    /// No delay (immediate start).
    pub const fn none() -> Self {
        Self(0)
    }

    /// Get the delay in seconds.
    pub fn as_secs(&self) -> u32 {
        self.0
    }

    /// Check if there is no delay.
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    /// Convert to CIM datetime interval format.
    ///
    /// CIM datetime interval format: `DDDDDDDDHHMMSS.MMMMMM:000`
    /// where D=days, H=hours, M=minutes, S=seconds, M=microseconds.
    pub fn to_cim_interval(&self) -> String {
        if self.0 == 0 {
            return String::new();
        }
        let hours = self.0 / 3600;
        let minutes = (self.0 % 3600) / 60;
        let seconds = self.0 % 60;
        format!(
            "00000000{:02}{:02}{:02}.000000:000",
            hours, minutes, seconds
        )
    }

    /// Parse from CIM datetime interval format.
    pub fn from_cim_interval(s: &str) -> Option<Self> {
        if s.is_empty() {
            return Some(Self(0));
        }
        // Format: DDDDDDDDHHMMSS.MMMMMM:000
        if s.len() < 14 {
            return None;
        }
        let hours: u32 = s.get(8..10)?.parse().ok()?;
        let minutes: u32 = s.get(10..12)?.parse().ok()?;
        let seconds: u32 = s.get(12..14)?.parse().ok()?;
        Self::from_secs(hours * 3600 + minutes * 60 + seconds)
    }
}

impl fmt::Display for StartupDelay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "no delay")
        } else if self.0 < 60 {
            write!(f, "{} seconds", self.0)
        } else if self.0 < 3600 {
            write!(f, "{} minutes", self.0 / 60)
        } else {
            write!(
                f,
                "{} hours {} minutes",
                self.0 / 3600,
                (self.0 % 3600) / 60
            )
        }
    }
}

use crate::error::VmStateError;

impl VmState {
    /// Convert to VmStateError for error reporting.
    pub fn to_error(&self) -> VmStateError {
        match self {
            VmState::Unknown => VmStateError::Unknown,
            VmState::Running => VmStateError::Running,
            VmState::Off => VmStateError::Off,
            VmState::ShuttingDown => VmStateError::ShuttingDown,
            VmState::Paused => VmStateError::Paused,
            VmState::Suspended | VmState::SavedV2 => VmStateError::Suspended,
            VmState::Starting => VmStateError::Starting,
            VmState::Stopping => VmStateError::Stopping,
            VmState::Hibernated => VmStateError::Hibernated,
            _ => VmStateError::Other(*self as u16),
        }
    }
}

/// Automatic stop action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomaticStopAction {
    /// Turn off the VM.
    TurnOff,
    /// Save the VM state.
    #[default]
    Save,
    /// Graceful shutdown.
    Shutdown,
}

impl AutomaticStopAction {
    pub fn to_value(&self) -> u16 {
        match self {
            AutomaticStopAction::TurnOff => 0,
            AutomaticStopAction::Save => 1,
            AutomaticStopAction::Shutdown => 2,
        }
    }

    pub fn from_value(value: u16) -> Self {
        match value {
            0 => AutomaticStopAction::TurnOff,
            1 => AutomaticStopAction::Save,
            2 => AutomaticStopAction::Shutdown,
            _ => AutomaticStopAction::Save,
        }
    }
}

/// Snapshot configuration for VM export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapshotExportMode {
    /// Export all snapshots.
    #[default]
    AllSnapshots,
    /// Export no snapshots (config only).
    NoSnapshots,
    /// Export only the snapshot subtree.
    SubtreeOnly,
}

impl SnapshotExportMode {
    pub fn to_value(&self) -> u8 {
        match self {
            SnapshotExportMode::AllSnapshots => 0,
            SnapshotExportMode::NoSnapshots => 1,
            SnapshotExportMode::SubtreeOnly => 2,
        }
    }
}

/// Capture live state mode for VM export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureLiveState {
    /// Capture crash-consistent state.
    #[default]
    CrashConsistent,
    /// Capture application-consistent state.
    ApplicationConsistent,
    /// No live state capture.
    None,
}

impl CaptureLiveState {
    pub fn to_value(&self) -> u8 {
        match self {
            CaptureLiveState::CrashConsistent => 0,
            CaptureLiveState::ApplicationConsistent => 1,
            CaptureLiveState::None => 2,
        }
    }
}

/// Settings for VM export operation.
///
/// These settings control what is included in the export and how the export is performed.
#[derive(Debug, Clone, Default)]
pub struct ExportSettings {
    /// Whether to copy VM runtime information (memory state, etc.).
    /// Default: true for full export, false for config-only export.
    pub copy_runtime_info: bool,

    /// Whether to copy VM storage (VHDs).
    /// Default: false (export only creates references to existing VHDs).
    pub copy_storage: bool,

    /// Snapshot export configuration.
    pub snapshot_mode: SnapshotExportMode,

    /// Live state capture mode.
    pub capture_live_state: CaptureLiveState,

    /// Whether to create a subdirectory for the VM export.
    /// Default: true.
    pub create_subdirectory: bool,

    /// Whether to export for live migration.
    /// When true, the export can be used for live migration scenarios.
    pub for_live_migration: bool,

    /// Whether to allow overwriting existing export files.
    /// Default: false.
    pub allow_overwrite: bool,
}

impl ExportSettings {
    /// Create settings for a full VM export (including runtime state).
    pub fn full() -> Self {
        Self {
            copy_runtime_info: true,
            copy_storage: false,
            snapshot_mode: SnapshotExportMode::AllSnapshots,
            capture_live_state: CaptureLiveState::default(),
            create_subdirectory: true,
            for_live_migration: false,
            allow_overwrite: false,
        }
    }

    /// Create settings for config-only export (no runtime state).
    pub fn config_only() -> Self {
        Self {
            copy_runtime_info: false,
            copy_storage: false,
            snapshot_mode: SnapshotExportMode::NoSnapshots,
            capture_live_state: CaptureLiveState::CrashConsistent,
            create_subdirectory: true,
            for_live_migration: false,
            allow_overwrite: false,
        }
    }

    /// Create settings for live migration export.
    pub fn for_live_migration() -> Self {
        Self {
            copy_runtime_info: false,
            copy_storage: false,
            snapshot_mode: SnapshotExportMode::NoSnapshots,
            capture_live_state: CaptureLiveState::CrashConsistent,
            create_subdirectory: true,
            for_live_migration: true,
            allow_overwrite: true,
        }
    }

    /// Builder method: set whether to copy storage.
    pub fn with_storage(mut self, copy: bool) -> Self {
        self.copy_storage = copy;
        self
    }

    /// Builder method: set snapshot export mode.
    pub fn with_snapshot_mode(mut self, mode: SnapshotExportMode) -> Self {
        self.snapshot_mode = mode;
        self
    }

    /// Builder method: set whether to allow overwrite.
    pub fn with_overwrite(mut self, allow: bool) -> Self {
        self.allow_overwrite = allow;
        self
    }
}

/// Settings for VM import operation.
#[derive(Debug, Clone, Default)]
pub struct ImportSettings {
    /// Generate a new unique identifier for the imported VM.
    /// If false, retains the original VM's identifier (may conflict if VM already exists).
    pub generate_new_id: bool,

    /// Path to the snapshot folder (relative to import directory).
    /// If None, uses the default "Snapshots" folder.
    pub snapshot_folder: Option<String>,
}

impl ImportSettings {
    /// Create import settings that retain the original VM ID.
    pub fn retain_id() -> Self {
        Self {
            generate_new_id: false,
            snapshot_folder: None,
        }
    }

    /// Create import settings that generate a new VM ID.
    pub fn new_id() -> Self {
        Self {
            generate_new_id: true,
            snapshot_folder: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_state_from_enabled_state() {
        assert_eq!(VmState::from_enabled_state(2), VmState::Running);
        assert_eq!(VmState::from_enabled_state(3), VmState::Off);
        assert_eq!(VmState::from_enabled_state(32768), VmState::Paused);
        assert_eq!(VmState::from_enabled_state(32769), VmState::Suspended);
        assert_eq!(VmState::from_enabled_state(32779), VmState::SavedV2);
        assert_eq!(VmState::from_enabled_state(32783), VmState::Hibernated);
        assert_eq!(VmState::from_enabled_state(999), VmState::Unknown);
    }

    #[test]
    fn test_vm_state_can_start() {
        assert!(VmState::Off.can_start());
        assert!(VmState::Suspended.can_start());
        assert!(VmState::Paused.can_start());
        assert!(VmState::SavedV2.can_start());
        assert!(VmState::Hibernated.can_start());
        assert!(!VmState::Running.can_start());
        assert!(!VmState::Starting.can_start());
    }

    #[test]
    fn test_vm_state_can_hibernate() {
        assert!(VmState::Running.can_hibernate());
        assert!(!VmState::Off.can_hibernate());
        assert!(!VmState::Paused.can_hibernate());
        assert!(!VmState::Hibernated.can_hibernate());
    }

    #[test]
    fn test_vm_state_is_saved() {
        assert!(VmState::Suspended.is_saved());
        assert!(VmState::SavedV2.is_saved());
        assert!(VmState::Hibernated.is_saved());
        assert!(!VmState::Running.is_saved());
        assert!(!VmState::Off.is_saved());
    }

    #[test]
    fn test_vm_state_can_stop() {
        assert!(VmState::Running.can_stop());
        assert!(VmState::Paused.can_stop());
        assert!(VmState::Suspended.can_stop());
        assert!(!VmState::Off.can_stop());
        assert!(!VmState::Stopping.can_stop());
    }

    #[test]
    fn test_vm_state_can_pause() {
        assert!(VmState::Running.can_pause());
        assert!(!VmState::Off.can_pause());
        assert!(!VmState::Paused.can_pause());
    }

    #[test]
    fn test_vm_state_can_save() {
        assert!(VmState::Running.can_save());
        assert!(VmState::Paused.can_save());
        assert!(!VmState::Off.can_save());
        assert!(!VmState::Suspended.can_save());
    }

    #[test]
    fn test_vm_state_is_transitional() {
        assert!(VmState::Starting.is_transitional());
        assert!(VmState::Stopping.is_transitional());
        assert!(VmState::Saving.is_transitional());
        assert!(VmState::Pausing.is_transitional());
        assert!(VmState::Resuming.is_transitional());
        assert!(VmState::ShuttingDown.is_transitional());
        assert!(VmState::Snapshotting.is_transitional());
        assert!(!VmState::Running.is_transitional());
        assert!(!VmState::Off.is_transitional());
    }

    #[test]
    fn test_vm_state_display() {
        assert_eq!(format!("{}", VmState::Running), "Running");
        assert_eq!(format!("{}", VmState::Off), "Off");
        assert_eq!(format!("{}", VmState::Suspended), "Saved");
        assert_eq!(format!("{}", VmState::ShuttingDown), "Shutting Down");
    }

    #[test]
    fn test_generation_to_subtype() {
        assert_eq!(Generation::Gen1.to_subtype(), "Microsoft:Hyper-V:SubType:1");
        assert_eq!(Generation::Gen2.to_subtype(), "Microsoft:Hyper-V:SubType:2");
    }

    #[test]
    fn test_generation_from_subtype() {
        assert_eq!(
            Generation::from_subtype("Microsoft:Hyper-V:SubType:1"),
            Generation::Gen1
        );
        assert_eq!(
            Generation::from_subtype("Microsoft:Hyper-V:SubType:2"),
            Generation::Gen2
        );
        assert_eq!(Generation::from_subtype("unknown"), Generation::Gen1);
    }

    #[test]
    fn test_generation_display() {
        assert_eq!(format!("{}", Generation::Gen1), "Generation 1");
        assert_eq!(format!("{}", Generation::Gen2), "Generation 2");
    }

    #[test]
    fn test_generation_default() {
        assert_eq!(Generation::default(), Generation::Gen1);
    }

    #[test]
    fn test_checkpoint_type_roundtrip() {
        for ct in [
            CheckpointType::Disabled,
            CheckpointType::Production,
            CheckpointType::ProductionOnly,
            CheckpointType::Standard,
        ] {
            assert_eq!(CheckpointType::from_value(ct.to_value()), ct);
        }
    }

    #[test]
    fn test_automatic_start_action_roundtrip() {
        for action in [
            AutomaticStartAction::Nothing,
            AutomaticStartAction::StartIfRunning,
            AutomaticStartAction::AlwaysStart,
        ] {
            assert_eq!(AutomaticStartAction::from_value(action.to_value()), action);
        }
    }

    #[test]
    fn test_automatic_stop_action_roundtrip() {
        for action in [
            AutomaticStopAction::TurnOff,
            AutomaticStopAction::Save,
            AutomaticStopAction::Shutdown,
        ] {
            assert_eq!(AutomaticStopAction::from_value(action.to_value()), action);
        }
    }

    #[test]
    fn test_operational_status_from_value() {
        assert_eq!(OperationalStatus::from_value(2), OperationalStatus::Ok);
        assert_eq!(OperationalStatus::from_value(6), OperationalStatus::Error);
        assert_eq!(
            OperationalStatus::from_value(999),
            OperationalStatus::Unknown
        );
    }

    #[test]
    fn test_operational_status_secondary_from_value() {
        assert_eq!(
            OperationalStatusSecondary::from_value(0),
            OperationalStatusSecondary::None
        );
        assert_eq!(
            OperationalStatusSecondary::from_value(32768),
            OperationalStatusSecondary::CreatingSnapshot
        );
        assert_eq!(
            OperationalStatusSecondary::from_value(32774),
            OperationalStatusSecondary::MigratingVm
        );
        assert_eq!(
            OperationalStatusSecondary::from_value(32796),
            OperationalStatusSecondary::MigratingVmToSuspended
        );
        assert_eq!(
            OperationalStatusSecondary::from_value(999),
            OperationalStatusSecondary::None
        );
    }

    #[test]
    fn test_operational_status_secondary_is_migrating() {
        assert!(OperationalStatusSecondary::MigratingVm.is_migrating());
        assert!(OperationalStatusSecondary::MigratingVmToSuspended.is_migrating());
        assert!(!OperationalStatusSecondary::None.is_migrating());
        assert!(!OperationalStatusSecondary::CreatingSnapshot.is_migrating());
    }

    #[test]
    fn test_startup_delay_from_secs() {
        assert_eq!(StartupDelay::from_secs(0), Some(StartupDelay::none()));
        assert_eq!(StartupDelay::from_secs(30).unwrap().as_secs(), 30);
        assert_eq!(StartupDelay::from_secs(3600).unwrap().as_secs(), 3600);
        assert_eq!(StartupDelay::from_secs(86400).unwrap().as_secs(), 86400);
        assert!(StartupDelay::from_secs(86401).is_none()); // Exceeds 24 hours
    }

    #[test]
    fn test_startup_delay_from_mins() {
        assert_eq!(StartupDelay::from_mins(5).unwrap().as_secs(), 300);
        assert_eq!(StartupDelay::from_mins(60).unwrap().as_secs(), 3600);
        assert!(StartupDelay::from_mins(1441).is_none()); // Exceeds 24 hours
    }

    #[test]
    fn test_startup_delay_from_hours() {
        assert_eq!(StartupDelay::from_hours(1).unwrap().as_secs(), 3600);
        assert_eq!(StartupDelay::from_hours(24).unwrap().as_secs(), 86400);
        assert!(StartupDelay::from_hours(25).is_none()); // Exceeds 24 hours
    }

    #[test]
    fn test_startup_delay_is_none() {
        assert!(StartupDelay::none().is_none());
        assert!(StartupDelay::from_secs(0).unwrap().is_none());
        assert!(!StartupDelay::from_secs(1).unwrap().is_none());
    }

    #[test]
    fn test_startup_delay_to_cim_interval() {
        assert_eq!(StartupDelay::none().to_cim_interval(), "");
        assert_eq!(
            StartupDelay::from_secs(30).unwrap().to_cim_interval(),
            "00000000000030.000000:000"
        );
        assert_eq!(
            StartupDelay::from_secs(90).unwrap().to_cim_interval(),
            "00000000000130.000000:000"
        );
        assert_eq!(
            StartupDelay::from_secs(3661).unwrap().to_cim_interval(),
            "00000000010101.000000:000"
        );
    }

    #[test]
    fn test_startup_delay_from_cim_interval() {
        assert_eq!(
            StartupDelay::from_cim_interval(""),
            Some(StartupDelay::none())
        );
        assert_eq!(
            StartupDelay::from_cim_interval("00000000000030.000000:000"),
            Some(StartupDelay::from_secs(30).unwrap())
        );
        assert_eq!(
            StartupDelay::from_cim_interval("00000000010101.000000:000"),
            Some(StartupDelay::from_secs(3661).unwrap())
        );
    }

    #[test]
    fn test_startup_delay_display() {
        assert_eq!(format!("{}", StartupDelay::none()), "no delay");
        assert_eq!(
            format!("{}", StartupDelay::from_secs(30).unwrap()),
            "30 seconds"
        );
        assert_eq!(
            format!("{}", StartupDelay::from_secs(120).unwrap()),
            "2 minutes"
        );
        assert_eq!(
            format!("{}", StartupDelay::from_secs(3660).unwrap()),
            "1 hours 1 minutes"
        );
    }

    #[test]
    fn test_startup_delay_default() {
        assert_eq!(StartupDelay::default(), StartupDelay::none());
    }
}
