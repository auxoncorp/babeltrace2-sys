use crate::{
    ffi, BtResult, ComponentClassSink, ComponentSink, Error, Message, MessageIterator, MessageType,
    NextStatus, OwnedEvent, Plugin, SelfComponentSink, StreamProperties, TraceProperties,
};
use std::collections::{BTreeSet, VecDeque};
use std::convert::{AsMut, AsRef};
use std::ffi::{c_void, CStr};

/// An output sink that funnels relevant trace information to the caller
pub struct ProxyPlugin(Plugin);

impl ProxyPlugin {
    /// Provides sink.proxy.output
    pub const PLUGIN_NAME: &'static [u8] = b"proxy\0";
    pub const OUTPUT_COMP_NAME: &'static [u8] = b"output\0";
    pub const GRAPH_NODE_NAME: &'static [u8] = b"sink.proxy.output\0";

    pub fn load() -> BtResult<Self> {
        let name = Self::plugin_name();
        Ok(ProxyPlugin(Plugin::load_from_statics_by_name(name)?))
    }

    pub fn borrow_output_sink_component_class_by_name(&self) -> BtResult<ComponentClassSink> {
        let name = Self::output_name();
        self.0.borrow_sink_component_class_by_name(name)
    }

    pub fn plugin_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::PLUGIN_NAME) }
    }

    pub fn output_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::OUTPUT_COMP_NAME) }
    }

    pub fn graph_node_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(Self::GRAPH_NODE_NAME) }
    }
}

#[derive(Default)]
pub struct ProxyPluginState {
    pub(crate) msg_iter: Option<MessageIterator>,
    pub(crate) trace_properties: TraceProperties,
    pub(crate) stream_properties: BTreeSet<StreamProperties>,
    pub(crate) events: VecDeque<OwnedEvent>,
}

/// Plugin state, dynamically allocated, shared with the caller and
/// the underlying plugin implementation
///
/// NOTE: lifetime must be >= to the plugin lifetime (until proxy_sink_finalize is called)
pub struct BoxedRawProxyPluginState(*mut ProxyPluginState);

impl BoxedRawProxyPluginState {
    pub fn new() -> Self {
        BoxedRawProxyPluginState(Box::into_raw(Box::new(ProxyPluginState::default())))
    }

    pub(crate) fn as_raw(&mut self) -> *mut ProxyPluginState {
        self.0
    }
}

impl AsRef<ProxyPluginState> for BoxedRawProxyPluginState {
    fn as_ref(&self) -> &ProxyPluginState {
        unsafe { &(*self.0) }
    }
}

impl AsMut<ProxyPluginState> for BoxedRawProxyPluginState {
    fn as_mut(&mut self) -> &mut ProxyPluginState {
        unsafe { &mut (*self.as_raw()) }
    }
}

impl Default for BoxedRawProxyPluginState {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BoxedRawProxyPluginState {
    fn drop(&mut self) {
        debug_assert!(!self.0.is_null());
        unsafe { drop(Box::from_raw(self.0)) };
    }
}

pub type ConsumeSuccessCode = ffi::bt_component_class_sink_consume_method_status::Type;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, err_derive::Error)]
pub enum ConsumeError {
    #[error(display = "Plugin state is NULL")]
    NullState,

    #[error(display = "Message iterator is NULL")]
    NullIterator,

    #[error(display = "Message iterator returned an error. {}", _0)]
    MessageIterator(Error),

    #[error(display = "Failed to borrow stream. {}", _0)]
    StreamBorrow(Error),

    #[error(display = "Failed to borrow event. {}", _0)]
    EventBorrow(Error),

