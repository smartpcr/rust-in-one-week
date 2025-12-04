//! WMI (Windows Management Instrumentation) wrapper for Hyper-V management
//!
//! Uses the Msvm_* classes from root\virtualization\v2 namespace for comprehensive
//! Hyper-V management operations.

pub mod msvm;
pub mod operations;

use crate::error::{HvError, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::Once;
use windows::core::{BSTR, HRESULT, PCWSTR};
use windows::Win32::Foundation::S_FALSE;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket,
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT,
    RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::{
    IEnumWbemClassObject, IWbemClassObject, IWbemLocator, IWbemObjectTextSrc, IWbemServices,
    WbemLocator, WbemObjectTextSrc, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
    WBEM_INFINITE, WMI_OBJ_TEXT_WMI_DTD_2_0,
};

// RPC_E_TOO_LATE error code (0x80010119)
const RPC_E_TOO_LATE: HRESULT = HRESULT(0x80010119_u32 as i32);
// RPC authentication constants
const RPC_C_AUTHN_WINNT: u32 = 10;
const RPC_C_AUTHZ_NONE: u32 = 0;

/// Hyper-V WMI namespace
pub const HYPERV_NAMESPACE: &str = r"root\virtualization\v2";

/// CIM V2 namespace for general system info
pub const CIMV2_NAMESPACE: &str = r"root\cimv2";

// Process-wide COM security initialization (can only happen once)
static COM_SECURITY_INIT: Once = Once::new();

// Thread-local COM initialization tracker
thread_local! {
    static COM_INITIALIZED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Initialize COM for the current thread
fn ensure_com_initialized() -> Result<()> {
    COM_INITIALIZED.with(|initialized| {
        if !initialized.get() {
            unsafe {
                // Initialize COM for this thread
                // S_FALSE means already initialized, which is fine
                let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
                if hr.is_err() && hr != S_FALSE {
                    return Err(HvError::WmiError(format!(
                        "Failed to initialize COM: {:?}",
                        hr
                    )));
                }

                // Try to set COM security once per process
                // RPC_E_TOO_LATE means it was already set, which is fine
                COM_SECURITY_INIT.call_once(|| {
                    let result = CoInitializeSecurity(
                        None,
                        -1,
                        None,
                        None,
                        RPC_C_AUTHN_LEVEL_DEFAULT,
                        RPC_C_IMP_LEVEL_IMPERSONATE,
                        None,
                        EOAC_NONE,
                        None,
                    );
                    // Ignore errors - RPC_E_TOO_LATE means security was already initialized
                    // We'll set proxy blanket on each connection anyway
                    if let Err(e) = result {
                        // Only warn for unexpected errors (not RPC_E_TOO_LATE)
                        if e.code() != RPC_E_TOO_LATE {
                            eprintln!("Warning: CoInitializeSecurity failed: {:?}", e);
                        }
                    }
                });
            }
            initialized.set(true);
        }
        Ok(())
    })
}

/// Set security on a WMI services proxy
fn set_proxy_security(services: &IWbemServices) -> Result<()> {
    unsafe {
        CoSetProxyBlanket(
            services,
            RPC_C_AUTHN_WINNT,
            RPC_C_AUTHZ_NONE,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
        )
        .map_err(|e| HvError::WmiError(format!("Failed to set proxy security: {:?}", e)))?;
    }
    Ok(())
}

/// WMI connection to a namespace
pub struct WmiConnection {
    services: IWbemServices,
}

impl WmiConnection {
    /// Connect to a WMI namespace
    pub fn connect(namespace: &str) -> Result<Self> {
        ensure_com_initialized()?;

        unsafe {
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| HvError::WmiError(format!("Failed to create WMI locator: {:?}", e)))?;

            let services = locator
                .ConnectServer(
                    &BSTR::from(namespace),
                    &BSTR::new(),
                    &BSTR::new(),
                    &BSTR::new(),
                    0,
                    &BSTR::new(),
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!(
                        "Failed to connect to namespace {}: {:?}",
                        namespace, e
                    ))
                })?;

            // Set security on the proxy to allow impersonation
            set_proxy_security(&services)?;

            Ok(WmiConnection { services })
        }
    }

    /// Connect to the Hyper-V namespace
    pub fn connect_hyperv() -> Result<Self> {
        Self::connect(HYPERV_NAMESPACE)
    }

    /// Execute a WQL query and return results
    pub fn query(&self, wql: &str) -> Result<WmiQueryResult> {
        unsafe {
            let enumerator = self
                .services
                .ExecQuery(
                    &BSTR::from("WQL"),
                    &BSTR::from(wql),
                    WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                    None,
                )
                .map_err(|e| HvError::WmiError(format!("Query failed: {:?}", e)))?;

            Ok(WmiQueryResult { enumerator })
        }
    }

    /// Get a single WMI object by path
    pub fn get_object(&self, object_path: &str) -> Result<WmiObject> {
        unsafe {
            let mut class: Option<IWbemClassObject> = None;
            self.services
                .GetObject(
                    &BSTR::from(object_path),
                    Default::default(),
                    None,
                    Some(&mut class),
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get object {}: {:?}", object_path, e))
                })?;

            class
                .map(|obj| WmiObject { inner: obj })
                .ok_or_else(|| HvError::WmiError(format!("Object not found: {}", object_path)))
        }
    }

    /// Execute a method on a WMI object
    pub fn exec_method(
        &self,
        object_path: &str,
        method_name: &str,
        in_params: Option<&WmiObject>,
    ) -> Result<Option<WmiObject>> {
        unsafe {
            let mut out_params: Option<IWbemClassObject> = None;

            self.services
                .ExecMethod(
                    &BSTR::from(object_path),
                    &BSTR::from(method_name),
                    Default::default(),
                    None,
                    in_params.map(|p| &p.inner),
                    Some(&mut out_params),
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!(
                        "Failed to execute method {} on {}: {:?}",
                        method_name, object_path, e
                    ))
                })?;

            Ok(out_params.map(|obj| WmiObject { inner: obj }))
        }
    }

    /// Get a class definition to spawn instances
    pub fn get_class(&self, class_name: &str) -> Result<WmiObject> {
        unsafe {
            let mut class: Option<IWbemClassObject> = None;
            self.services
                .GetObject(
                    &BSTR::from(class_name),
                    Default::default(),
                    None,
                    Some(&mut class),
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get class {}: {:?}", class_name, e))
                })?;

            class
                .map(|obj| WmiObject { inner: obj })
                .ok_or_else(|| HvError::WmiError(format!("Class not found: {}", class_name)))
        }
    }

    /// Get method input parameters template
    pub fn get_method_params(&self, class_name: &str, method_name: &str) -> Result<WmiObject> {
        let class = self.get_class(class_name)?;

        unsafe {
            let mut input: Option<IWbemClassObject> = None;
            class
                .inner
                .GetMethod(
                    &BSTR::from(method_name),
                    0,
                    &mut input,
                    std::ptr::null_mut(),
                )
                .map_err(|e| {
                    HvError::WmiError(format!(
                        "Failed to get method {} parameters: {:?}",
                        method_name, e
                    ))
                })?;

            input
                .map(|obj| {
                    // Spawn an instance for setting values
                    obj.SpawnInstance(0)
                        .map(|inst| WmiObject { inner: inst })
                        .map_err(|e| {
                            HvError::WmiError(format!(
                                "Failed to spawn parameter instance: {:?}",
                                e
                            ))
                        })
                })
                .ok_or_else(|| {
                    HvError::WmiError(format!("No input parameters for method {}", method_name))
                })?
        }
    }

    /// Get raw services handle for advanced operations
    pub fn services(&self) -> &IWbemServices {
        &self.services
    }
}

