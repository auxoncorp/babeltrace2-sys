use crate::{ffi, util, BtResult, ClockClassProperties, Error, Trace};

pub struct Stream {
    pub(crate) inner: *const ffi::bt_stream,
}

impl Stream {
    pub fn properties(&self) -> BtResult<StreamProperties> {
        let id = unsafe { ffi::bt_stream_get_id(self.inner) };
        let name_cstr = unsafe { ffi::bt_stream_get_name(self.inner) };
        let name = util::opt_owned_cstr(name_cstr)?;
        let class = unsafe { ffi::bt_stream_borrow_class_const(self.inner) };
        let clock_class = unsafe { ffi::bt_stream_class_borrow_default_clock_class_const(class) };
        let clock = ClockClassProperties::from_raw(clock_class)?;
        Ok(StreamProperties { id, name, clock })
    }

    pub fn trace(&self) -> BtResult<Trace> {
        let trace = unsafe { ffi::bt_stream_borrow_trace_const(self.inner) };
        if trace.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(Trace { inner: trace })
        }
    }
}

pub type StreamId = u64;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct StreamProperties {
    pub id: StreamId,
    pub name: Option<String>,
    pub clock: Option<ClockClassProperties>,
}
