use crate::{ffi, BtResult, BtResultExt, Error, MessageIterator, SelfComponentInputPort};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;

pub type SelfComponentSink = SelfComponent<ffi::bt_self_component_sink>;
pub type SelfComponentSource = SelfComponent<ffi::bt_self_component_source>;

pub struct SelfComponent<T = ffi::bt_self_component> {
    pub(crate) inner: *mut T,
}

impl SelfComponent<ffi::bt_self_component> {
    pub fn from_raw(inner: *mut ffi::bt_self_component) -> Self {
        debug_assert!(!inner.is_null());
        SelfComponent { inner }
    }

    // TODO - remove this once high-level types/API exists
    pub fn inner_mut(&mut self) -> *mut ffi::bt_self_component {
        self.inner
    }
}

impl<T> SelfComponent<T> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn set_c_user_data_ptr(&mut self, user_data: *mut c_void) {
        unsafe { ffi::bt_self_component_set_data(self.inner as _, user_data) };
    }

    pub fn get_c_user_data_ptr(&mut self) -> *mut c_void {
        unsafe { ffi::bt_self_component_get_data(self.inner as _) }
    }
}

impl SelfComponentSink {
    pub fn from_raw(inner: *mut ffi::bt_self_component_sink) -> Self {
        debug_assert!(!inner.is_null());
        SelfComponentSink { inner }
    }

    /// You can only call this function from within
    /// the initialization, "input port connected", and "output port connected" methods.
    pub fn add_input_port(&mut self, name: &CStr) -> BtResult<()> {
        unsafe {
            ffi::bt_self_component_sink_add_input_port(
                self.inner,
                name.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        }
        .capi_result()
    }

    pub fn borrow_input_port_by_index(&mut self, index: u64) -> BtResult<SelfComponentInputPort> {
        let port =
            unsafe { ffi::bt_self_component_sink_borrow_input_port_by_index(self.inner, index) };
        if port.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(SelfComponentInputPort { inner: port })
        }
    }

    pub fn create_message_iterator(
        &mut self,
        port: &SelfComponentInputPort,
    ) -> BtResult<MessageIterator> {
        let mut iter = ptr::null_mut();
        unsafe {
            ffi::bt_message_iterator_create_from_sink_component(self.inner, port.inner, &mut iter)
        }
        .capi_result()?;
        Ok(MessageIterator { inner: iter })
    }
}

impl SelfComponentSource {
    pub fn from_raw(inner: *mut ffi::bt_self_component_source) -> Self {
        debug_assert!(!inner.is_null());
        SelfComponentSource { inner }
    }

    pub fn add_output_port(&mut self, name: &CStr) -> BtResult<()> {
        unsafe {
            ffi::bt_self_component_source_add_output_port(
                self.inner,
                name.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        }
        .capi_result()
    }

    pub fn upcast(&self) -> SelfComponent<ffi::bt_self_component> {
        SelfComponent {
            inner: self.inner as *mut _,
        }
    }
}
