use crate::{
    ffi, BtResult, BtResultExt, ComponentClassFilter, ComponentClassSink, ComponentClassSource,
    Error,
};
use std::{ffi::CStr, ptr};

pub struct Plugin {
    inner: *const ffi::bt_plugin,
}

impl Plugin {
    pub fn load_from_statics_by_name(name: &CStr) -> BtResult<Self> {
        log::debug!("Loading static plugin '{}'", name.to_string_lossy());

        let find_in_std_env_var = 0;
        let find_in_user_dir = 0;
        let find_in_sys_dir = 0;
        let find_in_static = 1;
        let fail_on_load_error = 0;

        let mut inner = ptr::null();
        unsafe {
            ffi::bt_plugin_find(
                name.as_ptr(),
                find_in_std_env_var,
                find_in_user_dir,
                find_in_sys_dir,
                find_in_static,
                fail_on_load_error,
                &mut inner,
            )
        }
        .capi_result()?;

        Ok(Plugin { inner })
    }

    pub fn borrow_source_component_class_by_name(
        &self,
        name: &CStr,
    ) -> BtResult<ComponentClassSource> {
        log::debug!("Borrowing source component '{}'", name.to_string_lossy());
        let inner = unsafe {
            ffi::bt_plugin_borrow_source_component_class_by_name_const(self.inner, name.as_ptr())
        };
        if inner.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(ComponentClassSource { inner })
        }
    }

    pub fn borrow_sink_component_class_by_name(&self, name: &CStr) -> BtResult<ComponentClassSink> {
        log::debug!("Borrowing sink component '{}'", name.to_string_lossy());
        let inner = unsafe {
            ffi::bt_plugin_borrow_sink_component_class_by_name_const(self.inner, name.as_ptr())
        };
        if inner.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(ComponentClassSink { inner })
        }
    }

    pub fn borrow_filter_component_class_by_name(
        &self,
        name: &CStr,
    ) -> BtResult<ComponentClassFilter> {
        log::debug!("Borrowing filter component '{}'", name.to_string_lossy());
        let inner = unsafe {
            ffi::bt_plugin_borrow_filter_component_class_by_name_const(self.inner, name.as_ptr())
        };
        if inner.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(ComponentClassFilter { inner })
        }
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        unsafe { ffi::bt_plugin_put_ref(self.inner) };
    }
}
