use crate::{ffi, BtResult, Error, InputPort, OutputPort};
use std::ffi::CStr;

pub type ComponentSource = Component<ffi::bt_component_source>;
pub type ComponentFilter = Component<ffi::bt_component_filter>;
pub type ComponentSink = Component<ffi::bt_component_sink>;

/// An immutably borrowed component
pub struct Component<T> {
    pub(crate) inner: *const T,
}

impl<T> Component<T> {
    pub const IN_PORT_NAME: &'static [u8] = b"in\0";
    pub const OUT_PORT_NAME: &'static [u8] = b"out\0";

    pub fn in_port_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::IN_PORT_NAME) }
    }

    pub fn out_port_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::OUT_PORT_NAME) }
    }
}

impl ComponentSource {
    pub fn get_output_port_count(&self) -> u64 {
        unsafe { ffi::bt_component_source_get_output_port_count(self.inner) }
    }

    pub fn borrow_output_port_by_index(&self, index: u64) -> BtResult<OutputPort> {
        let port = unsafe {
            ffi::bt_component_source_borrow_output_port_by_index_const(self.inner, index)
        };
        if port.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(OutputPort { inner: port })
        }
    }
}

impl ComponentFilter {
    pub fn borrow_input_port_by_index(&self, index: u64) -> BtResult<InputPort> {
        let port =
            unsafe { ffi::bt_component_filter_borrow_input_port_by_index_const(self.inner, index) };
        if port.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(InputPort { inner: port })
        }
    }

    pub fn borrow_output_port_by_index(&self, index: u64) -> BtResult<OutputPort> {
        let port = unsafe {
            ffi::bt_component_filter_borrow_output_port_by_index_const(self.inner, index)
        };
        if port.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(OutputPort { inner: port })
        }
    }
}

impl ComponentSink {
    pub fn get_input_port_count(&self) -> u64 {
        unsafe { ffi::bt_component_sink_get_input_port_count(self.inner) }
    }

    pub fn borrow_input_port_by_index(&self, index: u64) -> BtResult<InputPort> {
        let port =
            unsafe { ffi::bt_component_sink_borrow_input_port_by_index_const(self.inner, index) };
        if port.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(InputPort { inner: port })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstrings_are_valid() {
        assert_ne!(ComponentSource::in_port_name().to_str().unwrap().len(), 0);
    }
}