    // Catch-all
    #[error(display = "{}", _0)]
    Error(#[error(source, from)] Error),
}

impl ProxyPluginState {
    fn consume(&mut self) -> Result<ConsumeSuccessCode, ConsumeError> {
        use ffi::bt_component_class_sink_consume_method_status::*;

        // Consume a batch of messages from the upstream message iterator
        let msg_iter = self.msg_iter.as_mut().ok_or(ConsumeError::NullIterator)?;
        let (next_status, msg_array) = msg_iter
            .next_message_array()
            .map_err(ConsumeError::MessageIterator)?;

        let retcode = match next_status {
            NextStatus::Ok => {
                let messages = msg_array.as_slice();
                log::trace!("Proxy sink consuming {} messages", messages.len());
                for msg_ref in messages.iter() {
                    let msg = Message::from_raw(*msg_ref);
                    let msg_type = msg.get_type();
                    match msg_type {
                        // Populate trace and stream properties at the beginning, this is idempotent
                        // when updating a trace upon encountering multiple stream beginning messages
                        // as they all refer to a single trace
                        MessageType::StreamBeginning => {
                            let stream = msg
                                .stream_beginning_borrow_stream()
                                .map_err(ConsumeError::StreamBorrow)?;
                            let props = stream.properties()?;
                            self.stream_properties.insert(props);

                            let trace = stream.trace()?;
                            self.trace_properties = trace.properties()?;
                        }
                        MessageType::Event => {
                            let event = msg
                                .borrow_event()
                                .map_err(ConsumeError::EventBorrow)?
                                .to_owned()?;
                            self.events.push_back(event);
                        }
                        // TODO - make this a type we surface
                        MessageType::DiscardedEvents => log::debug!(
                            "Tracer discarded events in trace UUID={:?}",
                            self.trace_properties.uuid
                        ),
                        MessageType::DiscardedPackets => log::debug!(
                            "Tracer discarded packets in trace UUID={:?}",
                            self.trace_properties.uuid
                        ),
                        _ => (),
                    }
                }

                BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_OK
            }
            NextStatus::End => {
                let _ = self.msg_iter.take(); // Done with iterator, drop it now
                BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_END
            }
            NextStatus::TryAgain => BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_AGAIN,
        };
        Ok(retcode)
    }
}

#[no_mangle]
extern "C" fn proxy_sink_initialize(
    sink: *mut ffi::bt_self_component_sink,
    _config: *mut ffi::bt_self_component_sink_configuration,
    _params: *const ffi::bt_value,
    initialize_method_data: *mut c_void,
) -> ffi::bt_component_class_initialize_method_status::Type {
    use ffi::bt_component_class_initialize_method_status::*;

    log::debug!("Initializing plugin");

    if initialize_method_data.is_null() {
        log::error!("Proxy plugin state is NULL");
        return BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_ERROR;
    }

    // Set the component's user data to our private
    let mut sink = SelfComponentSink::from_raw(sink);
    sink.set_c_user_data_ptr(initialize_method_data);

    // Add an input port named `in` to the sink component
    // This is needed so that this sink component can be connected to a
    // filter or a source component. With a connected upstream
    // component, this sink component can create a message iterator
    // to consume messages.
    if sink.add_input_port(ComponentSink::in_port_name()).is_err() {
        log::error!("Failed to add proxy plugin input port");
        BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_ERROR
    } else {
        BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_OK
    }
}

#[no_mangle]
extern "C" fn proxy_sink_finalize(_sink: *mut ffi::bt_self_component_sink) {
    log::debug!("Finalizing plugin");
}

#[no_mangle]
extern "C" fn proxy_sink_graph_is_configured(
    sink: *mut ffi::bt_self_component_sink,
) -> ffi::bt_component_class_sink_graph_is_configured_method_status::Type {
    use ffi::bt_component_class_sink_graph_is_configured_method_status::*;

    log::debug!("Graph sink component configured");

    let mut sink = SelfComponentSink::from_raw(sink);
    let state = sink.get_c_user_data_ptr() as *mut ProxyPluginState;
    if state.is_null() {
        log::error!("Plugin state is NULL");
        return BT_COMPONENT_CLASS_SINK_GRAPH_IS_CONFIGURED_METHOD_STATUS_ERROR;
    }

    // Borrow our port
    let in_port = if let Ok(p) = sink.borrow_input_port_by_index(0) {
        p
    } else {
        log::error!("Failed to borrow proxy sink inport port");
        return BT_COMPONENT_CLASS_SINK_GRAPH_IS_CONFIGURED_METHOD_STATUS_ERROR;
    };

    // Create the uptream message iterator
    let msg_iter = if let Ok(iter) = sink.create_message_iterator(&in_port) {
        iter
    } else {
        log::error!("Failed to create message iterator from proxy sink component");
        return BT_COMPONENT_CLASS_SINK_GRAPH_IS_CONFIGURED_METHOD_STATUS_ERROR;
    };

    let s = unsafe { &mut (*state) };
    s.msg_iter.replace(msg_iter);

    BT_COMPONENT_CLASS_SINK_GRAPH_IS_CONFIGURED_METHOD_STATUS_OK
}

#[no_mangle]
extern "C" fn proxy_sink_consume(
    sink: *mut ffi::bt_self_component_sink,
) -> ffi::bt_component_class_sink_consume_method_status::Type {
    use ffi::bt_component_class_sink_consume_method_status::*;

    let mut sink = SelfComponentSink::from_raw(sink);
    let state = sink.get_c_user_data_ptr() as *mut ProxyPluginState;
    if state.is_null() {
        log::error!("Proxy sink cannot consume, plugin state is NULL");
        return BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_ERROR;
    }

    let state = unsafe { &mut (*state) };
    match state.consume() {
        Ok(retcode) => retcode,
        Err(e) => {
            log::error!("Proxy sink cannot consume. {}", e);
            BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_ERROR
        }
    }
}

/// Plugin descriptor related data, pointers to this data
/// will end up in special linker sections
/// so libbabeltrace2 can discover it
pub mod proxy_plugin_descriptors {
    use super::*;
    use crate::ffi::*;

