use crate::error::{Error, Result};
use crate::vm::{
    AutomaticStartAction, AutomaticStopAction, CheckpointType, Generation,
};

/// VM settings for creation and modification.
///
/// Use [`VmSettingsBuilder`] to construct with validation.
#[derive(Debug, Clone)]
pub struct VmSettings {
    /// VM display name (required).
    pub name: String,
    /// VM generation (required).
    pub generation: Generation,
    /// Memory in MB (required, min 32).
    pub memory_mb: u64,
    /// Number of virtual processors (required, min 1).
    pub processor_count: u32,
    /// Path to store VM configuration files.
    pub config_path: Option<String>,
    /// Path to store VM snapshots.
    pub snapshot_path: Option<String>,
    /// Path for smart paging file.
    pub smart_paging_path: Option<String>,
    /// Enable dynamic memory.
    pub dynamic_memory: bool,
    /// Minimum memory when using dynamic memory (MB).
    pub dynamic_memory_min_mb: Option<u64>,
    /// Maximum memory when using dynamic memory (MB).
    pub dynamic_memory_max_mb: Option<u64>,
    /// Memory buffer percentage for dynamic memory.
    pub memory_buffer_percentage: Option<u32>,
    /// Enable secure boot (Gen2 only).
    pub secure_boot: bool,
    /// Secure boot template (Microsoft Windows, Microsoft UEFI Certificate Authority, etc.).
    pub secure_boot_template: Option<String>,
    /// Enable TPM.
    pub tpm_enabled: bool,
    /// Enable nested virtualization.
    pub nested_virtualization: bool,
    /// Automatic start action.
    pub automatic_start_action: AutomaticStartAction,
    /// Automatic start delay in seconds.
    pub automatic_start_delay: u32,
    /// Automatic stop action.
    pub automatic_stop_action: AutomaticStopAction,
    /// Checkpoint type.
    pub checkpoint_type: CheckpointType,
    /// VM notes/description.
    pub notes: Option<String>,
}

impl VmSettings {
    /// Create a new builder.
    pub fn builder() -> VmSettingsBuilder {
        VmSettingsBuilder::default()
    }

