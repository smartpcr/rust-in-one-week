use crate::error::{Error, Result};
use core::ptr;
use windows::core::BSTR;
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::SAFEARRAYBOUND;
use windows::Win32::System::Ole::{
    SafeArrayCreate, SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound,
    SafeArrayPutElement,
};
use windows::Win32::System::Variant::*;

/// Trait for converting from VARIANT to Rust types.
pub trait FromVariant: Sized {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>>;
}

/// Trait for converting Rust types to VARIANT.
pub trait ToVariant {
    fn to_variant(&self) -> VARIANT;
}

impl FromVariant for std::string::String {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            if vt == VT_BSTR {
                let bstr = &v.Anonymous.Anonymous.Anonymous.bstrVal;
                // Deref ManuallyDrop to get the inner BSTR, then convert to String
                let s = std::string::String::try_from(&**bstr).unwrap_or_default();
                return Ok(Some(s));
            }
            Err(Error::TypeConversion {
                property: "unknown",
                expected: "String",
            })
        }
    }
}

impl FromVariant for u16 {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            match vt {
                VT_UI2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.uiVal)),
                VT_I2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.iVal as u16)),
                VT_UI4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.ulVal as u16)),
                VT_I4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.lVal as u16)),
                _ => Err(Error::TypeConversion {
                    property: "unknown",
                    expected: "u16",
                }),
            }
        }
    }
}

impl FromVariant for u32 {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            match vt {
                VT_UI4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.ulVal)),
                VT_I4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.lVal as u32)),
                VT_UI2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.uiVal as u32)),
                VT_I2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.iVal as u32)),
                _ => Err(Error::TypeConversion {
                    property: "unknown",
                    expected: "u32",
                }),
            }
        }
    }
}

impl FromVariant for u64 {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            match vt {
                VT_UI8 => Ok(Some(v.Anonymous.Anonymous.Anonymous.ullVal)),
                VT_I8 => Ok(Some(v.Anonymous.Anonymous.Anonymous.llVal as u64)),
                VT_UI4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.ulVal as u64)),
                VT_I4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.lVal as u64)),
                VT_BSTR => {
                    // WMI sometimes returns uint64 as string
                    let bstr = &v.Anonymous.Anonymous.Anonymous.bstrVal;
                    // Deref ManuallyDrop to get the inner BSTR, then convert to String
                    let s = String::try_from(&**bstr).unwrap_or_default();
                    s.parse().map(Some).map_err(|_| Error::TypeConversion {
                        property: "unknown",
                        expected: "u64",
                    })
                }
                _ => Err(Error::TypeConversion {
                    property: "unknown",
                    expected: "u64",
                }),
            }
        }
    }
}

impl FromVariant for i32 {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            match vt {
                VT_I4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.lVal)),
                VT_I2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.iVal as i32)),
                VT_UI4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.ulVal as i32)),
                VT_UI2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.uiVal as i32)),
                _ => Err(Error::TypeConversion {
                    property: "unknown",
                    expected: "i32",
                }),
            }
        }
    }
}

impl FromVariant for bool {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            match vt {
                VT_BOOL => Ok(Some(v.Anonymous.Anonymous.Anonymous.boolVal.as_bool())),
                VT_I4 => Ok(Some(v.Anonymous.Anonymous.Anonymous.lVal != 0)),
                VT_I2 => Ok(Some(v.Anonymous.Anonymous.Anonymous.iVal != 0)),
                _ => Err(Error::TypeConversion {
                    property: "unknown",
                    expected: "bool",
                }),
            }
        }
    }
}

impl FromVariant for Vec<String> {
    fn from_variant(v: &VARIANT) -> Result<Option<Self>> {
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            if vt == VT_NULL || vt == VT_EMPTY {
                return Ok(None);
            }
            if (vt.0 & VT_ARRAY.0) == 0 {
                return Err(Error::TypeConversion {
                    property: "unknown",
                    expected: "String array",
                });
            }

            let parray = v.Anonymous.Anonymous.Anonymous.parray;
            if parray.is_null() {
                return Ok(Some(Vec::new()));
            }

            let lbound = SafeArrayGetLBound(&*parray, 1)?;
            let ubound = SafeArrayGetUBound(&*parray, 1)?;

            let mut result = Vec::new();
            for i in lbound..=ubound {
                let mut bstr = BSTR::new();
                SafeArrayGetElement(&*parray, &i, &mut bstr as *mut _ as *mut _)?;
                result.push(String::try_from(&bstr).unwrap_or_default());
            }
            Ok(Some(result))
        }
    }
}

