use crate::{BtResult, CtfPlugin, CtfPluginSrcExt, Value};
use std::{ffi::CStr, fmt, str::FromStr};
use tracing::debug;

/// When the message iterator does not find the specified remote tracing
/// session (SESSION part of the inputs parameter), do one of the following actions.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum SessionNotFoundAction {
    /// Keep on trying, returning "try again later" to the downstream user until the tracing session exists.
    ///
    /// With this action, the message iterator never ends, as the LTTng live protocol cannot currently
    /// indicate that a tracing session will never exist.
    ///
    /// This is the default behavior if not specified.
    Continue,

    /// Produce a failure
    Fail,

    /// End the iterator
    End,
}

/// See <https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.lttng-live.7/#doc-_initialization_parameters>
pub struct CtfPluginSourceLttnLiveInitParams {
    params: Value,
    _inputs_val: Value,
    _session_not_found_action_val: Option<Value>,
}

impl CtfPluginSourceLttnLiveInitParams {
    pub const INPUTS_KEY: &'static [u8] = b"inputs\0";
    pub const SESSION_NOT_FOUND_ACTION_KEY: &'static [u8] = b"session-not-found-action\0";

    pub fn new(
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.lttng-live.7/#doc-param-inputs
        url: &CStr,
        // https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.lttng-live.7/#doc-param-session-not-found-action
        session_not_found_action: Option<SessionNotFoundAction>,
    ) -> BtResult<Self> {
        debug!(
            "Creating source.ctf.lttng-live init params: url={:?}, session-not-found-action={:?}",
            url, session_not_found_action
        );

        let mut params = Value::new_map()?;

        let mut inputs_val = Value::new_array()?;
        inputs_val.append_string_element(url)?;
        params.insert_entry(Self::inputs_key(), &inputs_val)?;

        let session_not_found_action_val = if let Some(action) = session_not_found_action {
            let val = Value::new_string_with(action.to_cstr())?;
            params.insert_entry(Self::session_not_found_action_key(), &val)?;
            val.into()
        } else {
            None
        };

        Ok(CtfPluginSourceLttnLiveInitParams {
            params,
            _inputs_val: inputs_val,
            _session_not_found_action_val: session_not_found_action_val,
        })
    }

    pub fn params(&self) -> &Value {
        &self.params
    }

    fn inputs_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::INPUTS_KEY) }
    }

    fn session_not_found_action_key() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::SESSION_NOT_FOUND_ACTION_KEY) }
    }
}

impl CtfPluginSrcExt for CtfPluginSourceLttnLiveInitParams {
    fn parameters(&self) -> &Value {
        self.params()
    }

    fn source_component_class_name(&self) -> &'static CStr {
        CtfPlugin::lttng_live_name()
    }
}

impl SessionNotFoundAction {
    fn to_cstr(self) -> &'static CStr {
        use SessionNotFoundAction::*;
        match self {
            Continue => unsafe { CStr::from_bytes_with_nul_unchecked(b"continue\0") },
            Fail => unsafe { CStr::from_bytes_with_nul_unchecked(b"fail\0") },
            End => unsafe { CStr::from_bytes_with_nul_unchecked(b"end\0") },
        }
    }
}

impl FromStr for SessionNotFoundAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use SessionNotFoundAction::*;
        Ok(match s.trim().to_lowercase().as_str() {
            "continue" => Continue,
            "fail" => Fail,
            "end" => End,
            _ => return Err(format!("{} is not a valid action", s)),
        })
    }
}

impl fmt::Display for SessionNotFoundAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SessionNotFoundAction::*;
        let s = match self {
            Continue => "continue",
            Fail => "fail",
            End => "end",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstrings_are_valid() {
        assert_ne!(
            CtfPluginSourceLttnLiveInitParams::inputs_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            CtfPluginSourceLttnLiveInitParams::session_not_found_action_key()
                .to_str()
                .unwrap()
                .len(),
            0
        );

        assert_ne!(
            SessionNotFoundAction::Continue
                .to_cstr()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            SessionNotFoundAction::Fail
                .to_cstr()
                .to_str()
                .unwrap()
                .len(),
            0
        );
        assert_ne!(
            SessionNotFoundAction::End.to_cstr().to_str().unwrap().len(),
            0
        );
    }
}
