mod cli;
mod input;
mod output;
mod process;
mod signals;
mod supervisor;

use clap::Parser;
use cli::Cli;
use input::CommandReader;
use process::ProcessManager;
use signals::SignalHandler;
use supervisor::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    sprintln!("[supi] Supervisor PID: {}", std::process::id());
    sprintln!("[supi] Starting supervisor");
    sprintln!(
        "[supi] Config: restart_signal={}, stop_on_child_exit={}",
        args.restart_signal,
        args.stop_on_child_exit
    );

    let process_manager = ProcessManager::new(args.command, args.args);
    let signal_handler = SignalHandler::new(&args.restart_signal)?;

    // Set up stdin command reader
    let command_reader = match CommandReader::new() {
        Ok(reader) => {
            sprintln!("[supi] Command reader active: type 'help' for available commands");
            Some(reader)
        }
        Err(e) => {
            seprintln!("[supi] Warning: Could not enable command reader: {}", e);
            seprintln!("[supi] Continuing without stdin commands (signals still work)");
            None
        }
    };

    let mut supervisor = Supervisor::new(
        process_manager,
        signal_handler,
        command_reader,
        args.stop_on_child_exit,
    );

    supervisor.run().await?;

    Ok(())
}
