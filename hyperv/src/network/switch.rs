use crate::error::{Error, Result};
use crate::wmi::WbemClassObjectExt;
use windows::Win32::System::Wmi::IWbemClassObject;

/// Virtual switch type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchType {
    /// External - connected to physical network adapter.
    External,
    /// Internal - accessible from host and VMs.
    Internal,
    /// Private - only accessible between VMs.
    Private,
}

impl SwitchType {
    pub fn from_value(value: u32) -> Self {
        match value {
            0 => SwitchType::Private,
            1 => SwitchType::Internal,
            2 => SwitchType::External,
            _ => SwitchType::Private,
        }
    }

    pub fn to_value(&self) -> u32 {
        match self {
            SwitchType::Private => 0,
            SwitchType::Internal => 1,
            SwitchType::External => 2,
        }
    }
}

/// Represents a Hyper-V virtual switch.
#[derive(Debug)]
pub struct VirtualSwitch {
    /// Switch display name.
    pub name: String,
    /// Switch unique ID.
    pub id: String,
    /// Switch type.
    pub switch_type: SwitchType,
    /// Description/notes.
    pub notes: Option<String>,
    /// Whether the switch allows management OS access.
    pub allow_management_os: bool,
    /// WMI path.
    #[allow(dead_code)]
    path: String,
}

impl VirtualSwitch {
    /// Create from WMI object.
    pub(crate) fn from_wmi(obj: &IWbemClassObject) -> Result<Self> {
        let name = obj.get_string_prop_required("ElementName")?;
        let id = obj.get_string_prop_required("Name")?;
        let path = obj.get_path()?;
        let notes = obj.get_string_prop("Notes")?;

        // Get switch type from associated Msvm_VirtualEthernetSwitchSettingData
        // For simplicity, default to Private
        let switch_type = SwitchType::Private;
        let allow_management_os = false;

        Ok(Self {
            name,
            id,
            switch_type,
            notes,
            allow_management_os,
            path,
        })
    }

    /// Get the switch name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the switch ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the switch type.
    pub fn switch_type(&self) -> SwitchType {
        self.switch_type
    }

    /// Get the WMI path.
    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}

/// Settings for creating a virtual switch.
#[derive(Debug, Clone)]
pub struct VirtualSwitchSettings {
    /// Switch name.
    pub name: String,
    /// Switch type.
    pub switch_type: SwitchType,
    /// Notes/description.
    pub notes: Option<String>,
    /// Allow management OS to use this switch.
    pub allow_management_os: bool,
    /// External network adapter name (required for External type).
    pub external_adapter: Option<String>,
}

impl VirtualSwitchSettings {
    /// Create a new builder.
    pub fn builder() -> VirtualSwitchSettingsBuilder {
        VirtualSwitchSettingsBuilder::default()
    }

    /// Validate settings.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::Validation {
                field: "name",
                message: "Switch name cannot be empty".to_string(),
            });
        }

        if self.switch_type == SwitchType::External && self.external_adapter.is_none() {
            return Err(Error::Validation {
                field: "external_adapter",
                message: "External switch requires a physical network adapter".to_string(),
            });
        }

        Ok(())
    }
}

/// Builder for virtual switch settings.
#[derive(Default)]
pub struct VirtualSwitchSettingsBuilder {
    name: Option<String>,
    switch_type: SwitchType,
    notes: Option<String>,
    allow_management_os: bool,
    external_adapter: Option<String>,
}

impl Default for SwitchType {
    fn default() -> Self {
        SwitchType::Private
    }
}

impl VirtualSwitchSettingsBuilder {
    /// Set switch name (required).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set switch type.
    pub fn switch_type(mut self, switch_type: SwitchType) -> Self {
        self.switch_type = switch_type;
        self
    }

    /// Set notes/description.
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Allow management OS to use this switch.
    pub fn allow_management_os(mut self, allow: bool) -> Self {
        self.allow_management_os = allow;
        self
    }

    /// Set external network adapter (for External switches).
    pub fn external_adapter(mut self, adapter: impl Into<String>) -> Self {
        self.external_adapter = Some(adapter.into());
        self.switch_type = SwitchType::External;
        self
    }

    /// Build and validate settings.
    pub fn build(self) -> Result<VirtualSwitchSettings> {
        let settings = VirtualSwitchSettings {
            name: self.name.ok_or(Error::MissingRequired("name"))?,
            switch_type: self.switch_type,
            notes: self.notes,
            allow_management_os: self.allow_management_os,
            external_adapter: self.external_adapter,
        };

        settings.validate()?;
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_type_from_value() {
        assert_eq!(SwitchType::from_value(0), SwitchType::Private);
        assert_eq!(SwitchType::from_value(1), SwitchType::Internal);
        assert_eq!(SwitchType::from_value(2), SwitchType::External);
        assert_eq!(SwitchType::from_value(99), SwitchType::Private); // Default
    }

    #[test]
    fn test_switch_type_to_value() {
        assert_eq!(SwitchType::Private.to_value(), 0);
        assert_eq!(SwitchType::Internal.to_value(), 1);
        assert_eq!(SwitchType::External.to_value(), 2);
    }

    #[test]
    fn test_switch_type_roundtrip() {
        for st in [
            SwitchType::Private,
            SwitchType::Internal,
            SwitchType::External,
        ] {
            assert_eq!(SwitchType::from_value(st.to_value()), st);
        }
    }

    #[test]
    fn test_switch_type_default() {
        assert_eq!(SwitchType::default(), SwitchType::Private);
    }

    #[test]
    fn test_switch_settings_builder_private() {
        let result = VirtualSwitchSettings::builder()
            .name("TestSwitch")
            .switch_type(SwitchType::Private)
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.name, "TestSwitch");
        assert_eq!(settings.switch_type, SwitchType::Private);
    }

    #[test]
    fn test_switch_settings_builder_internal() {
        let result = VirtualSwitchSettings::builder()
            .name("InternalSwitch")
            .switch_type(SwitchType::Internal)
            .allow_management_os(true)
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.switch_type, SwitchType::Internal);
        assert!(settings.allow_management_os);
    }

    #[test]
    fn test_switch_settings_builder_external_with_adapter() {
        let result = VirtualSwitchSettings::builder()
            .name("ExternalSwitch")
            .external_adapter("Ethernet")
            .allow_management_os(true)
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.switch_type, SwitchType::External);
        assert_eq!(settings.external_adapter, Some("Ethernet".to_string()));
    }

    #[test]
    fn test_switch_settings_builder_external_without_adapter() {
        let result = VirtualSwitchSettings::builder()
            .name("ExternalSwitch")
            .switch_type(SwitchType::External)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_switch_settings_builder_missing_name() {
        let result = VirtualSwitchSettings::builder()
            .switch_type(SwitchType::Private)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_switch_settings_validation_empty_name() {
        let result = VirtualSwitchSettings::builder()
            .name("")
            .switch_type(SwitchType::Private)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_switch_settings_with_notes() {
        let result = VirtualSwitchSettings::builder()
            .name("TestSwitch")
            .notes("This is a test switch")
            .build();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.notes, Some("This is a test switch".to_string()));
    }
}
