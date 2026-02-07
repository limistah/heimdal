/// Logging macros that accept format! style arguments
/// These reduce boilerplate by eliminating the need for `&format!(...)`
#[macro_export]
macro_rules! info_fmt {
    ($($arg:tt)*) => {
        $crate::utils::info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! success_fmt {
    ($($arg:tt)*) => {
        $crate::utils::success(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! error_fmt {
    ($($arg:tt)*) => {
        $crate::utils::error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warning_fmt {
    ($($arg:tt)*) => {
        $crate::utils::warning(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! step_fmt {
    ($($arg:tt)*) => {
        $crate::utils::step(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! header_fmt {
    ($($arg:tt)*) => {
        $crate::utils::header(&format!($($arg)*))
    };
}
