use crate::ffi;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LoggingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    None,
}

impl Default for LoggingLevel {
    fn default() -> Self {
        LoggingLevel::None
    }
}

pub struct Logger;

impl Logger {
    pub fn set_level(level: LoggingLevel) {
        unsafe { ffi::bt_logging_set_global_level(level.into()) };
    }
}

impl From<LoggingLevel> for ffi::bt_logging_level::Type {
    fn from(level: LoggingLevel) -> Self {
        use LoggingLevel::*;
        match level {
            Trace => ffi::bt_logging_level::BT_LOGGING_LEVEL_TRACE,
            Debug => ffi::bt_logging_level::BT_LOGGING_LEVEL_DEBUG,
            Info => ffi::bt_logging_level::BT_LOGGING_LEVEL_INFO,
            Warn => ffi::bt_logging_level::BT_LOGGING_LEVEL_WARNING,
            Error => ffi::bt_logging_level::BT_LOGGING_LEVEL_ERROR,
            Fatal => ffi::bt_logging_level::BT_LOGGING_LEVEL_FATAL,
            None => ffi::bt_logging_level::BT_LOGGING_LEVEL_NONE,
        }
    }
}
