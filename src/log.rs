use std::{io::{stderr, Write}, sync::Mutex};

use delay_init::delay;

use crate::ServerError;

#[derive(PartialEq,PartialOrd)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
}

impl TryFrom<u8> for LogLevel {
    type Error = ServerError;

    fn try_from(value: u8) -> crate::Result<Self> {
        Ok(
            match value {
                0 => LogLevel::None,
                1 => LogLevel::Error,
                2 => LogLevel::Warn,
                3 => LogLevel::Info,
                _ => return Err(format!("Invalid log level: {value}").into())
            }
        )
    }

}

pub fn set_level(level: LogLevel) {
    #[allow(clippy::unwrap_used)]
    LOGGER.lock().unwrap().set_level(level);
}

pub trait Logger : Send + Sync {
    fn log(&mut self, level: LogLevel, args: core::fmt::Arguments);
    fn set_level(&mut self, level: LogLevel);
}

struct StdErrLogger(LogLevel);

impl Logger for StdErrLogger {
    fn log(&mut self, level: LogLevel, args: core::fmt::Arguments) {
        if level <= self.0 {
            #[allow(clippy::unwrap_used)]
            stderr().write_fmt(args).unwrap();
        }
    }
    fn set_level(&mut self, level: LogLevel) {
        self.0 = level;
    }
}

delay! {
    static LOGGER : Mutex<Box<dyn Logger>> = Mutex::new(Box::new(StdErrLogger(LogLevel::Error)));
}

#[macro_export]
#[doc(hidden)]
macro_rules! __log {
    ($lev:expr , $($arg:tt)*) => {
        {
            #[allow(clippy::used_underscore_items)]
            $crate::log::_log($lev, format_args!($($arg)*))
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __log_info {
    () => ($crate::log::__log!("\n"));
    ($($arg:tt)*) => ($crate::__log!($crate::log::LogLevel::Info, "INFO: {}\n", format_args!($($arg)*)));
}

#[macro_export]
#[doc(hidden)]
macro_rules! __log_warn {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::__log!($crate::log::LogLevel::Warn, "WARNING: {}\n", format_args!($($arg)*)));
}

#[macro_export]
#[doc(hidden)]
macro_rules! __log_error {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::__log!($crate::log::LogLevel::Error, "ERROR: {}\n", format_args!($($arg)*)));
}

pub mod prelude {
    pub use __log_error as log_error;
    pub use __log_warn as log_warn;
    pub use __log_info as log_info;
}

#[doc(hidden)]
pub fn _log(level: LogLevel, args: core::fmt::Arguments) {
    #[allow(clippy::unwrap_used)]
    LOGGER.lock().unwrap().log(level, args);
}
