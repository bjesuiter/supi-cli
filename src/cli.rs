use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "supi")]
#[command(version)]
#[command(about = "A lightweight process supervisor with restart capabilities", long_about = None)]
pub struct Cli {
    /// Stop the supervisor when the child process exits
    #[arg(long)]
    pub stop_on_child_exit: bool,

    /// Signal to use for restarting the child process (default: SIGUSR1)
    #[arg(long, default_value = "SIGUSR1")]
    pub restart_signal: String,

    /// Hotkey character for manual restart (default: 'r')
    #[arg(long, default_value = "r")]
    pub restart_hotkey: char,

    /// Color for supervisor log messages (default: yellow)
    /// Supported: yellow, red, green, blue, cyan, magenta, white, none
    #[arg(long, default_value = "yellow")]
    pub log_color: String,

    /// Color for informational messages (default: green)
    /// Supported: yellow, red, green, blue, cyan, magenta, white, none
    #[arg(long, default_value = "green")]
    pub info_color: String,

    /// Suppress all supervisor output (child process output still visible)
    #[arg(long)]
    pub silent: bool,

    /// Restart debounce time in milliseconds (default: 1000). Set to 0 to disable debouncing.
    #[arg(long, default_value = "1000")]
    pub restart_debounce_ms: u64,

    /// Command to run
    #[arg(required = true)]
    pub command: String,

    /// Arguments to pass to the command
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}