    pub const SINK_INIT_METHOD_NAME: &[u8] = b"sink_initialize_method";
    pub const SINK_FINI_METHOD_NAME: &[u8] = b"sink_finalize_method";
    pub const SINK_GRAPH_IS_CONF_METHOD_NAME: &[u8] = b"sink_graph_is_configured_method";

    pub static PLUGIN_DESC: __bt_plugin_descriptor = __bt_plugin_descriptor {
        name: ProxyPlugin::PLUGIN_NAME.as_ptr() as *const _,
    };

    pub static SINK_COMP_DESC: __bt_plugin_component_class_descriptor =
        __bt_plugin_component_class_descriptor {
            plugin_descriptor: &PLUGIN_DESC,
            name: ProxyPlugin::OUTPUT_COMP_NAME.as_ptr() as *const _,
            type_: bt_component_class_type::BT_COMPONENT_CLASS_TYPE_SINK,
            methods: __bt_plugin_component_class_descriptor__bindgen_ty_1 {
                sink: __bt_plugin_component_class_descriptor__bindgen_ty_1__bindgen_ty_3 {
                    consume: Some(proxy_sink_consume),
                },
            },
        };

    pub static SINK_COMP_CLASS_INIT_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
    comp_class_descriptor: &SINK_COMP_DESC,
    type_name: SINK_INIT_METHOD_NAME.as_ptr() as *const _,
    type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_INITIALIZE_METHOD,
    value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
        sink_initialize_method: Some(proxy_sink_initialize),
    },
};

    pub static SINK_COMP_CLASS_FINI_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
    comp_class_descriptor: &SINK_COMP_DESC,
    type_name: SINK_FINI_METHOD_NAME.as_ptr() as *const _,
      type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_FINALIZE_METHOD,
      value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
          sink_finalize_method: Some(proxy_sink_finalize),
      },
};

    pub static SINK_COMP_CLASS_GRAPH_CONF_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
    comp_class_descriptor: &SINK_COMP_DESC,
    type_name: SINK_GRAPH_IS_CONF_METHOD_NAME.as_ptr() as *const _,
      type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_GRAPH_IS_CONFIGURED_METHOD,
      value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
          sink_graph_is_configured_method: Some(proxy_sink_graph_is_configured),
      },
};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstrings_are_valid() {
        assert_ne!(ProxyPlugin::plugin_name().to_str().unwrap().len(), 0);
        assert_ne!(ProxyPlugin::output_name().to_str().unwrap().len(), 0);
        assert_ne!(ProxyPlugin::graph_node_name().to_str().unwrap().len(), 0);

        unsafe {
            assert_ne!(
                CStr::from_bytes_with_nul_unchecked(
                    proxy_plugin_descriptors::SINK_INIT_METHOD_NAME
                )
                .to_str()
                .unwrap()
                .len(),
                0
            );
            assert_ne!(
                CStr::from_bytes_with_nul_unchecked(
                    proxy_plugin_descriptors::SINK_FINI_METHOD_NAME
                )
                .to_str()
                .unwrap()
                .len(),
                0
            );
            assert_ne!(
                CStr::from_bytes_with_nul_unchecked(
                    proxy_plugin_descriptors::SINK_GRAPH_IS_CONF_METHOD_NAME
                )
                .to_str()
                .unwrap()
                .len(),
                0
            );
        }
    }
}
