use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// Command event emitted when a command is entered via stdin
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Restart,
    Quit,
    Status,
    Help,
}

impl Command {
    /// Parse a command string into a Command enum
    fn parse(input: &str) -> Option<Self> {
        let input = input.trim().to_lowercase();
        match input.as_str() {
            "restart" | "r" => Some(Command::Restart),
            "quit" | "q" | "exit" => Some(Command::Quit),
            "status" | "s" => Some(Command::Status),
            "help" | "h" | "?" => Some(Command::Help),
            _ => None,
        }
    }

    /// Display help text for available commands
    pub fn help_text() -> &'static str {
        r#"Available commands:
  restart, r    - Restart the supervised process
  quit, q       - Shutdown the supervisor and child process
  status, s     - Display process status (not yet implemented)
  help, h, ?    - Show this help message"#
    }
}

/// Manages stdin input and parses commands
pub struct CommandReader {
    receiver: mpsc::UnboundedReceiver<Command>,
}

impl CommandReader {
    /// Create a new command reader that monitors stdin for commands
    pub fn new() -> Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();

        // Spawn background task to read stdin
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF reached (stdin closed)
                        break;
                    }
                    Ok(_) => {
                        if let Some(command) = Command::parse(&line) {
                            if sender.send(command).is_err() {
                                // Channel closed, exit task
                                break;
                            }
                        }
                        // Silently ignore unrecognized commands
                    }
                    Err(_) => {
                        // Error reading stdin, exit task
                        break;
                    }
                }
            }
        });

        Ok(Self { receiver })
    }

    /// Wait for the next command from stdin
    pub async fn next(&mut self) -> Option<Command> {
        self.receiver.recv().await
    }
}
