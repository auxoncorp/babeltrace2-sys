use crate::{BtResult, CtfPlugin, CtfPluginSinkExt, CtfPluginSrcExt, Error, Value};
use std::ffi::CStr;
use tracing::debug;

/// See <https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/#doc-_initialization_parameters>
pub struct CtfPluginSourceFsInitParams {
    params: Value,
    _inputs_val: Value,
    _trace_name_val: Option<Value>,
    _offset_ns_val: Option<Value>,
    _offset_sec_val: Option<Value>,
    _force_epoch_val: Option<Value>,
}

impl CtfPluginSourceFsInitParams {
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
        debug!(
            "Creating source.ctf.fs init params: trace-name={:?}, clock-class-offset-ns={:?}, clock-class-offset-s={:?}, force-clock-class-origin-unix-epoch={:?}, inputs={:?}",
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

        Ok(CtfPluginSourceFsInitParams {
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

impl CtfPluginSrcExt for CtfPluginSourceFsInitParams {
    fn parameters(&self) -> &Value {
        self.params()
    }

    fn source_component_class_name(&self) -> &'static CStr {
        CtfPlugin::fs_name()
    }
}

/// See <https://babeltrace.org/docs/v2.0/man7/babeltrace2-sink.ctf.fs.7/#doc-_initialization_parameters>
pub struct CtfPluginSinkFsInitParams {
    params: Value,
    _path: Value,
    _assume_single_trace: Option<Value>,
    _ignore_discarded_events: Option<Value>,
    _ignore_discarded_packets: Option<Value>,
    _quiet: Option<Value>,
}

impl CtfPluginSinkFsInitParams {
    pub const PATH_KEY: &'static [u8] = b"path\0";
    pub const ASSUME_SINGLE_TRACE_KEY: &'static [u8] = b"assume-single-trace\0";
    pub const IGNORE_DISCARDED_EVENTS_KEY: &'static [u8] = b"ignore-discarded-events\0";
    pub const IGNORE_DISCARDED_PACKETS_KEY: &'static [u8] = b"ignore-discarded-packets\0";
    pub const QUIET_KEY: &'static [u8] = b"quiet\0";

    pub fn new(
        assume_single_trace: Option<bool>,
        ignore_discarded_events: Option<bool>,
        ignore_discarded_packets: Option<bool>,
        quiet: Option<bool>,
        path: &CStr,
    ) -> BtResult<Self> {
        debug!("Creating sink.ctf.fs init params: assume-single-trace={:?}, ignore-discarded-events={:?}, ignore-discarded-packets={:?}, quiet={:?}, path={:?}",
            assume_single_trace, ignore_discarded_events, ignore_discarded_packets, quiet, path);

        let mut params = Value::new_map()?;

        let assume_single_trace = if let Some(val) = assume_single_trace {
            let val = Value::new_bool_with(val)?;
            params.insert_entry(Self::assume_single_trace_key(), &val)?;
            val.into()
        } else {
            None
        };

        let ignore_discarded_events = if let Some(val) = ignore_discarded_events {
            let val = Value::new_bool_with(val)?;
            params.insert_entry(Self::ignore_discarded_events_key(), &val)?;
            val.into()
        } else {
            None
        };

        let ignore_discarded_packets = if let Some(val) = ignore_discarded_packets {
            let val = Value::new_bool_with(val)?;
            params.insert_entry(Self::ignore_discarded_packets_key(), &val)?;
            val.into()
        } else {
            None
        };

        let quiet = if let Some(val) = quiet {
            let val = Value::new_bool_with(val)?;
            params.insert_entry(Self::quiet_key(), &val)?;
            val.into()
        } else {
            None
        };

        let path_val = Value::new_string_with(path)?;
        params.insert_entry(Self::path_key(), &path_val)?;

        Ok(CtfPluginSinkFsInitParams {
            params,
            _path: path_val,
            _assume_single_trace: assume_single_trace,
            _ignore_discarded_events: ignore_discarded_events,
            _ignore_discarded_packets: ignore_discarded_packets,
            _quiet: quiet,
        })
    }

    pub fn params(&self) -> &Value {
        &self.params
    }

    fn assume_single_trace_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::ASSUME_SINGLE_TRACE_KEY) }
    }

    fn ignore_discarded_events_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::IGNORE_DISCARDED_EVENTS_KEY) }
    }

    fn ignore_discarded_packets_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::IGNORE_DISCARDED_PACKETS_KEY) }
    }

    fn quiet_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::QUIET_KEY) }
    }

    fn path_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PATH_KEY) }
    }
}

impl CtfPluginSinkExt for CtfPluginSinkFsInitParams {
    fn parameters(&self) -> &Value {
        self.params()
    }

    fn sink_component_class_name(&self) -> &'static CStr {
        CtfPlugin::fs_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_cstrings_are_valid() {
        assert_ne!(
            CtfPluginSourceFsInitParams::inputs_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceFsInitParams::trace_name_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceFsInitParams::offset_ns_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceFsInitParams::offset_sec_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceFsInitParams::force_epoch_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn sink_cstrings_are_valid() {
        assert_ne!(
            CtfPluginSinkFsInitParams::assume_single_trace_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSinkFsInitParams::ignore_discarded_events_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSinkFsInitParams::ignore_discarded_packets_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSinkFsInitParams::quiet_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSinkFsInitParams::path_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
    }
}