impl ToVariant for &str {
    fn to_variant(&self) -> VARIANT {
        let bstr = BSTR::from(*self);
        unsafe {
            let mut v = VARIANT::default();
            ptr::write(
                &mut (*v.Anonymous.Anonymous).vt,
                VT_BSTR,
            );
            ptr::write(
                &mut (*v.Anonymous.Anonymous).Anonymous.bstrVal,
                std::mem::ManuallyDrop::new(bstr),
            );
            v
        }
    }
}

impl ToVariant for String {
    fn to_variant(&self) -> VARIANT {
        self.as_str().to_variant()
    }
}

impl ToVariant for u16 {
    fn to_variant(&self) -> VARIANT {
        unsafe {
            let mut v = VARIANT::default();
            ptr::write(&mut (*v.Anonymous.Anonymous).vt, VT_UI2);
            ptr::write(&mut (*v.Anonymous.Anonymous).Anonymous.uiVal, *self);
            v
        }
    }
}

impl ToVariant for u32 {
    fn to_variant(&self) -> VARIANT {
        unsafe {
            let mut v = VARIANT::default();
            ptr::write(&mut (*v.Anonymous.Anonymous).vt, VT_UI4);
            ptr::write(&mut (*v.Anonymous.Anonymous).Anonymous.ulVal, *self);
            v
        }
    }
}

impl ToVariant for u64 {
    fn to_variant(&self) -> VARIANT {
        unsafe {
            let mut v = VARIANT::default();
            ptr::write(&mut (*v.Anonymous.Anonymous).vt, VT_UI8);
            ptr::write(&mut (*v.Anonymous.Anonymous).Anonymous.ullVal, *self);
            v
        }
    }
}

impl ToVariant for i32 {
    fn to_variant(&self) -> VARIANT {
        unsafe {
            let mut v = VARIANT::default();
            ptr::write(&mut (*v.Anonymous.Anonymous).vt, VT_I4);
            ptr::write(&mut (*v.Anonymous.Anonymous).Anonymous.lVal, *self);
            v
        }
    }
}

impl ToVariant for bool {
    fn to_variant(&self) -> VARIANT {
        unsafe {
            let mut v = VARIANT::default();
            ptr::write(&mut (*v.Anonymous.Anonymous).vt, VT_BOOL);
            ptr::write(
                &mut (*v.Anonymous.Anonymous).Anonymous.boolVal,
                if *self { VARIANT_TRUE } else { VARIANT_FALSE },
            );
            v
        }
    }
}

impl ToVariant for &[&str] {
    fn to_variant(&self) -> VARIANT {
        unsafe {
            let bounds = SAFEARRAYBOUND {
                cElements: self.len() as u32,
                lLbound: 0,
            };
            let parray = SafeArrayCreate(VT_BSTR, 1, &bounds);

            for (i, s) in self.iter().enumerate() {
                let bstr = BSTR::from(*s);
                let idx = i as i32;
                let _ = SafeArrayPutElement(parray, &idx, bstr.into_raw() as *const _);
            }

            let mut v = VARIANT::default();
            ptr::write(
                &mut (*v.Anonymous.Anonymous).vt,
                VARENUM(VT_ARRAY.0 | VT_BSTR.0),
            );
            ptr::write(&mut (*v.Anonymous.Anonymous).Anonymous.parray, parray);
            v
        }
    }
}

impl ToVariant for Vec<String> {
    fn to_variant(&self) -> VARIANT {
        let refs: Vec<&str> = self.iter().map(|s| s.as_str()).collect();
        refs.as_slice().to_variant()
    }
}
