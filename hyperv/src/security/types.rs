//! Security-related types for Hyper-V VMs.

use std::fmt;

/// Guest state isolation type.
///
/// Determines the type of hardware-backed isolation for the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[repr(u16)]
pub enum GuestIsolationType {
    /// No isolation - standard VM.
    #[default]
    None = 0,
    /// VBS (Virtualization-based Security) isolation.
    /// Uses Windows hypervisor capabilities.
    Vbs = 1,
    /// SNP (AMD SEV-SNP) isolation.
    /// Uses AMD Secure Encrypted Virtualization - Secure Nested Paging.
    Snp = 2,
    /// TDX (Intel TDX) isolation.
    /// Uses Intel Trust Domain Extensions.
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

impl fmt::Display for GuestIsolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuestIsolationType::None => write!(f, "None"),
            GuestIsolationType::Vbs => write!(f, "VBS"),
            GuestIsolationType::Snp => write!(f, "AMD SEV-SNP"),
            GuestIsolationType::Tdx => write!(f, "Intel TDX"),
        }
    }
}

/// Secure Boot template.
///
/// Determines which certificate authority chain is used for Secure Boot validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecureBootTemplate {
    /// Microsoft Windows template.
    /// Use for Windows VMs.
    MicrosoftWindows,
    /// Microsoft UEFI Certificate Authority.
    /// Use for Linux VMs and other third-party OSes.
    MicrosoftUefiCa,
    /// Open Source Shielded VM template.
    /// Use for open source operating systems in shielded VMs.
    OpenSourceShieldedVm,
}

impl SecureBootTemplate {
    /// Get the template GUID string.
    pub fn to_guid(&self) -> &'static str {
        match self {
            SecureBootTemplate::MicrosoftWindows => "{1734c6e8-3154-4dda-ba5f-a874cc483422}",
            SecureBootTemplate::MicrosoftUefiCa => "{272e7447-90a4-4563-a4b9-8e4ab00526ce}",
            SecureBootTemplate::OpenSourceShieldedVm => "{5c5b03be-6e38-4d00-a6a8-62b9b5f3aa72}",
        }
    }

    /// Get template from GUID string.
    pub fn from_guid(guid: &str) -> Option<Self> {
        let normalized = guid.to_lowercase();
        match normalized.as_str() {
            "{1734c6e8-3154-4dda-ba5f-a874cc483422}" => Some(SecureBootTemplate::MicrosoftWindows),
            "{272e7447-90a4-4563-a4b9-8e4ab00526ce}" => Some(SecureBootTemplate::MicrosoftUefiCa),
            "{5c5b03be-6e38-4d00-a6a8-62b9b5f3aa72}" => {
                Some(SecureBootTemplate::OpenSourceShieldedVm)
            }
            _ => None,
        }
    }
}

impl fmt::Display for SecureBootTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecureBootTemplate::MicrosoftWindows => write!(f, "Microsoft Windows"),
            SecureBootTemplate::MicrosoftUefiCa => write!(f, "Microsoft UEFI CA"),
            SecureBootTemplate::OpenSourceShieldedVm => write!(f, "Open Source Shielded VM"),
        }
    }
}

/// Firmware type for the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[repr(u16)]
pub enum FirmwareType {
    /// Legacy BIOS firmware.
    /// Used by Generation 1 VMs.
    #[default]
    Bios = 0,
    /// UEFI firmware.
    /// Used by Generation 2 VMs. Required for Secure Boot.
    Uefi = 1,
}

impl From<u16> for FirmwareType {
    fn from(value: u16) -> Self {
        match value {
            1 => FirmwareType::Uefi,
            _ => FirmwareType::Bios,
        }
    }
}

impl fmt::Display for FirmwareType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FirmwareType::Bios => write!(f, "BIOS"),
            FirmwareType::Uefi => write!(f, "UEFI"),
        }
    }
}

