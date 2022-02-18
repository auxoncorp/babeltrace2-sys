use babeltrace2_sys::*;
use std::ffi::{CString, NulError};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{process, thread};
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt, Debug)]
enum Cmd {
    /// Use the source.ctf.fs component to read CTF data from disk
    Fs {
        /// Print trace/stream properties and exit
        #[structopt(long)]
        no_events: bool,

        /// Set the name of the trace object that the component creates, overriding the data's trace
        /// name if present
        #[structopt(long)]
        trace_name: Option<String>,

        /// Add offset-ns nanoseconds to the offset of all the clock classes that the component creates
        #[structopt(long = "offset-ns")]
        clock_class_offset_ns: Option<i64>,

        /// Add offset-s seconds to the offset of all the clock classes that the component creates
        #[structopt(long = "offset-s")]
        clock_class_offset_s: Option<i64>,

        /// Force the origin of all clock classes that the component creates to have a Unix epoch origin
        #[structopt(long = "unix-epoch")]
        force_clock_class_origin_unix_epoch: Option<bool>,

        /// Path to trace directories
        #[structopt(name = "input", required = true, min_values = 1)]
        inputs: Vec<PathBuf>,
    },

    /// Use the source.ctf.lttng-live component to read from a local or remote LTTng relay daemon
    LttngLive {
        /// Print trace/stream properties and exit
        #[structopt(long)]
        no_events: bool,

        /// When babeltrace2 needs to retry to run
        /// the graph later, retry in retry-duration-us µs
        /// (default: 100000)
        #[structopt(long, short = "r", default_value = "100000")]
        retry_duration_us: u64,

        /// When the message iterator does not find the specified remote tracing
        /// session (SESSION part of the inputs parameter), do one of the following actions.
        /// * continue (default)
        /// * fail
        /// * end
        #[structopt(long, verbatim_doc_comment)]
        session_not_found_action: Option<SessionNotFoundAction>,

        /// The URL to connect to the LTTng relay daemon.
        ///
        /// Format: net[4]://RDHOST[:RDPORT]/host/TGTHOST/SESSION
        /// * RDHOST
        ///   LTTng relay daemon’s host name or IP address.
        /// * RDPORT
        ///   LTTng relay daemon’s listening port.
        ///   If not specified, the component uses the default port (5344).
        /// * TGTHOST
        ///   Target’s host name or IP address.
        /// * SESSION
        ///   Name of the LTTng tracing session from which to receive data.
        ///
        /// Example: net://localhost/host/ubuntu-focal/my-kernel-session
        #[structopt(verbatim_doc_comment)]
        url: Url,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match do_main() {
        Err(e) => {
            log::error!("{}", e);
            Err(e)
        }
        Ok(()) => Ok(()),
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let opts = Opts::from_args();

    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Exiting...");
        } else {
            process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    match opts.cmd {
        Cmd::Fs {
            no_events,
            trace_name,
            clock_class_offset_ns,
            clock_class_offset_s,
            force_clock_class_origin_unix_epoch,
            inputs,
        } => {
            let trace_name: Option<CString> =
                trace_name.map(|n| CString::new(n.as_bytes())).transpose()?;
            let input_cstrings: Vec<CString> = inputs
                .iter()
                .map(|p| CString::new(p.as_os_str().as_bytes()))
                .collect::<Result<Vec<CString>, NulError>>()?;
            let inputs = input_cstrings
                .iter()
                .map(|i| i.as_c_str())
                .collect::<Vec<_>>();

            let params = CtfPluginSourceFsInitParams::new(
                trace_name.as_deref(),
                clock_class_offset_ns,
                clock_class_offset_s,
                force_clock_class_origin_unix_epoch,
                &inputs,
            )?;

            let ctf_iter = CtfIterator::new(LoggingLevel::Warn, &params)?;

            println!("------------------------------------------------------------");
            println!("Trace Properties");
            println!("------------------------------------------------------------");
            println!("{:#?}", ctf_iter.trace_properties());
            println!();

            println!("------------------------------------------------------------");
            println!("Stream Properties");
            println!("------------------------------------------------------------");
            for s in ctf_iter.stream_properties().iter() {
                println!("{:#?}", s);
            }
            println!();

            if !no_events {
                for event in ctf_iter {
                    if running.load(Ordering::SeqCst) != 0 {
                        break;
                    }

                    let event = event?;
                    println!("{}", event);
                }
            }
        }
        Cmd::LttngLive {
            no_events,
            retry_duration_us,
            url,
            session_not_found_action,
        } => {
            let url = CString::new(url.to_string().as_bytes())?;
            let params = CtfPluginSourceLttnLiveInitParams::new(&url, session_not_found_action)?;

            let mut ctf_stream = CtfStream::new(LoggingLevel::Warn, &params)?;

            let retry_duration = Duration::from_micros(retry_duration_us);
            let mut metadata_shown = false;

            loop {
                if running.load(Ordering::SeqCst) != 0 {
                    break;
                }

                match ctf_stream.update()? {
                    RunStatus::Ok => (),
                    RunStatus::TryAgain => {
                        thread::sleep(retry_duration);
                        continue;
                    }
                    RunStatus::End => break,
                }

                if ctf_stream.has_metadata() && !metadata_shown {
                    metadata_shown = true;
                    println!("------------------------------------------------------------");
                    println!("Trace Properties");
                    println!("------------------------------------------------------------");
                    println!("{:#?}", ctf_stream.trace_properties());
                    println!();

                    println!("------------------------------------------------------------");
                    println!("Stream Properties");
                    println!("------------------------------------------------------------");
                    for s in ctf_stream.stream_properties().iter() {
                        println!("{:#?}", s);
                    }
                    println!();

                    if no_events {
                        break;
                    }
                }

                for event in ctf_stream.events_chunk() {
                    if running.load(Ordering::SeqCst) != 0 {
                        break;
                    }

                    println!("{}", event);
                }
            }
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