/// Result set from a WMI query
pub struct WmiQueryResult {
    enumerator: IEnumWbemClassObject,
}

impl Iterator for WmiQueryResult {
    type Item = Result<WmiObject>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut row = [None; 1];
            let mut returned = 0;

            let hr = self.enumerator.Next(WBEM_INFINITE, &mut row, &mut returned);

            // S_FALSE indicates no more items
            if hr == S_FALSE || returned == 0 {
                return None;
            }

            // Check for errors
            if hr.is_err() {
                return Some(Err(HvError::WmiError(format!(
                    "Error enumerating results: {:?}",
                    hr
                ))));
            }

            row[0].take().map(|obj| Ok(WmiObject { inner: obj }))
        }
    }
}

/// Convert a string to a wide null-terminated string
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// A WMI object instance
pub struct WmiObject {
    inner: IWbemClassObject,
}

impl WmiObject {
    /// Get a string property value
    pub fn get_string(&self, property: &str) -> Result<Option<String>> {
        unsafe {
            let prop_wide = to_wide(property);
            let mut value = VARIANT::default();
            self.inner
                .Get(
                    PCWSTR::from_raw(prop_wide.as_ptr()),
                    0,
                    &mut value,
                    None,
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get property {}: {:?}", property, e))
                })?;

            // Check for null/empty
            if value.is_empty() {
                return Ok(None);
            }

            // Access the variant type and data through the Anonymous union
            // VT_BSTR = 8
            let vt = value.vt();
            if vt.0 == 8 {
                // It's a BSTR - access through the Anonymous union
                // We need to access: value.Anonymous.Anonymous.Anonymous.bstrVal
                let ptr = std::ptr::addr_of!(value) as *const u8;
                // VARIANT layout: vt (2) + reserved (6) + data (8)
                // The BSTR pointer is at offset 8
                let bstr_ptr = *(ptr.add(8) as *const *const u16);
                if !bstr_ptr.is_null() {
                    // Read the BSTR length (4 bytes before the pointer)
                    let len_ptr = (bstr_ptr as *const u32).sub(1);
                    let byte_len = *len_ptr as usize;
                    let char_len = byte_len / 2;
                    let slice = std::slice::from_raw_parts(bstr_ptr, char_len);
                    return Ok(Some(String::from_utf16_lossy(slice)));
                }
            }

            Ok(None)
        }
    }

    /// Get a required string property (errors if null)
    pub fn get_string_required(&self, property: &str) -> Result<String> {
        self.get_string(property)?
            .ok_or_else(|| HvError::WmiError(format!("Property {} is null", property)))
    }

    /// Get an integer property value
    pub fn get_u32(&self, property: &str) -> Result<Option<u32>> {
        unsafe {
            let prop_wide = to_wide(property);
            let mut value = VARIANT::default();
            self.inner
                .Get(
                    PCWSTR::from_raw(prop_wide.as_ptr()),
                    0,
                    &mut value,
                    None,
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get property {}: {:?}", property, e))
                })?;

            if value.is_empty() {
                return Ok(None);
            }

            // Try different integer conversions
            if let Ok(v) = i32::try_from(&value) {
                return Ok(Some(v as u32));
            }
            if let Ok(v) = i16::try_from(&value) {
                return Ok(Some(v as u32));
            }
            if let Ok(v) = u32::try_from(&value) {
                return Ok(Some(v));
            }
            if let Ok(v) = u16::try_from(&value) {
                return Ok(Some(v as u32));
            }

            Ok(None)
        }
    }

    /// Get an unsigned 64-bit integer property
    pub fn get_u64(&self, property: &str) -> Result<Option<u64>> {
        unsafe {
            let prop_wide = to_wide(property);
            let mut value = VARIANT::default();
            self.inner
                .Get(
                    PCWSTR::from_raw(prop_wide.as_ptr()),
                    0,
                    &mut value,
                    None,
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get property {}: {:?}", property, e))
                })?;

            if value.is_empty() {
                return Ok(None);
            }

            // Try different integer conversions
            if let Ok(v) = i64::try_from(&value) {
                return Ok(Some(v as u64));
            }
            if let Ok(v) = u64::try_from(&value) {
                return Ok(Some(v));
            }
            if let Ok(v) = i32::try_from(&value) {
                return Ok(Some(v as u64));
            }
            if let Ok(v) = u32::try_from(&value) {
                return Ok(Some(v as u64));
            }

            Ok(None)
        }
    }

    /// Get a boolean property value
    pub fn get_bool(&self, property: &str) -> Result<Option<bool>> {
        unsafe {
            let prop_wide = to_wide(property);
            let mut value = VARIANT::default();
            self.inner
                .Get(
                    PCWSTR::from_raw(prop_wide.as_ptr()),
                    0,
                    &mut value,
                    None,
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get property {}: {:?}", property, e))
                })?;

            if value.is_empty() {
                return Ok(None);
            }

            if let Ok(v) = bool::try_from(&value) {
                return Ok(Some(v));
            }

            Ok(None)
        }
    }

    /// Get an array of strings
    pub fn get_string_array(&self, property: &str) -> Result<Vec<String>> {
        unsafe {
            let prop_wide = to_wide(property);
            let mut value = VARIANT::default();
            self.inner
                .Get(
                    PCWSTR::from_raw(prop_wide.as_ptr()),
                    0,
                    &mut value,
                    None,
                    None,
                )
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to get property {}: {:?}", property, e))
                })?;

            if value.is_empty() {
                return Ok(Vec::new());
            }

            // For now, return empty - array handling is complex
            // TODO: Implement proper SAFEARRAY parsing
            Ok(Vec::new())
        }
    }

    /// Set a string property value
    pub fn put_string(&self, property: &str, value: &str) -> Result<()> {
        unsafe {
            let prop_wide = to_wide(property);
            let variant = VARIANT::from(BSTR::from(value));
            self.inner
                .Put(PCWSTR::from_raw(prop_wide.as_ptr()), 0, &variant, 0)
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to set property {}: {:?}", property, e))
                })?;
            Ok(())
        }
    }

    /// Set a u16 property value
    pub fn put_u16(&self, property: &str, value: u16) -> Result<()> {
        unsafe {
            let prop_wide = to_wide(property);
            let variant = VARIANT::from(value as i16);
            self.inner
                .Put(PCWSTR::from_raw(prop_wide.as_ptr()), 0, &variant, 0)
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to set property {}: {:?}", property, e))
                })?;
            Ok(())
        }
    }

    /// Set a u32 property value
    pub fn put_u32(&self, property: &str, value: u32) -> Result<()> {
        unsafe {
            let prop_wide = to_wide(property);
            let variant = VARIANT::from(value as i32);
            self.inner
                .Put(PCWSTR::from_raw(prop_wide.as_ptr()), 0, &variant, 0)
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to set property {}: {:?}", property, e))
                })?;
            Ok(())
        }
    }

    /// Set a u64 property value
    ///
    /// WMI uint64 properties must be set as strings (BSTR)
    pub fn put_u64(&self, property: &str, value: u64) -> Result<()> {
        // WMI expects uint64 as a string representation
        self.put_string(property, &value.to_string())
    }

    /// Set a string array property value
    pub fn put_string_array(&self, property: &str, values: &[&str]) -> Result<()> {
        use windows::Win32::System::Com::SAFEARRAYBOUND;
        use windows::Win32::System::Ole::{SafeArrayCreate, SafeArrayDestroy, SafeArrayPutElement};
        use windows::Win32::System::Variant::{VT_ARRAY, VT_BSTR};

        unsafe {
            let prop_wide = to_wide(property);

            // Create a SAFEARRAY of BSTRs
            let bounds = SAFEARRAYBOUND {
                cElements: values.len() as u32,
                lLbound: 0,
            };
            let sa = SafeArrayCreate(VT_BSTR, 1, &bounds);
            if sa.is_null() {
                return Err(HvError::WmiError("Failed to create SAFEARRAY".to_string()));
            }

            // Put each string into the array
            for (i, value) in values.iter().enumerate() {
                let bstr = BSTR::from(*value);
                let index = i as i32;
                let hr = SafeArrayPutElement(sa, &index, bstr.into_raw() as *const _);
                if hr.is_err() {
                    let _ = SafeArrayDestroy(sa);
                    return Err(HvError::WmiError(format!(
                        "Failed to put element {}: {:?}",
                        i, hr
                    )));
                }
            }

            // Create variant containing the array
            let mut variant = VARIANT::default();
            // Set VT_ARRAY | VT_BSTR
            (*variant.Anonymous.Anonymous).vt = VT_ARRAY | VT_BSTR;
            (*variant.Anonymous.Anonymous).Anonymous.parray = sa;

            let result = self
                .inner
                .Put(PCWSTR::from_raw(prop_wide.as_ptr()), 0, &variant, 0);

            // Don't destroy the array - the variant owns it now
            // SafeArrayDestroy(sa);

            result.map_err(|e| {
                HvError::WmiError(format!("Failed to set property {}: {:?}", property, e))
            })?;
            Ok(())
        }
    }

    /// Set a boolean property value
    pub fn put_bool(&self, property: &str, value: bool) -> Result<()> {
        unsafe {
            let prop_wide = to_wide(property);
            let variant = VARIANT::from(value);
            self.inner
                .Put(PCWSTR::from_raw(prop_wide.as_ptr()), 0, &variant, 0)
                .map_err(|e| {
                    HvError::WmiError(format!("Failed to set property {}: {:?}", property, e))
                })?;
            Ok(())
        }
    }

    /// Spawn a new instance from this class template
    pub fn spawn_instance(&self) -> Result<WmiObject> {
        unsafe {
            self.inner
                .SpawnInstance(0)
                .map(|obj| WmiObject { inner: obj })
                .map_err(|e| HvError::WmiError(format!("Failed to spawn instance: {:?}", e)))
        }
    }

    /// Get the object path (__PATH)
    pub fn path(&self) -> Result<String> {
        self.get_string_required("__PATH")
    }

    /// Get the relative path (__RELPATH)
    pub fn relpath(&self) -> Result<String> {
        self.get_string_required("__RELPATH")
    }

    /// Get the raw IWbemClassObject
    pub fn inner(&self) -> &IWbemClassObject {
        &self.inner
    }
}