    /// Validate settings.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::Validation {
                field: "name",
                message: "VM name cannot be empty".to_string(),
            });
        }

        if self.name.len() > 100 {
            return Err(Error::Validation {
                field: "name",
                message: "VM name cannot exceed 100 characters".to_string(),
            });
        }

        // Check for invalid characters in name
        if self.name.chars().any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')) {
            return Err(Error::Validation {
                field: "name",
                message: "VM name contains invalid characters".to_string(),
            });
        }

        if self.memory_mb < 32 {
            return Err(Error::Validation {
                field: "memory_mb",
                message: "Memory must be at least 32 MB".to_string(),
            });
        }

        if self.memory_mb > 12_582_912 {
            // 12 TB max
            return Err(Error::Validation {
                field: "memory_mb",
                message: "Memory cannot exceed 12 TB".to_string(),
            });
        }

        if self.processor_count < 1 {
            return Err(Error::Validation {
                field: "processor_count",
                message: "Processor count must be at least 1".to_string(),
            });
        }

        if self.processor_count > 240 {
            return Err(Error::Validation {
                field: "processor_count",
                message: "Processor count cannot exceed 240".to_string(),
            });
        }

        if self.dynamic_memory {
            if let Some(min) = self.dynamic_memory_min_mb {
                if min < 32 {
                    return Err(Error::Validation {
                        field: "dynamic_memory_min_mb",
                        message: "Minimum dynamic memory must be at least 32 MB".to_string(),
                    });
                }
                if min > self.memory_mb {
                    return Err(Error::Validation {
                        field: "dynamic_memory_min_mb",
                        message: "Minimum memory cannot exceed startup memory".to_string(),
                    });
                }
            }

            if let Some(max) = self.dynamic_memory_max_mb {
                if max < self.memory_mb {
                    return Err(Error::Validation {
                        field: "dynamic_memory_max_mb",
                        message: "Maximum memory cannot be less than startup memory".to_string(),
                    });
                }
            }

            if let (Some(min), Some(max)) = (self.dynamic_memory_min_mb, self.dynamic_memory_max_mb) {
                if min > max {
                    return Err(Error::Validation {
                        field: "dynamic_memory",
                        message: "Minimum memory cannot exceed maximum memory".to_string(),
                    });
                }
            }

            if let Some(buffer) = self.memory_buffer_percentage {
                if buffer > 100 {
                    return Err(Error::Validation {
                        field: "memory_buffer_percentage",
                        message: "Memory buffer cannot exceed 100%".to_string(),
                    });
                }
            }
        }

        // Gen1-specific validations
        if self.generation == Generation::Gen1 {
            if self.secure_boot {
                return Err(Error::Validation {
                    field: "secure_boot",
                    message: "Secure Boot is only available for Generation 2 VMs".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Builder for [`VmSettings`] with required field enforcement.
#[derive(Default)]
pub struct VmSettingsBuilder {
    name: Option<String>,
    generation: Option<Generation>,
    memory_mb: Option<u64>,
    processor_count: Option<u32>,
    config_path: Option<String>,
    snapshot_path: Option<String>,
    smart_paging_path: Option<String>,
    dynamic_memory: bool,
    dynamic_memory_min_mb: Option<u64>,
    dynamic_memory_max_mb: Option<u64>,
    memory_buffer_percentage: Option<u32>,
    secure_boot: bool,
    secure_boot_template: Option<String>,
    tpm_enabled: bool,
    nested_virtualization: bool,
    automatic_start_action: AutomaticStartAction,
    automatic_start_delay: u32,
    automatic_stop_action: AutomaticStopAction,
    checkpoint_type: CheckpointType,
    notes: Option<String>,
}

impl VmSettingsBuilder {
    /// Set VM name (required).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set VM generation (required).
    pub fn generation(mut self, generation: Generation) -> Self {
        self.generation = Some(generation);
        self
    }

    /// Set memory size in MB (required, min 32).
    pub fn memory_mb(mut self, mb: u64) -> Self {
        self.memory_mb = Some(mb);
        self
    }

    /// Set number of virtual processors (required, min 1).
    pub fn processor_count(mut self, count: u32) -> Self {
        self.processor_count = Some(count);
        self
    }

    /// Set configuration file storage path.
    pub fn config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set snapshot storage path.
    pub fn snapshot_path(mut self, path: impl Into<String>) -> Self {
        self.snapshot_path = Some(path.into());
        self
    }

    /// Set smart paging file path.
    pub fn smart_paging_path(mut self, path: impl Into<String>) -> Self {
        self.smart_paging_path = Some(path.into());
        self
    }

    /// Enable dynamic memory.
    pub fn dynamic_memory(mut self, enabled: bool) -> Self {
        self.dynamic_memory = enabled;
        self
    }

    /// Set minimum memory for dynamic memory (MB).
    pub fn dynamic_memory_min_mb(mut self, mb: u64) -> Self {
        self.dynamic_memory_min_mb = Some(mb);
        self
    }

    /// Set maximum memory for dynamic memory (MB).
    pub fn dynamic_memory_max_mb(mut self, mb: u64) -> Self {
        self.dynamic_memory_max_mb = Some(mb);
        self
    }

    /// Set memory buffer percentage for dynamic memory.
    pub fn memory_buffer_percentage(mut self, percent: u32) -> Self {
        self.memory_buffer_percentage = Some(percent);
        self
    }

    /// Enable secure boot (Gen2 only).
    pub fn secure_boot(mut self, enabled: bool) -> Self {
        self.secure_boot = enabled;
        self
    }

    /// Set secure boot template.
    pub fn secure_boot_template(mut self, template: impl Into<String>) -> Self {
        self.secure_boot_template = Some(template.into());
        self
    }

    /// Enable TPM.
    pub fn tpm_enabled(mut self, enabled: bool) -> Self {
        self.tpm_enabled = enabled;
        self
    }

    /// Enable nested virtualization.
    pub fn nested_virtualization(mut self, enabled: bool) -> Self {
        self.nested_virtualization = enabled;
        self
    }

    /// Set automatic start action.
    pub fn automatic_start_action(mut self, action: AutomaticStartAction) -> Self {
        self.automatic_start_action = action;
        self
    }

    /// Set automatic start delay in seconds.
    pub fn automatic_start_delay(mut self, seconds: u32) -> Self {
        self.automatic_start_delay = seconds;
        self
    }

    /// Set automatic stop action.
    pub fn automatic_stop_action(mut self, action: AutomaticStopAction) -> Self {
        self.automatic_stop_action = action;
        self
    }

    /// Set checkpoint type.
    pub fn checkpoint_type(mut self, checkpoint_type: CheckpointType) -> Self {
        self.checkpoint_type = checkpoint_type;
        self
    }

    /// Set VM notes/description.
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Build and validate the settings.
    pub fn build(self) -> Result<VmSettings> {
        let settings = VmSettings {
            name: self.name.ok_or(Error::MissingRequired("name"))?,
            generation: self.generation.ok_or(Error::MissingRequired("generation"))?,
            memory_mb: self.memory_mb.ok_or(Error::MissingRequired("memory_mb"))?,
            processor_count: self.processor_count.ok_or(Error::MissingRequired("processor_count"))?,
            config_path: self.config_path,
            snapshot_path: self.snapshot_path,
            smart_paging_path: self.smart_paging_path,
            dynamic_memory: self.dynamic_memory,
            dynamic_memory_min_mb: self.dynamic_memory_min_mb,
            dynamic_memory_max_mb: self.dynamic_memory_max_mb,
            memory_buffer_percentage: self.memory_buffer_percentage,
            secure_boot: self.secure_boot,
            secure_boot_template: self.secure_boot_template,
            tpm_enabled: self.tpm_enabled,
            nested_virtualization: self.nested_virtualization,
            automatic_start_action: self.automatic_start_action,
            automatic_start_delay: self.automatic_start_delay,
            automatic_stop_action: self.automatic_stop_action,
            checkpoint_type: self.checkpoint_type,
            notes: self.notes,
        };

        settings.validate()?;
        Ok(settings)
    }
}
