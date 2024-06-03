use crate::{
    ffi, ComponentSource, MessageIteratorStatus, SelfComponent, SelfComponentSource,
    SelfMessageIterator, SourcePluginHandler,
};
use std::ffi::c_void;
use std::slice;
use tracing::{debug, error, trace};

pub type SourcePluginState = Box<dyn SourcePluginHandler>;

/// # Safety
///
/// Only to be used by the [source_plugin_descriptors] macro.
#[no_mangle]
pub unsafe extern "C" fn source_plugin_initialize(
    source: *mut ffi::bt_self_component_source,
    _config: *mut ffi::bt_self_component_source_configuration,
    _params: *const ffi::bt_value,
    initialize_method_data: *mut c_void,
) -> ffi::bt_component_class_initialize_method_status::Type {
    use ffi::bt_component_class_initialize_method_status::*;

    debug!("Initializing source plugin");

    if initialize_method_data.is_null() {
        error!("Plugin state is NULL");
        return BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_ERROR;
    }

    // Set the component's user data to our private
    let mut source = SelfComponentSource::from_raw(source);
    source.set_c_user_data_ptr(initialize_method_data);

    let state_raw = initialize_method_data as *mut SourcePluginState;
    let state = &mut (*state_raw);

    if let Err(e) = state.initialize(source.upcast()) {
        error!("Source plugin initialize returned an error. {e}");
        return BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_ERROR;
    }

    // Add an output port named `out` to the source component.
    // This is needed so that this source component can be connected to
    // a filter or a sink component. Once a downstream component is
    // connected, it can create our message iterator.
    if source
        .add_output_port(ComponentSource::out_port_name())
        .is_err()
    {
        error!("Failed to add source plugin output port");
        BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_ERROR
    } else {
        BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_OK
    }
}

/// # Safety
///
/// Only to be used by the [source_plugin_descriptors] macro.
#[no_mangle]
pub unsafe extern "C" fn source_plugin_finalize(source: *mut ffi::bt_self_component_source) {
    debug!("Finalizing source plugin");

    let mut source = SelfComponentSource::from_raw(source);
    let state_raw = source.get_c_user_data_ptr() as *mut SourcePluginState;
    if !state_raw.is_null() {
        let mut state = Box::from_raw(state_raw);
        if let Err(e) = state.finalize(source.upcast()) {
            error!("Source plugin finalize returned an error. {e}");
        }
    }
}

/// # Safety
///
/// Only to be used by the [source_plugin_descriptors] macro.
#[no_mangle]
pub unsafe extern "C" fn source_plugin_message_iterator_initialize(
    msg_iter: *mut ffi::bt_self_message_iterator,
    _config: *mut ffi::bt_self_message_iterator_configuration,
    _port: *mut ffi::bt_self_component_port_output,
) -> ffi::bt_message_iterator_class_initialize_method_status::Type {
    debug!("Initializing source plugin message iterator");

    use ffi::bt_message_iterator_class_initialize_method_status::*;

    let mut self_comp = SelfComponent::<ffi::bt_self_component>::from_raw(
        ffi::bt_self_message_iterator_borrow_component(msg_iter),
    );

    let mut msg_iter = SelfMessageIterator::from_raw(msg_iter);
    msg_iter.set_c_user_data_ptr(self_comp.get_c_user_data_ptr());

    BT_MESSAGE_ITERATOR_CLASS_INITIALIZE_METHOD_STATUS_OK
}

/// # Safety
///
/// Only to be used by the [source_plugin_descriptors] macro.
#[no_mangle]
pub unsafe extern "C" fn source_plugin_message_iterator_finalize(
    _msg_iter: *mut ffi::bt_self_message_iterator,
) {
    debug!("Finalizing source plugin message iterator");
}

/// # Safety
///
/// Only to be used by the [source_plugin_descriptors] macro.
#[no_mangle]
pub unsafe extern "C" fn source_plugin_message_iterator_next(
    msg_iter: *mut ffi::bt_self_message_iterator,
    messages: ffi::bt_message_array_const,
    capacity: u64,
    count: *mut u64,
) -> ffi::bt_message_iterator_class_next_method_status::Type {
    use ffi::bt_message_iterator_class_next_method_status::*;

    trace!("Source plugin message iterator next");

    let mut msg_iter = SelfMessageIterator::from_raw(msg_iter);

    let state_raw = msg_iter.get_c_user_data_ptr() as *mut SourcePluginState;
    let state = &mut (*state_raw);

    // TODO - need a high-level wrapper type for this
    let messages = slice::from_raw_parts_mut(messages, capacity as _);

    match state.iterator_next(msg_iter, messages) {
        Ok(status) => match status {
            MessageIteratorStatus::NoMessages => BT_MESSAGE_ITERATOR_CLASS_NEXT_METHOD_STATUS_AGAIN,
            MessageIteratorStatus::Messages(msg_cnt) => {
                *count = msg_cnt;
                BT_MESSAGE_ITERATOR_CLASS_NEXT_METHOD_STATUS_OK
            }
            MessageIteratorStatus::Done => BT_MESSAGE_ITERATOR_CLASS_NEXT_METHOD_STATUS_END,
        },
        Err(e) => {
            error!("Source plugin iterator_next returned an error. {e}");
            BT_MESSAGE_ITERATOR_CLASS_NEXT_METHOD_STATUS_ERROR
        }
    }
}