impl Clone for WmiObject {
    fn clone(&self) -> Self {
        unsafe {
            // Clone the underlying COM object
            match self.inner.Clone() {
                Ok(cloned) => WmiObject { inner: cloned },
                Err(_) => panic!("Failed to clone WMI object"),
            }
        }
    }
}

/// Hyper-V specific WMI utilities
pub mod hyperv {
    use super::*;

    /// VM enabled states from Msvm_ComputerSystem
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EnabledState {
        Unknown = 0,
        Other = 1,
        Enabled = 2,  // Running
        Disabled = 3, // Off
        ShuttingDown = 4,
        NotApplicable = 5,
        EnabledButOffline = 6,
        InTest = 7,
        Deferred = 8,
        Quiesce = 9,
        Starting = 10,
        Paused = 32768,
        Suspended = 32769, // Saved
        Starting2 = 32770,
        Saving = 32773,
        Stopping = 32774,
        Pausing = 32776,
        Resuming = 32777,
    }

    impl From<u32> for EnabledState {
        fn from(value: u32) -> Self {
            match value {
                0 => EnabledState::Unknown,
                1 => EnabledState::Other,
                2 => EnabledState::Enabled,
                3 => EnabledState::Disabled,
                4 => EnabledState::ShuttingDown,
                5 => EnabledState::NotApplicable,
                6 => EnabledState::EnabledButOffline,
                7 => EnabledState::InTest,
                8 => EnabledState::Deferred,
                9 => EnabledState::Quiesce,
                10 => EnabledState::Starting,
                32768 => EnabledState::Paused,
                32769 => EnabledState::Suspended,
                32770 => EnabledState::Starting2,
                32773 => EnabledState::Saving,
                32774 => EnabledState::Stopping,
                32776 => EnabledState::Pausing,
                32777 => EnabledState::Resuming,
                _ => EnabledState::Unknown,
            }
        }
    }

