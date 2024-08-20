use std::{io::{stderr, Write}, sync::Mutex};

use delay_init::delay;

#[derive(PartialEq,PartialOrd)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
}

impl From<u8> for LogLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => LogLevel::None,
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            _ => panic!("Invalid log level: {value}")
        }
    }
}

pub fn set_level(level: LogLevel) {
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
            stderr().write_fmt(args).unwrap()
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
macro_rules! log {
    ($lev:expr , $($arg:tt)*) => {
        $crate::log::_log($lev, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Info, "INFO: {}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_warn {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Warn, "WARNING: {}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_error {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Error, "ERROR: {}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _log(level: LogLevel, args: core::fmt::Arguments) {
    LOGGER.lock().unwrap().log(level, args)
}
