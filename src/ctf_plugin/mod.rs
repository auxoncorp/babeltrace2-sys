use crate::{BtResult, ComponentClassSource, Plugin, Value};
pub use fs::CtfPluginSourceFsInitParams;
pub use lttng_live::{CtfPluginSourceLttnLiveInitParams, SessionNotFoundAction};
use std::ffi::CStr;

mod fs;
mod lttng_live;

/// See <https://babeltrace.org/docs/v2.0/man7/babeltrace2-source.ctf.fs.7/>
pub struct CtfPlugin(Plugin);

impl CtfPlugin {
    pub const PLUGIN_NAME: &'static [u8] = b"ctf\0";
    pub const FS_COMP_NAME: &'static [u8] = b"fs\0";
    pub const LTTNG_LIVE_COMP_NAME: &'static [u8] = b"lttng-live\0";
    pub const GRAPH_NODE_NAME: &'static [u8] = b"source.ctf\0";

    pub fn load() -> BtResult<Self> {
        let name = Self::plugin_name();
        Ok(CtfPlugin(Plugin::load_from_statics_by_name(name)?))
    }

    pub fn borrow_source_component_class_by_name(
        &self,
        name: &CStr,
    ) -> BtResult<ComponentClassSource> {
        self.0.borrow_source_component_class_by_name(name)
    }

    pub fn plugin_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PLUGIN_NAME) }
    }

    pub fn fs_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::FS_COMP_NAME) }
    }

    pub fn lttng_live_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::LTTNG_LIVE_COMP_NAME) }
    }

    pub fn graph_node_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::GRAPH_NODE_NAME) }
    }
}

pub(crate) trait CtfPluginSrcExt {
    fn parameters(&self) -> &Value;
    fn source_component_class_name(&self) -> &'static CStr;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstrings_are_valid() {
        assert_ne!(CtfPlugin::plugin_name().to_str().unwrap().len(), 0);
        assert_ne!(CtfPlugin::fs_name().to_str().unwrap().len(), 0);
        assert_ne!(CtfPlugin::lttng_live_name().to_str().unwrap().len(), 0);
        assert_ne!(CtfPlugin::graph_node_name().to_str().unwrap().len(), 0);
    }
}
