use crate::error::{Error, Result};
use windows::core::{BSTR, HSTRING, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket,
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL,
    RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
use windows::Win32::System::Wmi::{
    IEnumWbemClassObject, IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator,
    WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE,
};

use std::cell::Cell;

thread_local! {
    static COM_INITIALIZED: Cell<bool> = const { Cell::new(false) };
}

/// Hyper-V WMI namespace.
pub const HYPERV_NAMESPACE: &str = r"root\virtualization\v2";

/// WMI connection wrapper for Hyper-V operations.
pub struct WmiConnection {
    services: IWbemServices,
}

impl std::fmt::Debug for WmiConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WmiConnection").finish_non_exhaustive()
    }
}

impl WmiConnection {
    /// Connect to the Hyper-V WMI namespace.
    pub fn connect() -> Result<Self> {
        Self::connect_to(HYPERV_NAMESPACE)
    }

    /// Connect to a specific WMI namespace.
    pub fn connect_to(namespace: &str) -> Result<Self> {
        unsafe {
            // Initialize COM if not already done
            Self::init_com()?;

            // Create WbemLocator
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
                .map_err(Error::WmiConnection)?;

            // Connect to namespace
            let namespace_bstr = BSTR::from(namespace);
            let services = locator
                .ConnectServer(
                    &namespace_bstr,
                    &BSTR::new(),
                    &BSTR::new(),
                    &BSTR::new(),
                    0,
                    &BSTR::new(),
                    None,
                )
                .map_err(Error::WmiConnection)?;

            // Set security on the proxy
            CoSetProxyBlanket(
                &services,
                RPC_C_AUTHN_WINNT,
                RPC_C_AUTHZ_NONE,
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            )
            .map_err(Error::WmiConnection)?;

            Ok(Self { services })
        }
    }

