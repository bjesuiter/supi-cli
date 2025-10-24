use crossterm::style::{Color, ResetColor, SetForegroundColor};
use std::io::{self, Write};
use std::sync::Mutex;

/// Global output synchronizer to prevent jumbled terminal output
static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

/// Log color configuration for supervisor messages
#[derive(Debug, Clone, Copy)]
pub enum LogColor {
    Yellow,
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    White,
    None,
}

impl LogColor {
    /// Parse a color string from CLI argument
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "yellow" => Ok(LogColor::Yellow),
            "red" => Ok(LogColor::Red),
            "green" => Ok(LogColor::Green),
            "blue" => Ok(LogColor::Blue),
            "cyan" => Ok(LogColor::Cyan),
            "magenta" => Ok(LogColor::Magenta),
            "white" => Ok(LogColor::White),
            "none" => Ok(LogColor::None),
            _ => Err(format!(
                "Invalid color '{}'. Supported: yellow, red, green, blue, cyan, magenta, white, none",
                s
            )),
        }
    }

    /// Convert to crossterm Color
    fn to_crossterm_color(self) -> Option<Color> {
        match self {
            LogColor::Yellow => Some(Color::Yellow),
            LogColor::Red => Some(Color::Red),
            LogColor::Green => Some(Color::Green),
            LogColor::Blue => Some(Color::Blue),
            LogColor::Cyan => Some(Color::Cyan),
            LogColor::Magenta => Some(Color::Magenta),
            LogColor::White => Some(Color::White),
            LogColor::None => None,
        }
    }
}

/// Stateful output manager that handles supervisor and child process output
/// with configurable colors and silent mode
#[derive(Debug, Clone)]
pub struct Output {
    log_color: LogColor,
    info_color: LogColor,
    silent: bool,
}

impl Output {
    /// Create a new Output instance
    pub fn new(log_color: LogColor, info_color: LogColor, silent: bool) -> Self {
        Self {
            log_color,
            info_color,
            silent,
        }
    }

    /// Print a supervisor log message (colored with log_color)
    /// Suppressed when silent mode is enabled
    pub fn log(&self, msg: &str) {
        if self.silent {
            return;
        }
        print_line_colored(msg, self.log_color);
    }

    /// Print a supervisor log message to stderr (colored with log_color)
    /// Suppressed when silent mode is enabled
    pub fn elog(&self, msg: &str) {
        if self.silent {
            return;
        }
        eprint_line_colored(msg, self.log_color);
    }

    /// Print an informational message (colored with info_color)
    /// Suppressed when silent mode is enabled
    pub fn info(&self, msg: &str) {
        if self.silent {
            return;
        }
        print_line_colored(msg, self.info_color);
    }

    /// Print an informational message to stderr (colored with info_color)
    /// Suppressed when silent mode is enabled
    pub fn einfo(&self, msg: &str) {
        if self.silent {
            return;
        }
        eprint_line_colored(msg, self.info_color);
    }

    /// Forward child process stdout (never suppressed, never colored)
    pub fn forward_stdout(&self, line: &str) {
        print_line(line);
    }

    /// Forward child process stderr (never suppressed, never colored)
    pub fn forward_stderr(&self, line: &str) {
        eprint_line(line);
    }
}

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

/// Print a line to stdout with color support
pub fn print_line_colored(msg: &str, color: LogColor) {
    let _guard = OUTPUT_LOCK.lock().unwrap();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if let Some(c) = color.to_crossterm_color() {
        let _ = crossterm::execute!(handle, SetForegroundColor(c));
    }

    let _ = writeln!(handle, "{}\r", msg);

    if color.to_crossterm_color().is_some() {
        let _ = crossterm::execute!(handle, ResetColor);
    }

    let _ = handle.flush();
}

/// Print a line to stderr with color support
pub fn eprint_line_colored(msg: &str, color: LogColor) {
    let _guard = OUTPUT_LOCK.lock().unwrap();
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    if let Some(c) = color.to_crossterm_color() {
        let _ = crossterm::execute!(handle, SetForegroundColor(c));
    }

    let _ = writeln!(handle, "{}\r", msg);

    if color.to_crossterm_color().is_some() {
        let _ = crossterm::execute!(handle, ResetColor);
    }

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

/// Macro for colored println! with synchronized output
#[macro_export]
macro_rules! sprintln_colored {
    ($color:expr, $($arg:tt)*) => {
        $crate::output::print_line_colored(&format!($($arg)*), $color)
    };
}

/// Macro for colored eprintln! with synchronized output
#[macro_export]
macro_rules! seprintln_colored {
    ($color:expr, $($arg:tt)*) => {
        $crate::output::eprint_line_colored(&format!($($arg)*), $color)
    };
}
