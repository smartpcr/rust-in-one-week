//! Security settings for Hyper-V VMs.
//!
//! This module provides support for:
//! - Secure Boot configuration
//! - vTPM (Virtual Trusted Platform Module)
//! - Guest state isolation (VBS, SNP, TDX)
//! - Shielded VM settings
//!
//! # Example
//!
//! ```no_run
//! use windows_hyperv::security::{SecuritySettings, SecureBootTemplate, GuestIsolationType};
//!
//! let settings = SecuritySettings::builder()
//!     .secure_boot_template(SecureBootTemplate::MicrosoftWindows)
//!     .tpm(true)
//!     .encrypt_state_and_migration(true)
//!     .build()?;
//! # Ok::<(), windows_hyperv::Error>(())
//! ```

mod settings;
mod types;

pub use settings::{SecuritySettings, SecuritySettingsBuilder};
pub use types::{
    FirmwareType, GuestIsolationType, KeyProtectorType, SecureBootTemplate, TpmState,
};
