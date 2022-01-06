use crate::ffi;

pub type ComponentClassSource = ComponentClass<ffi::bt_component_class_source>;
pub type ComponentClassFilter = ComponentClass<ffi::bt_component_class_filter>;
pub type ComponentClassSink = ComponentClass<ffi::bt_component_class_sink>;

/// An immutably borrowed component class
pub struct ComponentClass<T> {
    pub(crate) inner: *const T,
}
