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
        let vm_id = obj
            .get_string_prop("VirtualSystemIdentifier")?
            .unwrap_or_default();
        let parent_id = obj.get_string_prop("Parent")?;
        let creation_time = obj.get_string_prop("CreationTime")?.unwrap_or_default();
        // Notes is a string array in Hyper-V v2, join with newlines
        let notes = obj
            .get_string_array("Notes")?
            .map(|arr| arr.join("\n"))
            .filter(|s| !s.is_empty());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_level_to_value() {
        assert_eq!(ConsistencyLevel::ApplicationConsistent.to_value(), 1);
        assert_eq!(ConsistencyLevel::CrashConsistent.to_value(), 2);
    }

    #[test]
    fn test_consistency_level_default() {
        assert_eq!(
            ConsistencyLevel::default(),
            ConsistencyLevel::ApplicationConsistent
        );
    }

    #[test]
    fn test_checkpoint_settings_builder_valid() {
        let result = CheckpointSettings::builder().name("TestCheckpoint").build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.name, "TestCheckpoint");
        assert_eq!(settings.checkpoint_type, CheckpointType::Production);
        assert_eq!(
            settings.consistency_level,
            ConsistencyLevel::ApplicationConsistent
        );
    }

    #[test]
    fn test_checkpoint_settings_builder_missing_name() {
        let result = CheckpointSettings::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_settings_validation_empty_name() {
        let result = CheckpointSettings::builder().name("").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_settings_validation_name_too_long() {
        let long_name = "a".repeat(101);
        let result = CheckpointSettings::builder().name(long_name).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_settings_with_notes() {
        let result = CheckpointSettings::builder()
            .name("TestCheckpoint")
            .notes("This is a test checkpoint")
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(
            settings.notes,
            Some("This is a test checkpoint".to_string())
        );
    }

    #[test]
    fn test_checkpoint_settings_with_checkpoint_type() {
        let result = CheckpointSettings::builder()
            .name("StandardCheckpoint")
            .checkpoint_type(CheckpointType::Standard)
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.checkpoint_type, CheckpointType::Standard);
    }

    #[test]
    fn test_checkpoint_settings_with_consistency_level() {
        let result = CheckpointSettings::builder()
            .name("CrashConsistentCheckpoint")
            .consistency_level(ConsistencyLevel::CrashConsistent)
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(
            settings.consistency_level,
            ConsistencyLevel::CrashConsistent
        );
    }

    #[test]
    fn test_checkpoint_settings_all_options() {
        let result = CheckpointSettings::builder()
            .name("FullCheckpoint")
            .notes("Full featured checkpoint")
            .checkpoint_type(CheckpointType::ProductionOnly)
            .consistency_level(ConsistencyLevel::ApplicationConsistent)
            .build();

        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.name, "FullCheckpoint");
        assert_eq!(settings.notes, Some("Full featured checkpoint".to_string()));
        assert_eq!(settings.checkpoint_type, CheckpointType::ProductionOnly);
        assert_eq!(
            settings.consistency_level,
            ConsistencyLevel::ApplicationConsistent
        );
    }

    #[test]
    fn test_checkpoint_settings_max_length_name() {
        let name = "a".repeat(100);
        let result = CheckpointSettings::builder().name(name.clone()).build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.name.len(), 100);
    }
}
