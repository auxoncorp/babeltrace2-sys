use crate::{
    ffi, util, BtResult, ClockNanoseconds, ClockSnapshot, Error, Field, OwnedField, StreamId,
};
use std::fmt;

pub struct Event {
    pub(crate) clock_snapshot: Option<ClockSnapshot>,
    pub(crate) inner: *const ffi::bt_event,
}

impl Event {
    pub fn to_owned(self) -> BtResult<OwnedEvent> {
        let stream_id = self.stream_id();
        let clock_snapshot = self.clock_snapshot();
        let class_properties = self.class_properties()?;
        let properties = self.properties()?;
        Ok(OwnedEvent {
            stream_id,
            clock_snapshot,
            class_properties,
            properties,
        })
    }

    pub fn stream_id(&self) -> StreamId {
        unsafe {
            let stream = ffi::bt_event_borrow_stream_const(self.inner);
            ffi::bt_stream_get_id(stream)
        }
    }

    pub fn clock_snapshot(&self) -> Option<ClockNanoseconds> {
        self.clock_snapshot.map(|c| c.ns_from_origin()).flatten()
    }

    pub fn class_properties(&self) -> BtResult<EventClassProperties> {
        let class = unsafe { ffi::bt_event_borrow_class_const(self.inner) };
        if class.is_null() {
            return Err(Error::ResourceBorrow);
        }
        let id = unsafe { ffi::bt_event_class_get_id(class) };
        let name_cstr = unsafe { ffi::bt_event_class_get_name(class) };
        let name = util::opt_owned_cstr(name_cstr)?;
        let mut log_level_raw = 0;
        let log_level_avail =
            unsafe { ffi::bt_event_class_get_log_level(class, &mut log_level_raw) };
        let log_level = if log_level_avail
            == ffi::bt_property_availability::BT_PROPERTY_AVAILABILITY_AVAILABLE
        {
            EventLogLevel::from_raw(log_level_raw)
        } else {
            None
        };
        Ok(EventClassProperties {
            id,
            name,
            log_level,
        })
    }

    pub fn properties(&self) -> BtResult<EventProperties> {
        let payload = self.payload()?;
        let specific_context = self.specific_context()?;
        let common_context = self.common_context()?;
        Ok(EventProperties {
            payload,
            specific_context,
            common_context,
        })
    }

    pub fn payload(&self) -> BtResult<Option<OwnedField>> {
        let field = unsafe { ffi::bt_event_borrow_payload_field_const(self.inner) };
        Ok(Field::from_raw(field)
            .map(|f| f.to_owned())
            .transpose()?
            .flatten())
    }

    pub fn specific_context(&self) -> BtResult<Option<OwnedField>> {
        let field = unsafe { ffi::bt_event_borrow_specific_context_field_const(self.inner) };
        Ok(Field::from_raw(field)
            .map(|f| f.to_owned())
            .transpose()?
            .flatten())
    }

    pub fn common_context(&self) -> BtResult<Option<OwnedField>> {
        let field = unsafe { ffi::bt_event_borrow_common_context_field_const(self.inner) };
        Ok(Field::from_raw(field)
            .map(|f| f.to_owned())
            .transpose()?
            .flatten())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct OwnedEvent {
    pub stream_id: StreamId,
    pub clock_snapshot: Option<ClockNanoseconds>,
    pub class_properties: EventClassProperties,
    pub properties: EventProperties,
}

impl fmt::Display for OwnedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ns = self
            .clock_snapshot
            .map(|v| v.to_string())
            .unwrap_or_else(|| String::from("??"));
        let event_name = if let Some(n) = &self.class_properties.name {
            format!("{} (ID={})", n, self.class_properties.id)
        } else {
            format!("ID={}", self.class_properties.id)
        };
        write!(f, "[{}] {}", ns, event_name)?;
        write!(f, "\n  stream ID: {}", self.stream_id)?;
        if let Some(t) = self.class_properties.log_level {
            write!(f, "\n  log_level: {:?}", t)?;
        }
        if let Some(t) = &self.properties.payload {
            write!(f, "\n  payload: {{ {} }}", t)?;
        }
        if let Some(t) = &self.properties.specific_context {
            write!(f, "\n  specific context: {{ {} }}", t)?;
        }
        if let Some(t) = &self.properties.common_context {
            write!(f, "\n  common context: {{ {} }}", t)?;
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EventClassProperties {
    pub id: EventId,
    pub name: Option<String>,
    pub log_level: Option<EventLogLevel>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EventProperties {
    pub payload: Option<OwnedField>,
    pub specific_context: Option<OwnedField>,
    pub common_context: Option<OwnedField>,
}

pub type EventId = u64;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EventLogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    DebugSystem,
    DebugProgram,
    DebugProcess,
    DebugModule,
    DebugUnit,
    DebugFunction,
    DebugLine,
    Debug,
}

impl EventLogLevel {
    fn from_raw(value: ffi::bt_event_class_log_level::Type) -> Option<Self> {
        use ffi::bt_event_class_log_level::*;
        use EventLogLevel::*;
        match value {
            BT_EVENT_CLASS_LOG_LEVEL_EMERGENCY => Emergency.into(),
            BT_EVENT_CLASS_LOG_LEVEL_ALERT => Alert.into(),
            BT_EVENT_CLASS_LOG_LEVEL_CRITICAL => Critical.into(),
            BT_EVENT_CLASS_LOG_LEVEL_ERROR => Error.into(),
            BT_EVENT_CLASS_LOG_LEVEL_WARNING => Warning.into(),
            BT_EVENT_CLASS_LOG_LEVEL_NOTICE => Notice.into(),
            BT_EVENT_CLASS_LOG_LEVEL_INFO => Info.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_SYSTEM => DebugSystem.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_PROGRAM => DebugProgram.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_PROCESS => DebugProcess.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_MODULE => DebugModule.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_UNIT => DebugUnit.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_FUNCTION => DebugFunction.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG_LINE => DebugLine.into(),
            BT_EVENT_CLASS_LOG_LEVEL_DEBUG => Debug.into(),
            _ => {
                log::trace!("Unsupported event log level {}", value);
                None
            }
        }
    }
}
