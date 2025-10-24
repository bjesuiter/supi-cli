use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "supi-cli")]
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

    /// Command to run
    #[arg(required = true)]
    pub command: String,

    /// Arguments to pass to the command
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}
