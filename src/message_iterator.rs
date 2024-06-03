use crate::{ffi, BtResult, Error, MessageArray};
use std::os::raw::c_void;
use std::ptr;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum NextStatus {
    Ok,
    End,
    TryAgain,
}

pub struct MessageIterator {
    pub(crate) inner: *mut ffi::bt_message_iterator,
}

impl MessageIterator {
    pub fn next_message_array(&mut self) -> BtResult<(NextStatus, MessageArray)> {
        use ffi::bt_message_iterator_next_status::*;
        let mut messages = ptr::null_mut();
        let mut count = 0;
        let status =
            unsafe { ffi::bt_message_iterator_next(self.inner, &mut messages, &mut count) };
        match status {
            BT_MESSAGE_ITERATOR_NEXT_STATUS_OK => Ok((
                NextStatus::Ok,
                MessageArray {
                    count,
                    inner: messages,
                },
            )),
            BT_MESSAGE_ITERATOR_NEXT_STATUS_END => Ok((NextStatus::End, MessageArray::zero())),
            BT_MESSAGE_ITERATOR_NEXT_STATUS_AGAIN => {
                Ok((NextStatus::TryAgain, MessageArray::zero()))
            }
            _ => Err(Error::Failure(status as _)),
        }
    }
}

impl Drop for MessageIterator {
    fn drop(&mut self) {
        unsafe { ffi::bt_message_iterator_put_ref(self.inner) };
    }
}

pub struct SelfMessageIterator {
    pub(crate) inner: *mut ffi::bt_self_message_iterator,
}

impl SelfMessageIterator {
    pub(crate) fn from_raw(inner: *mut ffi::bt_self_message_iterator) -> Self {
        debug_assert!(!inner.is_null());
        Self { inner }
    }

    // TODO remove once high-level types/API exists
    pub fn inner_mut(&mut self) -> *mut ffi::bt_self_message_iterator {
        self.inner
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn set_c_user_data_ptr(&mut self, user_data: *mut c_void) {
        unsafe { ffi::bt_self_message_iterator_set_data(self.inner, user_data) };
    }

    pub fn get_c_user_data_ptr(&mut self) -> *mut c_void {
        unsafe { ffi::bt_self_message_iterator_get_data(self.inner as _) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MessageIteratorStatus {
    NoMessages,
    Messages(u64), // count
    Done,
}
