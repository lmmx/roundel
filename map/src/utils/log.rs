use std::sync::atomic::{AtomicU8, Ordering};
use wasm_bindgen::JsValue;
use web_sys::console;

// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    None = 4, // Used to disable logging completely
}

// Global log level
static GLOBAL_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

// Get current log level
pub fn get_log_level() -> LogLevel {
    match GLOBAL_LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Debug,
        1 => LogLevel::Info,
        2 => LogLevel::Warn,
        3 => LogLevel::Error,
        _ => LogLevel::None,
    }
}

// Set global log level
pub fn set_log_level(level: LogLevel) {
    GLOBAL_LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

// Log categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    Map,
    Simulation,
    App,
    General,
}

impl LogCategory {
    fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Map => "MAP",
            LogCategory::Simulation => "SIM",
            LogCategory::App => "APP",
            LogCategory::General => "GEN",
        }
    }
}

// Enabled categories (all enabled by default)
static mut ENABLED_CATEGORIES: [bool; 5] = [true, true, true, true, true];

// Enable/disable a specific category
#[allow(dead_code)]
pub fn set_category_enabled(category: LogCategory, enabled: bool) {
    unsafe {
        ENABLED_CATEGORIES[category as usize] = enabled;
    }
}

// Check if a category is enabled
fn is_category_enabled(category: LogCategory) -> bool {
    unsafe { ENABLED_CATEGORIES[category as usize] }
}

// Base log function
fn log_base(level: LogLevel, category: LogCategory, message: &str, source_info: Option<&str>) {
    // Check if logging for this level and category is enabled
    if level < get_log_level() || !is_category_enabled(category) {
        return;
    }

    let prefix = match level {
        LogLevel::Debug => "[DEBUG]",
        LogLevel::Info => "[INFO]",
        LogLevel::Warn => "[WARN]",
        LogLevel::Error => "[ERROR]",
        LogLevel::None => unreachable!(),
    };

    let category_str = category.as_str();

    let log_message = if let Some(source) = source_info {
        format!("{} [{}] [{}] {}", prefix, category_str, source, message)
    } else {
        format!("{} [{}] {}", prefix, category_str, message)
    };

    // Use the appropriate console method based on log level
    match level {
        LogLevel::Debug => console::debug_1(&JsValue::from_str(&log_message)),
        LogLevel::Info => console::log_1(&JsValue::from_str(&log_message)),
        LogLevel::Warn => console::warn_1(&JsValue::from_str(&log_message)),
        LogLevel::Error => console::error_1(&JsValue::from_str(&log_message)),
        LogLevel::None => unreachable!(),
    }
}

// Debug log
#[allow(dead_code)]
pub fn debug(message: &str) {
    log_base(LogLevel::Debug, LogCategory::General, message, None);
}

// Debug log with category
#[allow(dead_code)]
pub fn debug_with_category(category: LogCategory, message: &str) {
    log_base(LogLevel::Debug, category, message, None);
}

// Debug log with source info
#[allow(dead_code)]
pub fn debug_with_source(message: &str, file: &str, line: u32) {
    let source_info = format!("{}:{}", file, line);
    log_base(
        LogLevel::Debug,
        LogCategory::General,
        message,
        Some(&source_info),
    );
}

// Debug log with category and source info
#[allow(dead_code)]
pub fn debug_with_category_and_source(category: LogCategory, message: &str, file: &str, line: u32) {
    let source_info = format!("{}:{}", file, line);
    log_base(LogLevel::Debug, category, message, Some(&source_info));
}

// Conditional debug log that only evaluates if debug level is enabled
#[allow(dead_code)]
pub fn debug_enabled<F>(f: F)
where
    F: FnOnce() -> String,
{
    if get_log_level() <= LogLevel::Debug {
        debug(&f());
    }
}

// Info log
pub fn info(message: &str) {
    log_base(LogLevel::Info, LogCategory::General, message, None);
}

// Info log with category
pub fn info_with_category(category: LogCategory, message: &str) {
    log_base(LogLevel::Info, category, message, None);
}

// Warning log
#[allow(dead_code)]
pub fn warn(message: &str) {
    log_base(LogLevel::Warn, LogCategory::General, message, None);
}

// Warning log with category
#[allow(dead_code)]
pub fn warn_with_category(category: LogCategory, message: &str) {
    log_base(LogLevel::Warn, category, message, None);
}

// Error log
#[allow(dead_code)]
pub fn error(message: &str) {
    log_base(LogLevel::Error, LogCategory::General, message, None);
}

