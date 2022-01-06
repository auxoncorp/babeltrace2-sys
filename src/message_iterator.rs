use crate::{ffi, BtResult, Error, MessageArray};
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
