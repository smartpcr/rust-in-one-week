use crate::error::{Error, Result};
use crate::vm::CheckpointType;
use crate::wmi::WbemClassObjectExt;
use windows::Win32::System::Wmi::IWbemClassObject;

/// Represents a VM checkpoint (snapshot).
#[derive(Debug)]
pub struct Checkpoint {
    /// Checkpoint display name.
    pub name: String,
    /// Checkpoint unique ID.
    pub id: String,
    /// Associated VM ID.
    pub vm_id: String,
    /// Parent checkpoint ID (if not root).
    pub parent_id: Option<String>,
    /// Creation time.
    pub creation_time: String,
    /// Notes/description.
    pub notes: Option<String>,
    /// WMI path.
    path: String,
}

impl Checkpoint {
    /// Create from WMI object (Msvm_VirtualSystemSettingData with VirtualSystemType = snapshot).
    pub(crate) fn from_wmi(obj: &IWbemClassObject) -> Result<Self> {
        let name = obj.get_string_prop_required("ElementName")?;
        let id = obj.get_string_prop_required("InstanceID")?;
        let vm_id = obj.get_string_prop("VirtualSystemIdentifier")?.unwrap_or_default();
        let parent_id = obj.get_string_prop("Parent")?;
        let creation_time = obj.get_string_prop("CreationTime")?.unwrap_or_default();
        let notes = obj.get_string_prop("Notes")?;
        let path = obj.get_path()?;

        Ok(Self {
            name,
            id,
            vm_id,
            parent_id,
            creation_time,
            notes,
            path,
        })
    }

    /// Get checkpoint name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get checkpoint ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the WMI path.
    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}

/// Settings for creating a checkpoint.
#[derive(Debug, Clone)]
pub struct CheckpointSettings {
    /// Checkpoint name.
    pub name: String,
    /// Notes/description.
    pub notes: Option<String>,
    /// Checkpoint type.
    pub checkpoint_type: CheckpointType,
    /// Consistency level (for production checkpoints).
    pub consistency_level: ConsistencyLevel,
}

impl CheckpointSettings {
    /// Create a new builder.
    pub fn builder() -> CheckpointSettingsBuilder {
        CheckpointSettingsBuilder::default()
    }

    /// Validate settings.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::Validation {
                field: "name",
                message: "Checkpoint name cannot be empty".to_string(),
            });
        }

        if self.name.len() > 100 {
            return Err(Error::Validation {
                field: "name",
                message: "Checkpoint name cannot exceed 100 characters".to_string(),
            });
        }

        Ok(())
    }
}

/// Builder for checkpoint settings.
#[derive(Default)]
pub struct CheckpointSettingsBuilder {
    name: Option<String>,
    notes: Option<String>,
    checkpoint_type: CheckpointType,
    consistency_level: ConsistencyLevel,
}

impl CheckpointSettingsBuilder {
    /// Set checkpoint name (required).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set notes/description.
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set checkpoint type.
    pub fn checkpoint_type(mut self, checkpoint_type: CheckpointType) -> Self {
        self.checkpoint_type = checkpoint_type;
        self
    }

    /// Set consistency level.
    pub fn consistency_level(mut self, level: ConsistencyLevel) -> Self {
        self.consistency_level = level;
        self
    }

    /// Build and validate settings.
    pub fn build(self) -> Result<CheckpointSettings> {
        let settings = CheckpointSettings {
            name: self.name.ok_or(Error::MissingRequired("name"))?,
            notes: self.notes,
            checkpoint_type: self.checkpoint_type,
            consistency_level: self.consistency_level,
        };

        settings.validate()?;
        Ok(settings)
    }
}

/// Consistency level for production checkpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsistencyLevel {
    /// Application-consistent (requires VSS support in guest).
    #[default]
    ApplicationConsistent,
    /// Crash-consistent (fallback if app-consistent fails).
    CrashConsistent,
}

impl ConsistencyLevel {
    pub fn to_value(&self) -> u16 {
        match self {
            ConsistencyLevel::ApplicationConsistent => 1,
            ConsistencyLevel::CrashConsistent => 2,
        }
    }
}
