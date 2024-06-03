use crate::{ffi, util, BtResult, Env, EnvValue, Error};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::{ptr, slice};
use tracing::error;
use uuid::Uuid;

pub struct Trace {
    pub(crate) inner: *const ffi::bt_trace,
}

impl Trace {
    pub fn properties(&self) -> BtResult<TraceProperties> {
        use ffi::bt_value_type::*;

        let name_cstr = unsafe { ffi::bt_trace_get_name(self.inner) };
        let name = util::opt_owned_cstr(name_cstr)?;

        let uuid_raw = unsafe { ffi::bt_trace_get_uuid(self.inner) };
        let uuid = if uuid_raw.is_null() {
            None
        } else {
            let bytes: uuid::Bytes = unsafe { slice::from_raw_parts(uuid_raw, 16) }
                .try_into()
                .map_err(|_| Error::Uuid)?;
            Uuid::from_bytes(bytes).into()
        };

        let env_count = unsafe { ffi::bt_trace_get_environment_entry_count(self.inner) };
        let env = if env_count == 0 {
            None
        } else {
            let mut entries: BTreeMap<String, EnvValue> = Default::default();
            for idx in 0..env_count {
                let mut env_name = ptr::null();
                let mut env_val = ptr::null();
                unsafe {
                    ffi::bt_trace_borrow_environment_entry_by_index_const(
                        self.inner,
                        idx,
                        &mut env_name,
                        &mut env_val,
                    )
                };
                if let Some(key) = util::opt_owned_cstr(env_name)? {
                    match unsafe { ffi::bt_value_get_type(env_val) } {
                        BT_VALUE_TYPE_SIGNED_INTEGER => {
                            let v = unsafe { ffi::bt_value_integer_signed_get(env_val) };
                            entries.insert(key, EnvValue::Integer(v));
                        }
                        BT_VALUE_TYPE_STRING => {
                            let v = unsafe { ffi::bt_value_string_get(env_val) };
                            if let Some(val) = util::opt_owned_cstr(v)? {
                                entries.insert(key, EnvValue::String(val));
                            }
                        }
                        typ => {
                            error!(
                                "Environment value for key '{}' must be either string or integer (got {})",
                                key, typ
                            );
                            unsafe { ffi::bt_value_put_ref(env_val) };
                            return Err(Error::EnvValue);
                        }
                    }
                }
            }
            Env { entries }.into()
        };

        Ok(TraceProperties { name, uuid, env })
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TraceProperties {
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
    pub env: Option<Env>,
}
