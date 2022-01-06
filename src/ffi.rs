pub use crate::bindings::*;

unsafe impl Sync for __bt_plugin_descriptor {}
unsafe impl Sync for __bt_plugin_component_class_descriptor {}
unsafe impl Sync for __bt_plugin_component_class_descriptor__bindgen_ty_1 {}
unsafe impl Sync for __bt_plugin_component_class_descriptor_attribute {}

#[repr(transparent)]
pub struct __bt_plugin_descriptor_ptr(pub *const __bt_plugin_descriptor);
unsafe impl Sync for __bt_plugin_descriptor_ptr {}

#[repr(transparent)]
pub struct __bt_plugin_component_class_descriptor_ptr(
    pub *const __bt_plugin_component_class_descriptor,
);
unsafe impl Sync for __bt_plugin_component_class_descriptor_ptr {}

#[repr(transparent)]
pub struct __bt_plugin_component_class_descriptor_attribute_ptr(
    pub *const __bt_plugin_component_class_descriptor_attribute,
);
unsafe impl Sync for __bt_plugin_component_class_descriptor_attribute_ptr {}
