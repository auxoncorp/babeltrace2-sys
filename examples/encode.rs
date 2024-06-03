use babeltrace2_sys::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::{process, ptr};
use tracing::{error, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match do_main() {
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
        Ok(()) => Ok(()),
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let log_level = LoggingLevel::Warn;

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

    let output_path = CString::new("/tmp/trace_out")?;

    let params = CtfPluginSinkFsInitParams::new(
        Some(true), // assume_single_trace,
        None,       // ignore_discarded_events,
        None,       //ignore_discarded_packets,
        Some(true), // quiet,
        &output_path,
    )?;

    let state: Box<dyn SourcePluginHandler> = Box::new(ExamplePluginState::new());
    let state = Box::new(state);

    let mut pipeline = EncoderPipeline::new::<ExamplePlugin>(log_level, state, &params)?;

    loop {
        let last_run_status = pipeline.graph.run_once()?;

        info!("last_run_status = {:?}", last_run_status);

        if RunStatus::End == last_run_status {
            break;
        }
    }

    Ok(())
}

pub struct ExamplePluginState {
    stream: *mut ffi::bt_stream,
    event_class: *mut ffi::bt_event_class,
    num_events: usize,
}

impl ExamplePluginState {
    fn new() -> Self {
        Self {
            stream: ptr::null_mut(),
            event_class: ptr::null_mut(),
            num_events: 0,
        }
    }

    unsafe fn create_event_class(&mut self, stream_class: *mut ffi::bt_stream_class) {
        let name = CString::new("event_foo").unwrap();

        let trace_class = ffi::bt_stream_class_borrow_trace_class(stream_class);

        self.event_class = ffi::bt_event_class_create(stream_class);

        ffi::bt_event_class_set_name(self.event_class, name.as_c_str().as_ptr());

        let payload_field_class = ffi::bt_field_class_structure_create(trace_class);

        let msg_field_class = ffi::bt_field_class_string_create(trace_class);

        let msg_field = CString::new("field_bar").unwrap();
        ffi::bt_field_class_structure_append_member(
            payload_field_class,
            msg_field.as_c_str().as_ptr(),
            msg_field_class,
        );

        ffi::bt_event_class_set_payload_field_class(self.event_class, payload_field_class);

        // Put the references we don't need anymore
        ffi::bt_field_class_put_ref(payload_field_class);
        ffi::bt_field_class_put_ref(msg_field_class);
    }

    unsafe fn create_message(
        &self,
        msg_iter: *mut ffi::bt_self_message_iterator,
    ) -> *const ffi::bt_message {
        let timestamp = 1000000000 * self.num_events;

        let msg = ffi::bt_message_event_create_with_default_clock_snapshot(
            msg_iter,
            self.event_class,
            self.stream,
            timestamp as u64,
        );

        let event = ffi::bt_message_event_borrow_event(msg);
        let payload_field = ffi::bt_event_borrow_payload_field(event);
        let msg_field = ffi::bt_field_structure_borrow_member_field_by_index(payload_field, 0);

        let val = b"this is a message\0";
        ffi::bt_field_string_set_value(msg_field, val.as_ptr() as _);

        msg as *const _
    }
}

impl SourcePluginHandler for ExamplePluginState {
    fn initialize(&mut self, mut component: SelfComponent) -> Result<(), Error> {
        // Create the source component's metadata and stream objects
        info!("Creating metadata and stream objects");

        unsafe {
            let trace_class = ffi::bt_trace_class_create(component.inner_mut());

            let stream_class = ffi::bt_stream_class_create(trace_class);

            let clock_class = ffi::bt_clock_class_create(component.inner_mut());

            ffi::bt_stream_class_set_default_clock_class(stream_class, clock_class);

            self.create_event_class(stream_class);

            let trace = ffi::bt_trace_create(trace_class);

            let trace_name = b"my_trace\0";
            ffi::bt_trace_set_name(trace, trace_name.as_ptr() as _);

            //self.stream
            self.stream = ffi::bt_stream_create(stream_class, trace);

            // Put the references we don't need anymore
            ffi::bt_trace_put_ref(trace);
            ffi::bt_clock_class_put_ref(clock_class);
            ffi::bt_stream_class_put_ref(stream_class);
            ffi::bt_trace_class_put_ref(trace_class as *const _);
        };

        Ok(())
    }

    fn finalize(&mut self, _component: SelfComponent) -> Result<(), Error> {
        unsafe {
            ffi::bt_event_class_put_ref(self.event_class);
            ffi::bt_stream_put_ref(self.stream);
        }
        Ok(())
    }

    fn iterator_next(
        &mut self,
        mut msg_iter: SelfMessageIterator,
        messages: &mut [*const ffi::bt_message],
    ) -> Result<MessageIteratorStatus, Error> {
        let mut num_msgs = 0;

        if self.num_events == 0 {
            let msg = unsafe {
                ffi::bt_message_stream_beginning_create(msg_iter.inner_mut(), self.stream)
            };
            messages[num_msgs] = msg;
            num_msgs += 1;
        } else if self.num_events >= 10 {
            return Ok(MessageIteratorStatus::Done);
        }

        let msg = unsafe { self.create_message(msg_iter.inner_mut()) };
        messages[num_msgs] = msg;
        num_msgs += 1;
        self.num_events += 1;

        if self.num_events == 10 {
            let msg =
                unsafe { ffi::bt_message_stream_end_create(msg_iter.inner_mut(), self.stream) };
            messages[num_msgs] = msg;
            num_msgs += 1;
        }

        Ok(MessageIteratorStatus::Messages(num_msgs as u64))
    }
}

struct ExamplePlugin;

impl SourcePluginDescriptor for ExamplePlugin {
    /// Provides source.example.output
    const PLUGIN_NAME: &'static [u8] = b"example\0";
    const OUTPUT_COMP_NAME: &'static [u8] = b"output\0";
    const GRAPH_NODE_NAME: &'static [u8] = b"source.example.output\0";

    fn load() -> BtResult<Plugin> {
        let name = Self::plugin_name();
        Ok(Plugin::load_from_statics_by_name(name)?)
    }

    fn plugin_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PLUGIN_NAME) }
    }

    fn output_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::OUTPUT_COMP_NAME) }
    }

    fn graph_node_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::GRAPH_NODE_NAME) }
    }
}

source_plugin_descriptors!(ExamplePlugin);

pub mod utils_plugin_descriptors {
    use babeltrace2_sys::ffi::*;

    #[link(
        name = "babeltrace-plugin-utils",
        kind = "static",
        modifiers = "+whole-archive"
    )]
    extern "C" {
        pub static __bt_plugin_descriptor_auto_ptr: *const __bt_plugin_descriptor;
    }
}

pub mod ctf_plugin_descriptors {
    use babeltrace2_sys::ffi::*;

    #[link(
        name = "babeltrace-plugin-ctf",
        kind = "static",
        modifiers = "+whole-archive"
    )]
    extern "C" {
        pub static __bt_plugin_descriptor_auto_ptr: *const __bt_plugin_descriptor;
    }
}
