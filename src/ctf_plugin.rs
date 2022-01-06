use crate::{BtResult, ComponentClassSource, Error, Plugin, Value};
use std::ffi::CStr;

/// https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/
pub struct CtfPlugin(Plugin);

impl CtfPlugin {
    pub const PLUGIN_NAME: &'static [u8] = b"ctf\0";
    pub const FS_COMP_NAME: &'static [u8] = b"fs\0";
    pub const GRAPH_NODE_NAME: &'static [u8] = b"source.ctf.fs\0";

    pub fn load() -> BtResult<Self> {
        let name = Self::plugin_name();
        Ok(CtfPlugin(Plugin::load_from_statics_by_name(name)?))
    }

    pub fn borrow_fs_source_component_class(&self) -> BtResult<ComponentClassSource> {
        let name = Self::fs_name();
        self.0.borrow_source_component_class_by_name(name)
    }

    pub fn plugin_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PLUGIN_NAME) }
    }

    pub fn fs_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::FS_COMP_NAME) }
    }

    pub fn graph_node_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::GRAPH_NODE_NAME) }
    }
}

/// https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-_initialization_parameters
pub struct CtfPluginSourceInitParams {
    params: Value,
    _inputs_val: Value,
    _trace_name_val: Option<Value>,
    _offset_ns_val: Option<Value>,
    _offset_sec_val: Option<Value>,
    _force_epoch_val: Option<Value>,
}

impl CtfPluginSourceInitParams {
    pub const TRACE_NAME_KEY: &'static [u8] = b"trace-name\0";
    pub const CLOCK_CLASS_OFFSET_NS_KEY: &'static [u8] = b"clock-class-offset-ns\0";
    pub const CLOCK_CLASS_OFFSET_S_KEY: &'static [u8] = b"clock-class-offset-s\0";
    pub const FORCE_CLOCK_CLASS_ORIGIN_UNIX_EPOCH_KEY: &'static [u8] =
        b"force-clock-class-origin-unix-epoch\0";
    pub const INPUTS_KEY: &'static [u8] = b"inputs\0";

    pub fn new(
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-param-trace-name
        trace_name: Option<&CStr>,
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-param-clock-class-offset-ns
        clock_class_offset_ns: Option<i64>,
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-param-clock-class-offset-s
        clock_class_offset_s: Option<i64>,
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-param-force-clock-class-origin-unix-epoch
        force_clock_class_origin_unix_epoch: Option<bool>,
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-param-inputs
        inputs: &[&CStr],
    ) -> BtResult<Self> {
        log::debug!(
            "Creating CTF-source init params: trace-name={:?}, clock-class-offset-ns={:?}, clock-class-offset-s={:?}, force-clock-class-origin-unix-epoch={:?}, inputs={:?}",
            trace_name,
            clock_class_offset_ns,
            clock_class_offset_s,
            force_clock_class_origin_unix_epoch,
            inputs,
        );

        if inputs.is_empty() {
            return Err(Error::CtfSourceRequiresInputs);
        }

        let mut params = Value::new_map()?;

        let mut inputs_val = Value::new_array()?;
        for input_path in inputs.iter() {
            inputs_val.append_string_element(input_path)?;
        }
        params.insert_entry(Self::inputs_key(), &inputs_val)?;

        let trace_name_val = if let Some(trace_name) = trace_name {
            let val = Value::new_string_with(trace_name)?;
            params.insert_entry(Self::trace_name_key(), &val)?;
            val.into()
        } else {
            None
        };

        let offset_ns_val = if let Some(offset_ns) = clock_class_offset_ns {
            let val = Value::new_signed_int_with(offset_ns)?;
            params.insert_entry(Self::offset_ns_key(), &val)?;
            val.into()
        } else {
            None
        };

        let offset_sec_val = if let Some(offset_sec) = clock_class_offset_s {
            let val = Value::new_signed_int_with(offset_sec)?;
            params.insert_entry(Self::offset_sec_key(), &val)?;
            val.into()
        } else {
            None
        };

        let force_epoch_val = if let Some(force_epoch) = force_clock_class_origin_unix_epoch {
            let val = Value::new_bool_with(force_epoch)?;
            params.insert_entry(Self::force_epoch_key(), &val)?;
            val.into()
        } else {
            None
        };

        Ok(CtfPluginSourceInitParams {
            params,
            _inputs_val: inputs_val,
            _trace_name_val: trace_name_val,
            _offset_ns_val: offset_ns_val,
            _offset_sec_val: offset_sec_val,
            _force_epoch_val: force_epoch_val,
        })
    }

    pub fn params(&self) -> &Value {
        &self.params
    }

    fn inputs_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::INPUTS_KEY) }
    }

    fn trace_name_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::TRACE_NAME_KEY) }
    }

    fn offset_ns_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::CLOCK_CLASS_OFFSET_NS_KEY) }
    }

    fn offset_sec_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::CLOCK_CLASS_OFFSET_S_KEY) }
    }

    fn force_epoch_key() -> &'static CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(Self::FORCE_CLOCK_CLASS_ORIGIN_UNIX_EPOCH_KEY)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstrings_are_valid() {
        assert_ne!(CtfPlugin::plugin_name().to_str().unwrap().len(), 0);
        assert_ne!(CtfPlugin::fs_name().to_str().unwrap().len(), 0);
        assert_ne!(CtfPlugin::graph_node_name().to_str().unwrap().len(), 0);

        assert_ne!(
            CtfPluginSourceInitParams::inputs_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceInitParams::trace_name_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceInitParams::offset_ns_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceInitParams::offset_sec_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceInitParams::force_epoch_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
    }
}
