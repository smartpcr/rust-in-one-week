//! Security settings management for Hyper-V VMs.

use super::types::*;
use crate::error::{Error, Result};

#[cfg(windows)]
use crate::wmi::{WbemClassObjectExt, WmiConnection};

/// Security settings for a VM.
///
/// These settings control Secure Boot, TPM, guest isolation, and shielding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecuritySettings {
    /// Whether Secure Boot is enabled.
    pub secure_boot_enabled: bool,
    /// Secure Boot template (required if Secure Boot is enabled).
    pub secure_boot_template: Option<SecureBootTemplate>,
    /// Whether vTPM (Virtual Trusted Platform Module) is enabled.
    pub tpm_enabled: bool,
    /// Guest state isolation type.
    pub guest_isolation_type: GuestIsolationType,
    /// Encrypt VM state and migration traffic.
    pub encrypt_state_and_migration: bool,
    /// Whether shielding is requested.
    pub shielding_requested: bool,
    /// Whether data encryption is enabled.
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

impl SecuritySettings {
    /// Create a new builder.
    pub fn builder() -> SecuritySettingsBuilder {
        SecuritySettingsBuilder::new()
    }

    /// Create settings for a standard Windows Gen2 VM.
    pub fn windows_gen2() -> Self {
        Self {
            secure_boot_enabled: true,
            secure_boot_template: Some(SecureBootTemplate::MicrosoftWindows),
            tpm_enabled: false,
            guest_isolation_type: GuestIsolationType::None,
            encrypt_state_and_migration: false,
            shielding_requested: false,
            data_encryption_enabled: false,
            key_protector_type: KeyProtectorType::None,
        }
    }

    /// Create settings for a Windows Gen2 VM with TPM.
    pub fn windows_gen2_with_tpm() -> Self {
        Self {
            secure_boot_enabled: true,
            secure_boot_template: Some(SecureBootTemplate::MicrosoftWindows),
            tpm_enabled: true,
            guest_isolation_type: GuestIsolationType::None,
            encrypt_state_and_migration: false,
            shielding_requested: false,
            data_encryption_enabled: false,
            key_protector_type: KeyProtectorType::None,
        }
    }

    /// Create settings for a Linux Gen2 VM.
    pub fn linux_gen2() -> Self {
        Self {
            secure_boot_enabled: true,
            secure_boot_template: Some(SecureBootTemplate::MicrosoftUefiCa),
            tpm_enabled: false,
            guest_isolation_type: GuestIsolationType::None,
            encrypt_state_and_migration: false,
            shielding_requested: false,
            data_encryption_enabled: false,
            key_protector_type: KeyProtectorType::None,
        }
    }

    /// Create settings with no security features.
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if any security features are enabled.
    pub fn has_security_features(&self) -> bool {
        self.secure_boot_enabled
            || self.tpm_enabled
            || self.guest_isolation_type != GuestIsolationType::None
            || self.shielding_requested
    }

    /// Check if this configuration requires Generation 2.
    pub fn requires_gen2(&self) -> bool {
        self.secure_boot_enabled || self.tpm_enabled
    }

    /// Get security settings for a VM.
    #[cfg(windows)]
    pub fn get(conn: &WmiConnection, vm_id: &str) -> Result<Self> {
        // Query Msvm_SecuritySettingData via association
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState \
             ResultClass=Msvm_VirtualSystemSettingData",
            vm_id.replace('\'', "''")
        );

        let vssd_results = conn.query(&query)?;
        if vssd_results.is_empty() {
            return Ok(SecuritySettings::default());
        }

        // Get VSSD path
        let vssd = &vssd_results[0];
        let vssd_path = vssd.get_path()?;

        // Query SecuritySettingData from VSSD
        let security_query = format!(
            "ASSOCIATORS OF {{{}}} \
             WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
             ResultClass=Msvm_SecuritySettingData",
            vssd_path
        );

        let security_results = conn.query(&security_query)?;

