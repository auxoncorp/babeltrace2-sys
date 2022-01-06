use crate::BtResult;
use std::ffi::CStr;
use std::os::raw::c_char;

pub(crate) fn opt_owned_cstr(ptr: *const c_char) -> BtResult<Option<String>> {
    if ptr.is_null() {
        Ok(None)
    } else {
        let s = unsafe { CStr::from_ptr(ptr) }.to_str()?;
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(s.to_string().into())
        }
    }
}