    /// Initialize COM for the current thread.
    fn init_com() -> Result<()> {
        COM_INITIALIZED.with(|initialized| {
            if !initialized.get() {
                unsafe {
                    // Initialize COM
                    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

                    // Set security levels
                    let _ = CoInitializeSecurity(
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
                }
                initialized.set(true);
            }
            Ok(())
        })
    }

    /// Execute a WQL query and return all results.
    pub fn query(&self, wql: &str) -> Result<Vec<IWbemClassObject>> {
        unsafe {
            let query_lang = BSTR::from("WQL");
            let query_str = BSTR::from(wql);

            let enumerator = self
                .services
                .ExecQuery(
                    &query_lang,
                    &query_str,
                    WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                    None,
                )
                .map_err(|e| Error::WmiQuery {
                    query: wql.to_string(),
                    source: e,
                })?;

            self.collect_results(enumerator)
        }
    }

    /// Execute a WQL query and return the first result.
    pub fn query_first(&self, wql: &str) -> Result<Option<IWbemClassObject>> {
        let results = self.query(wql)?;
        Ok(results.into_iter().next())
    }

    /// Get a single object by path.
    pub fn get_object(&self, path: &str) -> Result<IWbemClassObject> {
        unsafe {
            let path_bstr = BSTR::from(path);
            let mut obj = None;
            self.services
                .GetObject(&path_bstr, Default::default(), None, Some(&mut obj), None)
                .map_err(|e| Error::WmiQuery {
                    query: path.to_string(),
                    source: e,
                })?;
            obj.ok_or_else(|| Error::WmiQuery {
                query: path.to_string(),
                source: windows_core::Error::from_hresult(windows_core::HRESULT(-1)),
            })
        }
    }

    /// Get a class definition for spawning instances.
    pub fn get_class(&self, class_name: &str) -> Result<IWbemClassObject> {
        self.get_object(class_name)
    }

    /// Spawn a new instance of a WMI class.
    pub fn spawn_instance(&self, class_name: &str) -> Result<IWbemClassObject> {
        let class = self.get_class(class_name)?;
        let class_name_owned = class_name.to_string();
        unsafe {
            class.SpawnInstance(0).map_err(move |e| Error::WmiMethod {
                class: Box::leak(class_name_owned.into_boxed_str()),
                method: "SpawnInstance",
                source: e,
            })
        }
    }

    /// Execute a method on a WMI object.
    pub fn exec_method(
        &self,
        object_path: &str,
        method_name: &str,
        in_params: Option<&IWbemClassObject>,
    ) -> Result<IWbemClassObject> {
        let method_name_owned = method_name.to_string();
        unsafe {
            let path_bstr = BSTR::from(object_path);
            let method_bstr = BSTR::from(method_name);

            let mut out_params = None;
            self.services
                .ExecMethod(
                    &path_bstr,
                    &method_bstr,
                    Default::default(),
                    None,
                    in_params,
                    Some(&mut out_params),
                    None,
                )
                .map_err(|e| Error::WmiMethod {
                    class: "Object",
                    method: Box::leak(method_name_owned.clone().into_boxed_str()),
                    source: e,
                })?;

            out_params.ok_or_else(|| Error::WmiMethod {
                class: "Object",
                method: Box::leak(method_name_owned.into_boxed_str()),
                source: windows_core::Error::from_hresult(windows_core::HRESULT(-1)),
            })
        }
    }

    /// Get method definition for preparing input parameters.
    pub fn get_method_params(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Result<IWbemClassObject> {
        let class = self.get_class(class_name)?;
        let class_name_owned = class_name.to_string();
        let method_name_owned = method_name.to_string();
        unsafe {
            let method_hstring = HSTRING::from(method_name);
            let mut in_params = None;
            let mut out_params = None;
            class
                .GetMethod(PCWSTR(method_hstring.as_ptr()), 0, &mut in_params, &mut out_params)
                .map_err(|e| Error::WmiMethod {
                    class: Box::leak(class_name_owned.clone().into_boxed_str()),
                    method: Box::leak(method_name_owned.clone().into_boxed_str()),
                    source: e,
                })?;

            in_params
                .map(|p| p.SpawnInstance(0))
                .transpose()
                .map_err(|e| Error::WmiMethod {
                    class: Box::leak(class_name_owned.clone().into_boxed_str()),
                    method: Box::leak(method_name_owned.clone().into_boxed_str()),
                    source: e,
                })?
                .ok_or_else(|| Error::WmiMethod {
                    class: Box::leak(class_name_owned.into_boxed_str()),
                    method: Box::leak(method_name_owned.into_boxed_str()),
                    source: windows_core::Error::from_hresult(windows_core::HRESULT(-1)),
                })
        }
    }

    /// Access the underlying IWbemServices.
    pub fn services(&self) -> &IWbemServices {
        &self.services
    }

    /// Collect all results from an enumerator.
    fn collect_results(&self, enumerator: IEnumWbemClassObject) -> Result<Vec<IWbemClassObject>> {
        let mut results = Vec::new();
        loop {
            let mut objects: [Option<IWbemClassObject>; 1] = [None];
            let mut returned = 0u32;

            unsafe {
                let hr = enumerator.Next(WBEM_INFINITE, &mut objects, &mut returned);
                if hr.is_err() || returned == 0 {
                    break;
                }
                if let Some(obj) = objects[0].take() {
                    results.push(obj);
                }
            }
        }
        Ok(results)
    }
}

/// Extension trait for IWbemClassObject property access.
pub trait WbemClassObjectExt {
    /// Get a string property.
    fn get_string_prop(&self, name: &str) -> Result<Option<std::string::String>>;

    /// Get a required string property.
    fn get_string_prop_required(&self, name: &str) -> Result<std::string::String>;

    /// Get a u16 property.
    fn get_u16(&self, name: &str) -> Result<Option<u16>>;

    /// Get a u32 property.
    fn get_u32(&self, name: &str) -> Result<Option<u32>>;

    /// Get a u64 property.
    fn get_u64(&self, name: &str) -> Result<Option<u64>>;

    /// Get a bool property.
    fn get_bool(&self, name: &str) -> Result<Option<bool>>;

    /// Get the WMI object path (__PATH).
    fn get_path(&self) -> Result<std::string::String>;

    /// Get the relative path (__RELPATH).
    fn get_relpath(&self) -> Result<std::string::String>;

    /// Set a string property.
    fn put_string(&self, name: &str, value: &str) -> Result<()>;

    /// Set a u16 property.
    fn put_u16(&self, name: &str, value: u16) -> Result<()>;

    /// Set a u32 property.
    fn put_u32(&self, name: &str, value: u32) -> Result<()>;

    /// Set a u64 property.
    fn put_u64(&self, name: &str, value: u64) -> Result<()>;

    /// Set a bool property.
    fn put_bool(&self, name: &str, value: bool) -> Result<()>;

    /// Set a string array property.
    fn put_string_array(&self, name: &str, values: &[&str]) -> Result<()>;

    /// Get the object as an embedded object string (MOF).
    fn get_text(&self) -> Result<std::string::String>;
}

impl WbemClassObjectExt for IWbemClassObject {
    fn get_string_prop(&self, name: &str) -> Result<Option<std::string::String>> {
        use windows::Win32::System::Variant::{VARIANT, VT_BSTR, VT_NULL, VT_EMPTY};

        unsafe {
            let name_hstring = HSTRING::from(name);
            let mut value = VARIANT::default();
            let hr = self.Get(PCWSTR(name_hstring.as_ptr()), 0, &mut value, None, None);
            if hr.is_err() {
                return Ok(None);
            }
            let vt = value.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            if vt == VT_BSTR {
                let bstr = &value.Anonymous.Anonymous.Anonymous.bstrVal;
                let s: std::string::String = std::string::String::try_from(&**bstr).unwrap_or_default();
                return Ok(Some(s));
            }
            Err(Error::TypeConversion {
                property: "unknown",
                expected: "String",
            })
        }
    }

    fn get_string_prop_required(&self, name: &str) -> Result<std::string::String> {
        self.get_string_prop(name)?
            .ok_or_else(|| Error::MissingRequired(Box::leak(name.to_string().into_boxed_str())))
    }

    fn get_u16(&self, name: &str) -> Result<Option<u16>> {
        use super::variant::FromVariant;
        use windows::Win32::System::Variant::VARIANT;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let mut value = VARIANT::default();
            let hr = self.Get(PCWSTR(name_hstring.as_ptr()), 0, &mut value, None, None);
            if hr.is_err() {
                return Ok(None);
            }
            u16::from_variant(&value)
        }
    }

    fn get_u32(&self, name: &str) -> Result<Option<u32>> {
        use super::variant::FromVariant;
        use windows::Win32::System::Variant::VARIANT;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let mut value = VARIANT::default();
            let hr = self.Get(PCWSTR(name_hstring.as_ptr()), 0, &mut value, None, None);
            if hr.is_err() {
                return Ok(None);
            }
            u32::from_variant(&value)
        }
    }

