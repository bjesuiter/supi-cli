use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// Hotkey event emitted when the configured key is pressed
#[derive(Debug, Clone, Copy)]
pub struct HotkeyPressed;

/// Manages terminal input and detects hotkey presses
pub struct HotkeyListener {
    hotkey: char,
    receiver: mpsc::UnboundedReceiver<HotkeyPressed>,
    _cleanup: Option<TerminalCleanup>,
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
    /// Note: This does not enable raw mode yet. Call `enable_raw_mode()` after
    /// validating that the command exists.
    pub fn new(hotkey: char) -> Result<Self> {
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
            hotkey,
            receiver,
            _cleanup: None,
        })
    }

    /// Enable raw mode for terminal input
    /// Call this after validating that the command exists and can be spawned
    pub fn enable_raw_mode(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self._cleanup = Some(TerminalCleanup);
        Ok(())
    }

    /// Get the configured hotkey character
    pub fn hotkey(&self) -> char {
        self.hotkey
    }

    /// Wait for the next hotkey press
    pub async fn next(&mut self) -> Option<HotkeyPressed> {
        self.receiver.recv().await
    }
}
