use crate::ffi;

pub type InputPort = Port<ffi::bt_port_input>;
pub type OutputPort = Port<ffi::bt_port_output>;

/// An immutably borrowed port
pub struct Port<T> {
    pub(crate) inner: *const T,
}

pub type SelfComponentInputPort = SelfComponentPort<ffi::bt_self_component_port_input>;
pub type SelfComponentOutputPort = SelfComponentPort<ffi::bt_self_component_port_output>;

pub struct SelfComponentPort<T> {
    pub(crate) inner: *mut T,
}