    fn get_u64(&self, name: &str) -> Result<Option<u64>> {
        use super::variant::FromVariant;
        use windows::Win32::System::Variant::VARIANT;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let mut value = VARIANT::default();
            let hr = self.Get(PCWSTR(name_hstring.as_ptr()), 0, &mut value, None, None);
            if hr.is_err() {
                return Ok(None);
            }
            u64::from_variant(&value)
        }
    }

    fn get_bool(&self, name: &str) -> Result<Option<bool>> {
        use super::variant::FromVariant;
        use windows::Win32::System::Variant::VARIANT;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let mut value = VARIANT::default();
            let hr = self.Get(PCWSTR(name_hstring.as_ptr()), 0, &mut value, None, None);
            if hr.is_err() {
                return Ok(None);
            }
            bool::from_variant(&value)
        }
    }

    fn get_path(&self) -> Result<std::string::String> {
        self.get_string_prop_required("__PATH")
    }

    fn get_relpath(&self) -> Result<std::string::String> {
        self.get_string_prop_required("__RELPATH")
    }

    fn put_string(&self, name: &str, value: &str) -> Result<()> {
        use super::variant::ToVariant;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let variant = value.to_variant();
            self.Put(PCWSTR(name_hstring.as_ptr()), 0, &variant, 0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "Put",
                    source: e,
                })
        }
    }

    fn put_u16(&self, name: &str, value: u16) -> Result<()> {
        use super::variant::ToVariant;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let variant = value.to_variant();
            self.Put(PCWSTR(name_hstring.as_ptr()), 0, &variant, 0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "Put",
                    source: e,
                })
        }
    }

    fn put_u32(&self, name: &str, value: u32) -> Result<()> {
        use super::variant::ToVariant;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let variant = value.to_variant();
            self.Put(PCWSTR(name_hstring.as_ptr()), 0, &variant, 0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "Put",
                    source: e,
                })
        }
    }

    fn put_u64(&self, name: &str, value: u64) -> Result<()> {
        use super::variant::ToVariant;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let variant = value.to_variant();
            self.Put(PCWSTR(name_hstring.as_ptr()), 0, &variant, 0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "Put",
                    source: e,
                })
        }
    }

    fn put_bool(&self, name: &str, value: bool) -> Result<()> {
        use super::variant::ToVariant;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let variant = value.to_variant();
            self.Put(PCWSTR(name_hstring.as_ptr()), 0, &variant, 0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "Put",
                    source: e,
                })
        }
    }

    fn put_string_array(&self, name: &str, values: &[&str]) -> Result<()> {
        use super::variant::ToVariant;

        unsafe {
            let name_hstring = HSTRING::from(name);
            let variant = values.to_variant();
            self.Put(PCWSTR(name_hstring.as_ptr()), 0, &variant, 0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "Put",
                    source: e,
                })
        }
    }

    fn get_text(&self) -> Result<std::string::String> {
        unsafe {
            let text = self.GetObjectText(0)
                .map_err(|e| Error::WmiMethod {
                    class: "IWbemClassObject",
                    method: "GetObjectText",
                    source: e,
                })?;
            Ok(std::string::String::try_from(&text).unwrap_or_default())
        }
    }
}