    /// Requested state for VM state change
    #[derive(Debug, Clone, Copy)]
    pub enum RequestedState {
        Enabled = 2,  // Start VM
        Disabled = 3, // Stop VM (hard)
        Shutdown = 4, // Graceful shutdown
        Offline = 6,
        Test = 7,
        Defer = 8,
        Quiesce = 9,
        Reboot = 10,
        Reset = 11,
        Paused = 32768,
        Saved = 32769,
    }

    /// Get the Msvm_VirtualSystemManagementService singleton
    pub fn get_vsms(conn: &WmiConnection) -> Result<WmiObject> {
        let mut result = conn.query("SELECT * FROM Msvm_VirtualSystemManagementService")?;
        result
            .next()
            .ok_or_else(|| HvError::WmiError("VSMS not found".to_string()))?
    }

    /// Get the Msvm_VirtualEthernetSwitchManagementService singleton
    pub fn get_vesms(conn: &WmiConnection) -> Result<WmiObject> {
        let mut result = conn.query("SELECT * FROM Msvm_VirtualEthernetSwitchManagementService")?;
        result
            .next()
            .ok_or_else(|| HvError::WmiError("VESMS not found".to_string()))?
    }

    /// Get the Msvm_ImageManagementService singleton (for VHD operations)
    pub fn get_ims(conn: &WmiConnection) -> Result<WmiObject> {
        let mut result = conn.query("SELECT * FROM Msvm_ImageManagementService")?;
        result
            .next()
            .ok_or_else(|| HvError::WmiError("ImageManagementService not found".to_string()))?
    }

