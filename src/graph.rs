use crate::{
    ffi, BtResult, BtResultExt, ComponentClassFilter, ComponentClassSink, ComponentClassSource,
    ComponentFilter, ComponentSink, ComponentSource, Error, InputPort, LoggingLevel, OutputPort,
    Value,
};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum RunStatus {
    /// Sink component's consuming method returned ok, keep consuming
    Ok,
    /// Sink component's consuming method returned try again
    TryAgain,
    /// All sink components are finished processing
    End,
}

/// Trace processing graph
pub struct Graph {
    inner: *mut ffi::bt_graph,
}

impl Graph {
    pub fn new() -> BtResult<Self> {
        // NOTE: As of Babeltrace 2.0, the only available MIP version is 0
        let mip_version = 0;
        let inner = unsafe { ffi::bt_graph_create(mip_version) };
        if inner.is_null() {
            Err(Error::Memory)
        } else {
            Ok(Graph { inner })
        }
    }

    pub fn add_source_component(
        &mut self,
        class: &ComponentClassSource,
        name: &CStr,
        params: &Value,
        log_level: LoggingLevel,
    ) -> BtResult<ComponentSource> {
        log::debug!("Adding source component to graph");
        let mut comp = ptr::null();
        unsafe {
            ffi::bt_graph_add_source_component(
                self.inner,
                class.inner,
                name.as_ptr(),
                params.inner,
                log_level.into(),
                &mut comp,
            )
        }
        .capi_result()?;
        Ok(ComponentSource { inner: comp })
    }

    pub fn add_filter_component(
        &mut self,
        class: &ComponentClassFilter,
        name: &CStr,
        log_level: LoggingLevel,
    ) -> BtResult<ComponentFilter> {
        log::debug!("Adding filter component to graph");
        let mut comp = ptr::null();
        unsafe {
            ffi::bt_graph_add_filter_component(
                self.inner,
                class.inner,
                name.as_ptr(),
                ptr::null(),
                log_level.into(),
                &mut comp,
            )
        }
        .capi_result()?;
        Ok(ComponentFilter { inner: comp })
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn add_sink_component_with_initialize_method_data(
        &mut self,
        class: &ComponentClassSink,
        name: &CStr,
        initialize_method_data: *mut c_void,
        log_level: LoggingLevel,
    ) -> BtResult<ComponentSink> {
        log::debug!("Adding sink component to graph");
        let mut comp = ptr::null();
        unsafe {
            ffi::bt_graph_add_sink_component_with_initialize_method_data(
                self.inner,
                class.inner,
                name.as_ptr(),
                ptr::null(),
                initialize_method_data,
                log_level.into(),
                &mut comp,
            )
        }
        .capi_result()?;
        Ok(ComponentSink { inner: comp })
    }

    pub fn connect_ports(
        &mut self,
        upstream_port: &OutputPort,
        downstream_port: &InputPort,
    ) -> BtResult<()> {
        unsafe {
            ffi::bt_graph_connect_ports(
                self.inner,
                upstream_port.inner,
                downstream_port.inner,
                ptr::null_mut(),
            )
        }
        .capi_result()
    }

    pub fn run_once(&mut self) -> BtResult<RunStatus> {
        let status = unsafe { ffi::bt_graph_run_once(self.inner) };
        use ffi::bt_graph_run_once_status::*;
        match status {
            BT_GRAPH_RUN_ONCE_STATUS_OK => Ok(RunStatus::Ok),
            BT_GRAPH_RUN_ONCE_STATUS_END => Ok(RunStatus::End),
            BT_GRAPH_RUN_ONCE_STATUS_AGAIN => Ok(RunStatus::TryAgain),
            _ => Err(Error::Failure(status as _)),
        }
    }
}

impl Drop for Graph {
    fn drop(&mut self) {
        unsafe { ffi::bt_graph_put_ref(self.inner) };
    }
}
