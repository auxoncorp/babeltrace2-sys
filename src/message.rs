use crate::{ffi, BtResult, ClockSnapshot, Error, Event, Stream};
use std::os::raw::c_uint;
use std::{ptr, slice};

pub struct MessageArray {
    pub(crate) count: u64,
    pub(crate) inner: ffi::bt_message_array_const,
}

impl MessageArray {
    pub(crate) fn zero() -> Self {
        MessageArray {
            count: 0,
            inner: ptr::null_mut(),
        }
    }

    pub fn as_slice(&self) -> &[*const ffi::bt_message] {
        if self.count == 0 || self.inner.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.inner, self.count as _) }
        }
    }
}

pub struct Message {
    inner: *const ffi::bt_message,
}

impl Message {
    pub fn from_raw(message: *const ffi::bt_message) -> Self {
        Message { inner: message }
    }

    pub fn get_type(&self) -> MessageType {
        use ffi::bt_message_type::*;
        use MessageType::*;
        let typ = unsafe { ffi::bt_message_get_type(self.inner) };
        match typ {
            BT_MESSAGE_TYPE_STREAM_BEGINNING => StreamBeginning,
            BT_MESSAGE_TYPE_STREAM_END => StreamEnd,
            BT_MESSAGE_TYPE_EVENT => Event,
            BT_MESSAGE_TYPE_PACKET_BEGINNING => PacketBeginning,
            BT_MESSAGE_TYPE_PACKET_END => PacketEnd,
            BT_MESSAGE_TYPE_DISCARDED_EVENTS => DiscardedEvents,
            BT_MESSAGE_TYPE_DISCARDED_PACKETS => DiscardedPackets,
            BT_MESSAGE_TYPE_MESSAGE_ITERATOR_INACTIVITY => MessageIteratorInactivity,
            _ => Unknown(typ as _),
        }
    }

    pub fn stream_beginning_borrow_stream(&self) -> BtResult<Stream> {
        debug_assert_eq!(self.get_type(), MessageType::StreamBeginning);
        let stream = unsafe { ffi::bt_message_stream_beginning_borrow_stream_const(self.inner) };
        if stream.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            Ok(Stream { inner: stream })
        }
    }

    pub fn borrow_event(&self) -> BtResult<Event> {
        debug_assert_eq!(self.get_type(), MessageType::Event);
        let event = unsafe { ffi::bt_message_event_borrow_event_const(self.inner) };
        if event.is_null() {
            Err(Error::ResourceBorrow)
        } else {
            let clock =
                unsafe { ffi::bt_message_event_borrow_default_clock_snapshot_const(self.inner) };
            let clock_snapshot = if clock.is_null() {
                None
            } else {
                ClockSnapshot { inner: clock }.into()
            };

            Ok(Event {
                clock_snapshot,
                inner: event,
            })
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { ffi::bt_message_put_ref(self.inner) };
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MessageType {
    StreamBeginning,
    StreamEnd,
    Event,
    PacketBeginning,
    PacketEnd,
    DiscardedEvents,
    DiscardedPackets,
    MessageIteratorInactivity,
    Unknown(c_uint),
}
