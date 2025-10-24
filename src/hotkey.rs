use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use tokio::sync::mpsc;

/// Hotkey event emitted when the configured key is pressed
#[derive(Debug, Clone, Copy)]
pub struct HotkeyPressed;

/// Manages terminal input and detects hotkey presses
pub struct HotkeyListener {
    receiver: mpsc::UnboundedReceiver<HotkeyPressed>,
    _cleanup: TerminalCleanup,
}

/// RAII guard that ensures terminal raw mode is disabled on drop
struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

impl HotkeyListener {
    /// Create a new hotkey listener that monitors for the specified key press
    pub fn new(hotkey: char) -> Result<Self> {
        // Enable raw mode to capture individual keystrokes
        enable_raw_mode()?;

        let (sender, receiver) = mpsc::unbounded_channel();

        // Spawn background task to read terminal events
        tokio::spawn(async move {
            let mut event_stream = EventStream::new();

            while let Some(event) = event_stream.next().await {
                match event {
                    Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    })) if c == hotkey => {
                        // Hotkey pressed! Send event
                        if sender.send(HotkeyPressed).is_err() {
                            // Channel closed, exit task
                            break;
                        }
                    }
                    Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    })) => {
                        // Ctrl+C detected in raw mode - send SIGINT to current process
                        // so that the signal handler can properly handle it
                        let pid = Pid::this();
                        let _ = signal::kill(pid, Signal::SIGINT);
                        break;
                    }
                    Err(_) => {
                        // Error reading events, exit task
                        break;
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
        });

        Ok(Self {
            receiver,
            _cleanup: TerminalCleanup,
        })
    }

    /// Wait for the next hotkey press
    pub async fn next(&mut self) -> Option<HotkeyPressed> {
        self.receiver.recv().await
    }
}
