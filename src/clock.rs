use crate::{ffi, util, BtResult, Error};
use std::convert::TryInto;
use std::slice;
use uuid::Uuid;

pub type ClockCycles = u64;
pub type ClockNanoseconds = i64;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ClockClassProperties {
    pub frequency: u64,
    pub offset_seconds: i64,
    pub offset_cycles: ClockCycles,
    pub precision: ClockCycles,
    /// Whether or not the origin of the clock class is the Unix epoch
    pub unix_epoch_origin: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    /// When the clock class's origin is not the Unix epoch,
    /// then the clock class's UUID determines whether or not
    /// two different clock classes have correlatable instances
    pub uuid: Option<Uuid>,
}

impl ClockClassProperties {
    pub(crate) fn from_raw(class: *const ffi::bt_clock_class) -> BtResult<Option<Self>> {
        if class.is_null() {
            Ok(None)
        } else {
            let frequency = unsafe { ffi::bt_clock_class_get_frequency(class) };
            let mut offset_seconds = 0;
            let mut offset_cycles = 0;
            unsafe {
                ffi::bt_clock_class_get_offset(class, &mut offset_seconds, &mut offset_cycles)
            };
            let precision = unsafe { ffi::bt_clock_class_get_precision(class) };
            let unix_epoch_origin = unsafe { ffi::bt_clock_class_origin_is_unix_epoch(class) } != 0;
            let name_raw = unsafe { ffi::bt_clock_class_get_name(class) };
            let name = util::opt_owned_cstr(name_raw)?;
            let desc_raw = unsafe { ffi::bt_clock_class_get_description(class) };
            let description = util::opt_owned_cstr(desc_raw)?;
            let uuid_raw = unsafe { ffi::bt_clock_class_get_uuid(class) };
            let uuid = if uuid_raw.is_null() {
                None
            } else {
                let bytes: uuid::Bytes = unsafe { slice::from_raw_parts(uuid_raw, 16) }
                    .try_into()
                    .map_err(|_| Error::Uuid)?;
                Uuid::from_bytes(bytes).into()
            };
            Ok(ClockClassProperties {
                frequency,
                offset_seconds,
                offset_cycles,
                precision,
                unix_epoch_origin,
                name,
                description,
                uuid,
            }
            .into())
        }
    }
}

/// A clock snapshot is a snapshot of the value of a stream clock
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ClockSnapshot {
    pub(crate) inner: *const ffi::bt_clock_snapshot,
}

impl ClockSnapshot {
    /// Returns the value, in clock cycles, of the clock snapshot
    pub fn cycles(&self) -> ClockCycles {
        unsafe { ffi::bt_clock_snapshot_get_value(self.inner) }
    }

    /// Converts the value of the clock snapshot clock_snapshot from
    /// cycles to nanoseconds from the origin of its clock class
    ///
    /// Retunrs None if the computation overflowed
    pub fn ns_from_origin(&self) -> Option<ClockNanoseconds> {
        use ffi::bt_clock_snapshot_get_ns_from_origin_status::*;
        let mut ns_from_origin = 0;
        let status =
            unsafe { ffi::bt_clock_snapshot_get_ns_from_origin(self.inner, &mut ns_from_origin) };
        if status == BT_CLOCK_SNAPSHOT_GET_NS_FROM_ORIGIN_STATUS_OK {
            Some(ns_from_origin)
        } else {
            log::warn!("Clock class conversion ns from origin overflowed");
            None
        }
    }
}
