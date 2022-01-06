use crate::{ffi, BtResult, BtResultExt, Error};
use std::ffi::CStr;

/// Generic, JSON-like basic data containers
pub struct Value {
    pub(crate) inner: *mut ffi::bt_value,
}

impl Value {
    pub(crate) fn new_map() -> BtResult<Self> {
        let inner = unsafe { ffi::bt_value_map_create() };
        if inner.is_null() {
            Err(Error::Memory)
        } else {
            Ok(Value { inner })
        }
    }

    pub(crate) fn new_array() -> BtResult<Self> {
        let inner = unsafe { ffi::bt_value_array_create() };
        if inner.is_null() {
            Err(Error::Memory)
        } else {
            Ok(Value { inner })
        }
    }

    pub(crate) fn new_string_with(value: &CStr) -> BtResult<Self> {
        let inner = unsafe { ffi::bt_value_string_create_init(value.as_ptr()) };
        if inner.is_null() {
            Err(Error::Memory)
        } else {
            Ok(Value { inner })
        }
    }

    pub(crate) fn new_signed_int_with(value: i64) -> BtResult<Self> {
        let inner = unsafe { ffi::bt_value_integer_signed_create_init(value) };
        if inner.is_null() {
            Err(Error::Memory)
        } else {
            Ok(Value { inner })
        }
    }

    pub(crate) fn new_bool_with(value: bool) -> BtResult<Self> {
        let inner = unsafe { ffi::bt_value_bool_create_init(value as _) };
        if inner.is_null() {
            Err(Error::Memory)
        } else {
            Ok(Value { inner })
        }
    }

    pub(crate) fn insert_entry(&mut self, key: &CStr, value: &Value) -> BtResult<()> {
        debug_assert_eq!(
            unsafe { ffi::bt_value_get_type(self.inner) },
            ffi::bt_value_type::BT_VALUE_TYPE_MAP
        );
        unsafe { ffi::bt_value_map_insert_entry(self.inner, key.as_ptr(), value.inner) }
            .capi_result()
    }

    pub(crate) fn append_string_element(&mut self, value: &CStr) -> BtResult<()> {
        debug_assert_eq!(
            unsafe { ffi::bt_value_get_type(self.inner) },
            ffi::bt_value_type::BT_VALUE_TYPE_ARRAY
        );
        unsafe { ffi::bt_value_array_append_string_element(self.inner, value.as_ptr()) }
            .capi_result()
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe { ffi::bt_value_put_ref(self.inner) };
    }
}
