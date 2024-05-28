use crate::ffi;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum LoggingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    #[default]
    None,
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

impl fmt::Display for LoggingLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoggingLevel::*;
        let s = match self {
            Trace => "trace",
            Debug => "debug",
            Info => "info",
            Warn => "warn",
            Error => "error",
            Fatal => "fatal",
            None => "none",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for LoggingLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LoggingLevel::*;
        Ok(match s.to_lowercase().as_str() {
            "trace" => Trace,
            "debug" => Debug,
            "info" => Info,
            "warn" | "warning" => Warn,
            "error" | "err" => Error,
            "fatal" => Fatal,
            "none" => None,
            _ => return Err(format!("'{}' is not a valid logging level", s)),
        })
    }
}
