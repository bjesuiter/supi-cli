use std::io::{self, Write};
use std::sync::Mutex;

/// Global output synchronizer to prevent jumbled terminal output
static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

/// Print a line to stdout with proper synchronization and raw mode support
pub fn print_line(msg: &str) {
    let _guard = OUTPUT_LOCK.lock().unwrap();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // In raw mode, we need \r\n instead of just \n
    // Writing \r\n works fine in both raw and normal mode
    let _ = writeln!(handle, "{}\r", msg);
    let _ = handle.flush();
}

/// Print a line to stderr with proper synchronization and raw mode support
pub fn eprint_line(msg: &str) {
    let _guard = OUTPUT_LOCK.lock().unwrap();
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    // In raw mode, we need \r\n instead of just \n
    // Writing \r\n works fine in both raw and normal mode
    let _ = writeln!(handle, "{}\r", msg);
    let _ = handle.flush();
}

/// Macro to replace println! with synchronized output
#[macro_export]
macro_rules! sprintln {
    ($($arg:tt)*) => {
        $crate::output::print_line(&format!($($arg)*))
    };
}

/// Macro to replace eprintln! with synchronized output
#[macro_export]
macro_rules! seprintln {
    ($($arg:tt)*) => {
        $crate::output::eprint_line(&format!($($arg)*))
    };
}