// Error log with category
#[allow(dead_code)]
pub fn error_with_category(category: LogCategory, message: &str) {
    log_base(LogLevel::Error, category, message, None);
}

// Format and log
#[allow(dead_code)]
pub fn format_and_log(level: LogLevel, category: LogCategory, fmt: &str, args: &[&str]) {
    if level < get_log_level() || !is_category_enabled(category) {
        return;
    }

    let message = if args.is_empty() {
        fmt.to_string()
    } else {
        // Simple formatting implementation
        let mut result = fmt.to_string();
        for arg in args {
            if let Some(pos) = result.find("{}") {
                result.replace_range(pos..pos + 2, arg);
            }
        }
        result
    };

    log_base(level, category, &message, None);
}

// Context logger for structured logging
pub struct ContextLogger {
    context: String,
    category: LogCategory,
}

impl ContextLogger {
    pub fn new(context: &str, category: LogCategory) -> Self {
        Self {
            context: context.to_string(),
            category,
        }
    }

    pub fn debug(&self, message: &str) {
        log_base(LogLevel::Debug, self.category, message, Some(&self.context));
    }

    pub fn info(&self, message: &str) {
        log_base(LogLevel::Info, self.category, message, Some(&self.context));
    }

    #[allow(dead_code)]
    pub fn warn(&self, message: &str) {
        log_base(LogLevel::Warn, self.category, message, Some(&self.context));
    }

    pub fn error(&self, message: &str) {
        log_base(LogLevel::Error, self.category, message, Some(&self.context));
    }
}

// Context logging
pub fn with_context<F, R>(context: &str, category: LogCategory, f: F) -> R
where
    F: FnOnce(&ContextLogger) -> R,
{
    let logger = ContextLogger::new(context, category);
    f(&logger)
}

// Convenience macros
#[macro_export]
macro_rules! debug {
    ($message:expr) => {
        $crate::utils::log::debug($message)
    };
    ($fmt:expr, $($arg:expr),*) => {
        {
            let args = &[$($arg),*];
            $crate::utils::log::format_and_log(
                $crate::utils::log::LogLevel::Debug,
                $crate::utils::log::LogCategory::General,
                $fmt,
                args
            )
        }
    };
}

#[macro_export]
macro_rules! info {
    ($message:expr) => {
        $crate::utils::log::info($message)
    };
    ($fmt:expr, $($arg:expr),*) => {
        {
            let args = &[$($arg),*];
            $crate::utils::log::format_and_log(
                $crate::utils::log::LogLevel::Info,
                $crate::utils::log::LogCategory::General,
                $fmt,
                args
            )
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($message:expr) => {
        $crate::utils::log::warn($message)
    };
    ($fmt:expr, $($arg:expr),*) => {
        {
            let args = &[$($arg),*];
            $crate::utils::log::format_and_log(
                $crate::utils::log::LogLevel::Warn,
                $crate::utils::log::LogCategory::General,
                $fmt,
                args
            )
        }
    };
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        $crate::utils::log::error($message)
    };
    ($fmt:expr, $($arg:expr),*) => {
        {
            let args = &[$($arg),*];
            $crate::utils::log::format_and_log(
                $crate::utils::log::LogLevel::Error,
                $crate::utils::log::LogCategory::General,
                $fmt,
                args
            )
        }
    };
}

// Source location macros
#[macro_export]
macro_rules! debug_here {
    ($message:expr) => {
        $crate::utils::log::debug_with_source($message, file!(), line!())
    };
}

// With category macros
#[macro_export]
macro_rules! debug_map {
    ($message:expr) => {
        $crate::utils::log::debug_with_category($crate::utils::log::LogCategory::Map, $message)
    };
}

#[macro_export]
macro_rules! info_map {
    ($message:expr) => {
        $crate::utils::log::info_with_category($crate::utils::log::LogCategory::Map, $message)
    };
}

#[macro_export]
macro_rules! warn_map {
    ($message:expr) => {
        $crate::utils::log::warn_with_category($crate::utils::log::LogCategory::Map, $message)
    };
}

#[macro_export]
macro_rules! error_map {
    ($message:expr) => {
        $crate::utils::log::error_with_category($crate::utils::log::LogCategory::Map, $message)
    };
}

// Simulation category macros
#[macro_export]
macro_rules! debug_sim {
    ($message:expr) => {
        $crate::utils::log::debug_with_category(
            $crate::utils::log::LogCategory::Simulation,
            $message,
        )
    };
}

#[macro_export]
macro_rules! info_sim {
    ($message:expr) => {
        $crate::utils::log::info_with_category(
            $crate::utils::log::LogCategory::Simulation,
            $message,
        )
    };
}
