use babeltrace2_sys::*;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    /// Path to trace directory
    #[structopt(long)]
    path: PathBuf,

    /// Print trace/stream properties and exit
    #[structopt(long)]
    no_events: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let opts = Opts::from_args();

    log::info!("Using path '{}'", opts.path.display());
    let path = CString::new(opts.path.as_os_str().as_bytes())?;

    let ctf_params = CtfPluginSourceInitParams::new(None, None, None, None, &[&path])?;

    let trace_iter = TraceIterator::new(LoggingLevel::None, ctf_params)?;

    println!("{:#?}", trace_iter.trace_properties());
    println!("{:#?}", trace_iter.stream_properties());

    if !opts.no_events {
        for event in trace_iter {
            let event = event?;
            println!("{}", event);
        }
    }

    Ok(())
}

/// Plugin descriptor related data, pointers to this data
/// will end up in special linker sections in the binary
/// so libbabeltrace2 can discover it
///
/// TODO: figure out how to work around https://github.com/rust-lang/rust/issues/47384
pub mod proxy_plugin_descriptors {
    use babeltrace2_sys::ffi::*;
    use babeltrace2_sys::proxy_plugin_descriptors::*;

    #[used]
    #[link_section = "__bt_plugin_descriptors"]
    pub static PLUGIN_DESC_PTR: __bt_plugin_descriptor_ptr =
        __bt_plugin_descriptor_ptr(&PLUGIN_DESC);

    #[used]
    #[link_section = "__bt_plugin_component_class_descriptors"]
    pub static SINK_COMP_DESC_PTR: __bt_plugin_component_class_descriptor_ptr =
        __bt_plugin_component_class_descriptor_ptr(&SINK_COMP_DESC);

    #[used]
    #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
    pub static SINK_COMP_CLASS_INIT_ATTR_PTR: __bt_plugin_component_class_descriptor_attribute_ptr =
        __bt_plugin_component_class_descriptor_attribute_ptr(&SINK_COMP_CLASS_INIT_ATTR);

    #[used]
    #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
    pub static SINK_COMP_CLASS_FINI_ATTR_PTR: __bt_plugin_component_class_descriptor_attribute_ptr =
        __bt_plugin_component_class_descriptor_attribute_ptr(&SINK_COMP_CLASS_FINI_ATTR);

    #[used]
    #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
    pub static SINK_COMP_CLASS_GRAPH_CONF_ATTR_PTR:
        __bt_plugin_component_class_descriptor_attribute_ptr =
        __bt_plugin_component_class_descriptor_attribute_ptr(&SINK_COMP_CLASS_GRAPH_CONF_ATTR);
}

pub mod utils_plugin_descriptors {
    use babeltrace2_sys::ffi::*;

    #[link(name = "babeltrace-plugin-utils", kind = "static")]
    extern "C" {
        pub static __bt_plugin_descriptor_auto_ptr: *const __bt_plugin_descriptor;
    }
}

pub mod ctf_plugin_descriptors {
    use babeltrace2_sys::ffi::*;

    #[link(name = "babeltrace-plugin-ctf", kind = "static")]
    extern "C" {
        pub static __bt_plugin_descriptor_auto_ptr: *const __bt_plugin_descriptor;
    }
}