#[macro_export]
macro_rules! source_plugin_descriptors {
    ($plugin:ident) => {
        pub mod source_plugin_descriptors {
            use super::$plugin;
            use babeltrace2_sys::ffi::*;
            use babeltrace2_sys::SourcePluginDescriptor;

            pub const SOURCE_INIT_METHOD_NAME: &[u8] = b"source_initialize_method\0";
            pub const SOURCE_FINI_METHOD_NAME: &[u8] = b"source_finalize_method\0";
            pub const MSG_ITER_INIT_METHOD_NAME: &[u8] = b"msg_iter_initialize_method\0";
            pub const MSG_ITER_FINI_METHOD_NAME: &[u8] = b"msg_iter_finalize_method\0";

            pub static PLUGIN_DESC: __bt_plugin_descriptor = __bt_plugin_descriptor {
                name: $plugin::PLUGIN_NAME.as_ptr() as *const _,
            };

            pub static SOURCE_COMP_DESC: __bt_plugin_component_class_descriptor =
            __bt_plugin_component_class_descriptor {
                plugin_descriptor: &PLUGIN_DESC,
                name: $plugin::OUTPUT_COMP_NAME.as_ptr() as *const _,
                type_: bt_component_class_type::BT_COMPONENT_CLASS_TYPE_SOURCE,
                methods: __bt_plugin_component_class_descriptor__bindgen_ty_1 {
                    source: __bt_plugin_component_class_descriptor__bindgen_ty_1__bindgen_ty_1 {
                        msg_iter_next: Some($crate::source_plugin::source_plugin_message_iterator_next),
                    },
                },
            };

            pub static SOURCE_COMP_CLASS_INIT_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
                comp_class_descriptor: &SOURCE_COMP_DESC,
                type_name: SOURCE_INIT_METHOD_NAME.as_ptr() as *const _,
                type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_INITIALIZE_METHOD,
                value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
                    source_initialize_method: Some($crate::source_plugin::source_plugin_initialize),
                },
            };

            pub static SOURCE_COMP_CLASS_FINI_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
                comp_class_descriptor: &SOURCE_COMP_DESC,
                type_name: SOURCE_FINI_METHOD_NAME.as_ptr() as *const _,
                type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_FINALIZE_METHOD,
                value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
                  source_finalize_method: Some($crate::source_plugin::source_plugin_finalize),
                },
            };

            pub static SOURCE_COMP_CLASS_MSG_ITER_INIT_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
                comp_class_descriptor: &SOURCE_COMP_DESC,
                type_name: MSG_ITER_INIT_METHOD_NAME.as_ptr() as *const _,
                type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_MSG_ITER_INITIALIZE_METHOD,
                value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
                  msg_iter_initialize_method: Some($crate::source_plugin::source_plugin_message_iterator_initialize),
                },
            };

            pub static SOURCE_COMP_CLASS_MSG_ITER_FINI_ATTR: __bt_plugin_component_class_descriptor_attribute = __bt_plugin_component_class_descriptor_attribute {
                comp_class_descriptor: &SOURCE_COMP_DESC,
                type_name: MSG_ITER_FINI_METHOD_NAME.as_ptr() as *const _,
                type_: __bt_plugin_component_class_descriptor_attribute_type::BT_PLUGIN_COMPONENT_CLASS_DESCRIPTOR_ATTRIBUTE_TYPE_MSG_ITER_FINALIZE_METHOD,
                value: __bt_plugin_component_class_descriptor_attribute__bindgen_ty_1 {
                  msg_iter_finalize_method: Some($crate::source_plugin::source_plugin_message_iterator_finalize),
                },
            };

        #[used]
        #[link_section = "__bt_plugin_descriptors"]
        pub static PLUGIN_DESC_PTR: __bt_plugin_descriptor_ptr =
            __bt_plugin_descriptor_ptr(&PLUGIN_DESC);

        #[used]
        #[link_section = "__bt_plugin_component_class_descriptors"]
        pub static SOURCE_COMP_DESC_PTR: __bt_plugin_component_class_descriptor_ptr =
            __bt_plugin_component_class_descriptor_ptr(&SOURCE_COMP_DESC);

        #[used]
        #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
        pub static SOURCE_COMP_CLASS_INIT_ATTR_PTR:
            __bt_plugin_component_class_descriptor_attribute_ptr =
            __bt_plugin_component_class_descriptor_attribute_ptr(&SOURCE_COMP_CLASS_INIT_ATTR);

        #[used]
        #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
        pub static SOURCE_COMP_CLASS_FINI_ATTR_PTR:
            __bt_plugin_component_class_descriptor_attribute_ptr =
            __bt_plugin_component_class_descriptor_attribute_ptr(&SOURCE_COMP_CLASS_FINI_ATTR);

        #[used]
        #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
        pub static SOURCE_COMP_CLASS_MSG_ITER_INIT_PTR:
            __bt_plugin_component_class_descriptor_attribute_ptr =
            __bt_plugin_component_class_descriptor_attribute_ptr(&SOURCE_COMP_CLASS_MSG_ITER_INIT_ATTR);

        #[used]
        #[link_section = "__bt_plugin_component_class_descriptor_attributes"]
        pub static SOURCE_COMP_CLASS_MSG_ITER_FINI_PTR:
            __bt_plugin_component_class_descriptor_attribute_ptr =
            __bt_plugin_component_class_descriptor_attribute_ptr(&SOURCE_COMP_CLASS_MSG_ITER_FINI_ATTR);
            }
        };
}
