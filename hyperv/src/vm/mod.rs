mod computer_system;
mod settings;
mod state;
mod types;

pub use computer_system::VirtualMachine;
pub use settings::{VmSettings, VmSettingsBuilder};
pub use state::{
    AutomaticStartAction, AutomaticStopAction, CaptureLiveState, CheckpointType, ExportSettings,
    Generation, ImportSettings, OperationalStatus, OperationalStatusSecondary, RequestedState,
    ShutdownType, SnapshotExportMode, StartupDelay, VmState,
};
pub use types::*;