/// Key protector type for shielded VMs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum KeyProtectorType {
    /// No key protector.
    #[default]
    None,
    /// Local key protector.
    /// Keys are stored locally on the host.
    Local,
    /// Host Guardian Service (HGS) key protector.
    /// Keys are protected by a remote attestation service.
    Hgs,
}

impl fmt::Display for KeyProtectorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyProtectorType::None => write!(f, "None"),
            KeyProtectorType::Local => write!(f, "Local"),
            KeyProtectorType::Hgs => write!(f, "Host Guardian Service"),
        }
    }
}

/// TPM (Trusted Platform Module) state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TpmState {
    /// TPM not present.
    #[default]
    NotPresent,
    /// TPM present but not ready.
    NotReady,
    /// TPM ready for use.
    Ready,
    /// TPM locked out.
    LockedOut,
}

impl fmt::Display for TpmState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TpmState::NotPresent => write!(f, "Not Present"),
            TpmState::NotReady => write!(f, "Not Ready"),
            TpmState::Ready => write!(f, "Ready"),
            TpmState::LockedOut => write!(f, "Locked Out"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guest_isolation_type_from_u16() {
        assert_eq!(GuestIsolationType::from(0), GuestIsolationType::None);
        assert_eq!(GuestIsolationType::from(1), GuestIsolationType::Vbs);
        assert_eq!(GuestIsolationType::from(2), GuestIsolationType::Snp);
        assert_eq!(GuestIsolationType::from(3), GuestIsolationType::Tdx);
        assert_eq!(GuestIsolationType::from(99), GuestIsolationType::None);
    }

    #[test]
    fn test_guest_isolation_type_display() {
        assert_eq!(format!("{}", GuestIsolationType::None), "None");
        assert_eq!(format!("{}", GuestIsolationType::Vbs), "VBS");
        assert_eq!(format!("{}", GuestIsolationType::Snp), "AMD SEV-SNP");
        assert_eq!(format!("{}", GuestIsolationType::Tdx), "Intel TDX");
    }

    #[test]
    fn test_secure_boot_template_guid() {
        assert!(SecureBootTemplate::MicrosoftWindows
            .to_guid()
            .starts_with('{'));
        assert!(SecureBootTemplate::MicrosoftUefiCa
            .to_guid()
            .ends_with('}'));
    }

    #[test]
    fn test_secure_boot_template_from_guid() {
        let guid = SecureBootTemplate::MicrosoftWindows.to_guid();
        assert_eq!(
            SecureBootTemplate::from_guid(guid),
            Some(SecureBootTemplate::MicrosoftWindows)
        );

        // Test case insensitivity
        let upper = guid.to_uppercase();
        assert_eq!(
            SecureBootTemplate::from_guid(&upper),
            Some(SecureBootTemplate::MicrosoftWindows)
        );

        assert_eq!(SecureBootTemplate::from_guid("invalid"), None);
    }

    #[test]
    fn test_secure_boot_template_display() {
        assert_eq!(
            format!("{}", SecureBootTemplate::MicrosoftWindows),
            "Microsoft Windows"
        );
        assert_eq!(
            format!("{}", SecureBootTemplate::MicrosoftUefiCa),
            "Microsoft UEFI CA"
        );
    }

    #[test]
    fn test_firmware_type_from_u16() {
        assert_eq!(FirmwareType::from(0), FirmwareType::Bios);
        assert_eq!(FirmwareType::from(1), FirmwareType::Uefi);
        assert_eq!(FirmwareType::from(99), FirmwareType::Bios);
    }

    #[test]
    fn test_firmware_type_display() {
        assert_eq!(format!("{}", FirmwareType::Bios), "BIOS");
        assert_eq!(format!("{}", FirmwareType::Uefi), "UEFI");
    }

    #[test]
    fn test_key_protector_type_default() {
        assert_eq!(KeyProtectorType::default(), KeyProtectorType::None);
    }

    #[test]
    fn test_tpm_state_default() {
        assert_eq!(TpmState::default(), TpmState::NotPresent);
    }
}