    /// Wait for a WMI job to complete
    pub fn wait_for_job(conn: &WmiConnection, job_path: &str) -> Result<()> {
        loop {
            let job = conn.get_object(job_path)?;
            let job_state = job.get_u32("JobState")?.unwrap_or(0);

            match job_state {
                // Completed
                7 => return Ok(()),
                // Exception/Error
                8 | 9 | 10 | 11 => {
                    let error_code = job.get_u32("ErrorCode")?.unwrap_or(0);
                    let error_desc = job
                        .get_string("ErrorDescription")?
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(HvError::WmiError(format!(
                        "Job failed (code {}): {}",
                        error_code, error_desc
                    )));
                }
                // Running states - wait
                2 | 3 | 4 => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    /// Check return value and wait for job if needed
    pub fn check_job_result(conn: &WmiConnection, result: &WmiObject) -> Result<()> {
        let return_value = result.get_u32("ReturnValue")?.unwrap_or(0);

        match return_value {
            // Completed with no error
            0 => Ok(()),
            // Job started - wait for completion
            4096 => {
                let job_path = result.get_string_required("Job")?;
                wait_for_job(conn, &job_path)
            }
            // Error
            code => Err(HvError::WmiError(format!(
                "Operation failed with code {}",
                code
            ))),
        }
    }
}
