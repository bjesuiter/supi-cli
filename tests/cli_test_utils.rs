use portable_pty::{native_pty_system, PtyPair, PtySize};
use std::io::Read;
use std::sync::{Arc, Mutex};

/// Helper function to create PTY test environment
/// 
/// Returns a tuple of:
/// - PtyPair: The PTY master/slave pair
/// - Arc<Mutex<Vec<u8>>>: Shared buffer containing all output from the PTY
/// - JoinHandle: Thread that reads from the PTY into the buffer
///
/// All tests use a PTY setup for testing to avoid display issues in cargo output
/// due to the use of raw tty mode in hotkey.rs. PTY provides a clean, realistic
/// terminal environment for testing.
pub fn create_pty_with_reader() -> (
    PtyPair,
    Arc<Mutex<Vec<u8>>>,
    std::thread::JoinHandle<()>,
) {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let output = Arc::new(Mutex::new(Vec::new()));
    let output_clone = Arc::clone(&output);
    let mut reader = pair.master.try_clone_reader().unwrap();

    let reader_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    output_clone.lock().unwrap().extend_from_slice(&buf[..n]);
                }
                _ => break,
            }
        }
    });

    (pair, output, reader_thread)
}

