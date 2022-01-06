use crate::{BtResult, ComponentClassFilter, Plugin};
use std::ffi::CStr;

/// https://babeltrace.org/docs/v2.0/man7/babeltrace2-filter.utils.muxer.7/
pub struct UtilsPlugin(Plugin);

impl UtilsPlugin {
    pub const PLUGIN_NAME: &'static [u8] = b"utils\0";
    pub const MUXER_COMP_NAME: &'static [u8] = b"muxer\0";
    pub const GRAPH_NODE_NAME: &'static [u8] = b"filter.utils.muxer\0";

    pub fn load() -> BtResult<Self> {
        let name = Self::plugin_name();
        Ok(UtilsPlugin(Plugin::load_from_statics_by_name(name)?))
    }

    pub fn borrow_muxer_filter_component_class(&self) -> BtResult<ComponentClassFilter> {
        let name = Self::muxer_name();
        self.0.borrow_filter_component_class_by_name(name)
    }

    pub fn plugin_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PLUGIN_NAME) }
    }

    pub fn muxer_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::MUXER_COMP_NAME) }
    }

    pub fn graph_node_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::GRAPH_NODE_NAME) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstrings_are_valid() {
        assert_ne!(UtilsPlugin::plugin_name().to_str().unwrap().len(), 0);
        assert_ne!(UtilsPlugin::muxer_name().to_str().unwrap().len(), 0);
        assert_ne!(UtilsPlugin::graph_node_name().to_str().unwrap().len(), 0);
    }
}
