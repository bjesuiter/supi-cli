use crate::output::Output;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{Duration, timeout};

pub struct ProcessManager {
    command: String,
    args: Vec<String>,
    child: Option<Child>,
    output: Output,
}

impl ProcessManager {
    pub fn new(command: String, args: Vec<String>, output: Output) -> Self {
        Self {
            command,
            args,
            child: None,
            output,
        }
    }

    pub async fn spawn(&mut self) -> Result<()> {
        if self.child.is_some() {
            anyhow::bail!("Process already running");
        }

        self.output.log(&format!(
            "[supi] Starting child process: {} {:?}",
            self.command, self.args
        ));

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn child process")?;

        // Get stdout/stderr handles
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture child stdout")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to capture child stderr")?;

        // Clone output for spawned tasks
        let output_stdout = self.output.clone();
        let output_stderr = self.output.clone();

        // Spawn tasks to forward output
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                output_stdout.forward_stdout(&line);
            }
        });

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                output_stderr.forward_stderr(&line);
            }
        });

        let pid = child.id().unwrap_or(0);
        self.child = Some(child);

        self.output
            .log(&format!("[supi] Child process running (PID: {})", pid));

        Ok(())
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        if let Some(child) = &mut self.child {
            let status = child.wait().await.context("Failed to wait on child")?;
            self.child = None;
            Ok(status)
        } else {
            anyhow::bail!("No process running")
        }
    }

    pub async fn restart(&mut self) -> Result<()> {
        self.output.log("[supi] Restarting child process...");
        self.shutdown().await?;
        self.spawn().await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            self.output
                .log("[supi] Stopping child process gracefully...");

            // Try graceful shutdown with SIGTERM first
            #[cfg(unix)]
            {
                use nix::sys::signal::{Signal, kill};
                use nix::unistd::Pid;

                if let Some(pid) = child.id() {
                    // Send SIGTERM for graceful shutdown
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);

                    // Wait up to 5 seconds for graceful exit
                    match timeout(Duration::from_secs(5), child.wait()).await {
                        Ok(Ok(_status)) => {
                            self.output.log("[supi] Child process stopped gracefully");
                            return Ok(());
                        }
                        Ok(Err(e)) => {
                            self.output
                                .elog(&format!("[supi] Error waiting for child process: {}", e));
                        }
                        Err(_) => {
                            self.output
                                .log("[supi] Child process didn't stop gracefully, forcing...");
                        }
                    }
                }
            }

            // Force kill if graceful shutdown failed or on non-Unix platforms
            child.kill().await.context("Failed to kill child process")?;
            let _ = child.wait().await;
            self.output.log("[supi] Child process stopped");
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}