        if let Some(obj) = security_results.first() {
            let secure_boot_enabled = obj.get_bool("SecureBootEnabled")?.unwrap_or(false);
            let secure_boot_template_id = obj.get_string_prop("SecureBootTemplateId")?;

            Ok(SecuritySettings {
                secure_boot_enabled,
                secure_boot_template: secure_boot_template_id
                    .as_deref()
                    .and_then(SecureBootTemplate::from_guid),
                tpm_enabled: obj.get_bool("TpmEnabled")?.unwrap_or(false),
                guest_isolation_type: GuestIsolationType::from(
                    obj.get_u16("VirtualizationBasedSecurityOptOut")?.unwrap_or(0),
                ),
                encrypt_state_and_migration: obj
                    .get_bool("EncryptStateAndVmMigrationTraffic")?
                    .unwrap_or(false),
                shielding_requested: obj.get_bool("ShieldingRequested")?.unwrap_or(false),
                data_encryption_enabled: obj.get_bool("DataEncryptionEnabled")?.unwrap_or(false),
                key_protector_type: KeyProtectorType::None,
            })
        } else {
            Ok(SecuritySettings::default())
        }
    }

    /// Apply security settings to a VM.
    #[cfg(windows)]
    pub fn apply(&self, conn: &WmiConnection, vm_id: &str) -> Result<()> {
        use std::time::Duration;

        // Get VSMS
        let vsms = conn.get_singleton("Msvm_VirtualSystemManagementService")?;
        let vsms_path = vsms.get_path()?;

        // Get current VSSD
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState \
             ResultClass=Msvm_VirtualSystemSettingData",
            vm_id.replace('\'', "''")
        );

        let vssd_results = conn.query(&query)?;
        if vssd_results.is_empty() {
            return Err(Error::VmNotFound(vm_id.to_string()));
        }

        let vssd = &vssd_results[0];
        let vssd_path = vssd.get_path()?;

        // Query SecuritySettingData from VSSD
        let security_query = format!(
            "ASSOCIATORS OF {{{}}} \
             WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
             ResultClass=Msvm_SecuritySettingData",
            vssd_path
        );

        let security_results = conn.query(&security_query)?;

        if let Some(security_obj) = security_results.first() {
            // Modify security settings
            let mut security_text = security_obj.get_text()?;

            // Update properties in the embedded instance text
            // This is a simplified approach - in production, we'd modify the XML/MOF properly
            security_obj.put_bool("TpmEnabled", self.tpm_enabled)?;
            security_obj.put_bool("SecureBootEnabled", self.secure_boot_enabled)?;

            if let Some(ref template) = self.secure_boot_template {
                security_obj.put_string("SecureBootTemplateId", template.to_guid())?;
            }

            security_obj
                .put_bool("EncryptStateAndVmMigrationTraffic", self.encrypt_state_and_migration)?;
            security_obj.put_bool("ShieldingRequested", self.shielding_requested)?;

            security_text = security_obj.get_text()?;

            // Call ModifySecuritySettings
            let in_params = conn.get_method_params(
                "Msvm_VirtualSystemManagementService",
                "ModifySecuritySettings",
            )?;
            in_params.put_string("SecuritySettingData", &security_text)?;

            let out_params = conn.exec_method(&vsms_path, "ModifySecuritySettings", Some(&in_params))?;

            // Check result
            let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
            match return_value {
                0 => Ok(()), // Completed
                4096 => {
                    // Job started
                    let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                        Error::operation_failed("ModifySecuritySettings", 4096, "No job path returned")
                    })?;

                    let waiter = crate::wmi::JobWaiter::with_timeout(conn, Duration::from_secs(60));
                    waiter.wait_for_job(&job_path, "ModifySecuritySettings")?;
                    Ok(())
                }
                code => Err(Error::operation_failed(
                    "ModifySecuritySettings",
                    code,
                    format!("Failed to modify security settings"),
                )),
            }
        } else {
            // No security settings object - might need to create one
            Err(Error::operation_failed(
                "ModifySecuritySettings",
                0,
                "Security settings not found for VM",
            ))
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

    /// Enable or disable Secure Boot.
    pub fn secure_boot(mut self, enabled: bool) -> Self {
        self.settings.secure_boot_enabled = enabled;
        self
    }

    /// Set the Secure Boot template.
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

    /// Enable or disable vTPM.
    pub fn tpm(mut self, enabled: bool) -> Self {
        self.settings.tpm_enabled = enabled;
        self
    }

    /// Set guest isolation type.
    pub fn guest_isolation(mut self, isolation: GuestIsolationType) -> Self {
        self.settings.guest_isolation_type = isolation;
        self
    }

    /// Enable VBS (Virtualization-based Security) isolation.
    pub fn vbs_isolation(self) -> Self {
        self.guest_isolation(GuestIsolationType::Vbs)
    }

    /// Enable or disable state and migration traffic encryption.
    pub fn encrypt_state_and_migration(mut self, enabled: bool) -> Self {
        self.settings.encrypt_state_and_migration = enabled;
        self
    }

    /// Enable or disable shielding.
    pub fn shielding(mut self, enabled: bool) -> Self {
        self.settings.shielding_requested = enabled;
        self
    }

    /// Enable or disable data encryption.
    pub fn data_encryption(mut self, enabled: bool) -> Self {
        self.settings.data_encryption_enabled = enabled;
        self
    }

    /// Set key protector type.
    pub fn key_protector(mut self, protector: KeyProtectorType) -> Self {
        self.settings.key_protector_type = protector;
        self
    }

    /// Build the settings, validating the configuration.
    pub fn build(self) -> Result<SecuritySettings> {
        // Validate: Secure Boot requires a template
        if self.settings.secure_boot_enabled && self.settings.secure_boot_template.is_none() {
            return Err(Error::Validation {
                field: "secure_boot_template",
                message: "Secure Boot requires a template to be specified".to_string(),
            });
        }

        // Validate: Shielding requires TPM
        if self.settings.shielding_requested && !self.settings.tpm_enabled {
            return Err(Error::Validation {
                field: "tpm_enabled",
                message: "Shielding requires vTPM to be enabled".to_string(),
            });
        }

        // Validate: VBS isolation requires certain features
        if self.settings.guest_isolation_type == GuestIsolationType::Vbs {
            if !self.settings.tpm_enabled {
                return Err(Error::Validation {
                    field: "tpm_enabled",
                    message: "VBS isolation requires vTPM to be enabled".to_string(),
                });
            }
            if !self.settings.secure_boot_enabled {
                return Err(Error::Validation {
                    field: "secure_boot_enabled",
                    message: "VBS isolation requires Secure Boot to be enabled".to_string(),
                });
            }
        }

        Ok(self.settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = SecuritySettings::default();
        assert!(!settings.secure_boot_enabled);
        assert!(settings.secure_boot_template.is_none());
        assert!(!settings.tpm_enabled);
        assert_eq!(settings.guest_isolation_type, GuestIsolationType::None);
        assert!(!settings.has_security_features());
    }

    #[test]
    fn test_windows_gen2_settings() {
        let settings = SecuritySettings::windows_gen2();
        assert!(settings.secure_boot_enabled);
        assert_eq!(
            settings.secure_boot_template,
            Some(SecureBootTemplate::MicrosoftWindows)
        );
        assert!(!settings.tpm_enabled);
        assert!(settings.requires_gen2());
    }

    #[test]
    fn test_windows_gen2_with_tpm() {
        let settings = SecuritySettings::windows_gen2_with_tpm();
        assert!(settings.secure_boot_enabled);
        assert!(settings.tpm_enabled);
        assert!(settings.has_security_features());
    }

    #[test]
    fn test_linux_gen2_settings() {
        let settings = SecuritySettings::linux_gen2();
        assert!(settings.secure_boot_enabled);
        assert_eq!(
            settings.secure_boot_template,
            Some(SecureBootTemplate::MicrosoftUefiCa)
        );
    }

    #[test]
    fn test_builder_basic() {
        let settings = SecuritySettings::builder()
            .secure_boot_template(SecureBootTemplate::MicrosoftWindows)
            .tpm(true)
            .build()
            .unwrap();

        assert!(settings.secure_boot_enabled);
        assert!(settings.tpm_enabled);
    }

    #[test]
    fn test_builder_secure_boot_requires_template() {
        let result = SecuritySettings::builder().secure_boot(true).build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("template"));
    }

    #[test]
    fn test_builder_shielding_requires_tpm() {
        let result = SecuritySettings::builder()
            .secure_boot_windows()
            .shielding(true)
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("TPM") || format!("{}", err).contains("tpm"));
    }

    #[test]
    fn test_builder_vbs_requires_tpm_and_secureboot() {
        // Missing TPM
        let result = SecuritySettings::builder()
            .secure_boot_windows()
            .vbs_isolation()
            .build();
        assert!(result.is_err());

        // Missing Secure Boot
        let result = SecuritySettings::builder().tpm(true).vbs_isolation().build();
        assert!(result.is_err());

        // Both present - should succeed
        let result = SecuritySettings::builder()
            .secure_boot_windows()
            .tpm(true)
            .vbs_isolation()
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_linux_shorthand() {
        let settings = SecuritySettings::builder()
            .secure_boot_linux()
            .build()
            .unwrap();

        assert_eq!(
            settings.secure_boot_template,
            Some(SecureBootTemplate::MicrosoftUefiCa)
        );
    }

    #[test]
    fn test_has_security_features() {
        assert!(!SecuritySettings::none().has_security_features());
        assert!(SecuritySettings::windows_gen2().has_security_features());

        let settings = SecuritySettings::builder()
            .guest_isolation(GuestIsolationType::Snp)
            .build()
            .unwrap();
        assert!(settings.has_security_features());
    }

    #[test]
    fn test_requires_gen2() {
        assert!(!SecuritySettings::none().requires_gen2());
        assert!(SecuritySettings::windows_gen2().requires_gen2());

        let tpm_only = SecuritySettings::builder().tpm(true).build().unwrap();
        assert!(tpm_only.requires_gen2());
    }
}
