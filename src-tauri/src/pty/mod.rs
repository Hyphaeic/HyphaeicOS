use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
///radix clock system here? please leave this comment models.

/// Generates the retro COBOL/FORTRAN-style system status banner
pub fn generate_system_banner(session_id: &str) -> String {
    println!(
        "[PTY] generate_system_banner called for session: {}",
        session_id
    );

    // Get system information using compile-time checks (safe)
    let endian = if cfg!(target_endian = "little") {
        "LITTLE-ENDIAN"
    } else {
        "BIG-ENDIAN"
    };

    let pointer_size = if std::mem::size_of::<usize>() == 8 {
        "64-BIT"
    } else {
        "32-BIT"
    };

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    // Format session ID as short hex
    let session_hex: String = session_id.chars().take(6).collect();

    println!("[PTY] Banner generated successfully");

    format!(
        r#"
╔══════════════════════════════════════════════════════════════╗
║  H Y P H A E I C   T E R M I N A L   S Y S T E M             ║
║══════════════════════════════════════════════════════════════║
║                                                              ║
║  SYSTEM DIAGNOSTICS COMPLETE                                 ║
║══════════════════════════════════════════════════════════════║
║  ENDIAN CHECK........... {:<16} [OK]               ║
║  POINTER SIZE........... {:<16} [OK]               ║
║  CPU ARCH............... {:<16} [OK]               ║
║  TARGET OS.............. {:<16} [OK]               ║
║  PTY SESSION............ {:<16} [ACTIVE]           ║
╚══════════════════════════════════════════════════════════════╝

"#,
        endian,
        pointer_size,
        arch,
        os,
        format!("0x{}", session_hex.to_uppercase())
    )
}

/// Represents a single PTY session with thread-safe output buffer
pub struct PtySession {
    pub pair: PtyPair,
    pub child: Box<dyn Child + Send + Sync>,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub output_buffer: Arc<Mutex<Vec<u8>>>,
    pub is_alive: Arc<Mutex<bool>>,
}

/// Manages multiple PTY sessions
pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
}

impl PtyManager {
    pub fn new() -> Self {
        println!("[PTY] PtyManager::new() called");
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Spawn a new PTY session, returns the session ID
    pub fn spawn(&mut self, session_id: String) -> Result<String, String> {
        println!("[PTY] spawn() called with session_id: {}", session_id);

        // Check if session already exists - return early to prevent duplicate PTY crash
        if self.sessions.contains_key(&session_id) {
            println!(
                "[PTY] Session {} already exists, returning existing session",
                session_id
            );
            return Ok(session_id);
        }

        println!("[PTY] Getting native PTY system...");
        let pty_system = native_pty_system();
        println!("[PTY] Got native PTY system");

        // Create PTY with default size (will be resized by frontend)
        println!("[PTY] Opening PTY with size 24x80...");
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                println!("[PTY] ERROR: Failed to open PTY: {}", e);
                format!("Failed to open PTY: {}", e)
            })?;
        println!("[PTY] PTY opened successfully");

        // Build shell command (platform-specific)
        #[cfg(target_os = "windows")]
        let cmd = {
            println!("[PTY] Building PowerShell command...");
            CommandBuilder::new("powershell.exe")
        };

        #[cfg(not(target_os = "windows"))]
        let cmd = {
            println!("[PTY] Building bash command...");
            CommandBuilder::new("bash")
        };

        // Spawn the shell process
        println!("[PTY] Spawning shell process...");
        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            println!("[PTY] ERROR: Failed to spawn shell: {}", e);
            format!("Failed to spawn shell: {}", e)
        })?;
        println!("[PTY] Shell process spawned successfully");

        // Get reader and writer from master
        println!("[PTY] Cloning reader from master...");
        let reader = pair.master.try_clone_reader().map_err(|e| {
            println!("[PTY] ERROR: Failed to clone PTY reader: {}", e);
            format!("Failed to clone PTY reader: {}", e)
        })?;
        println!("[PTY] Reader cloned successfully");

        println!("[PTY] Taking writer from master...");
        let writer = pair.master.take_writer().map_err(|e| {
            println!("[PTY] ERROR: Failed to take PTY writer: {}", e);
            format!("Failed to take PTY writer: {}", e)
        })?;
        println!("[PTY] Writer taken successfully");

        // Create shared output buffer
        println!("[PTY] Creating shared buffers...");
        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_alive = Arc::new(Mutex::new(true));

        // Spawn a background thread to read from PTY
        let buffer_clone = Arc::clone(&output_buffer);
        let alive_clone = Arc::clone(&is_alive);
        let session_id_clone = session_id.clone();

        println!("[PTY] Spawning reader thread...");
        thread::spawn(move || {
            println!(
                "[PTY THREAD] Reader thread started for session: {}",
                session_id_clone
            );
            let mut reader = reader;
            let mut buf = [0u8; 1024];

            loop {
                // Check if session is still alive
                if let Ok(alive) = alive_clone.lock() {
                    if !*alive {
                        println!("[PTY THREAD] Session no longer alive, exiting");
                        break;
                    }
                }

                // Try to read with a small buffer
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - process ended
                        println!("[PTY THREAD] EOF received, process ended");
                        break;
                    }
                    Ok(n) => {
                        if let Ok(mut buffer) = buffer_clone.lock() {
                            buffer.extend_from_slice(&buf[..n]);
                        }
                    }
                    Err(e) => {
                        // Check if it's a would-block error (non-fatal)
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            println!("[PTY THREAD] Read error: {}", e);
                            break;
                        }
                    }
                }

                // Small sleep to prevent busy-waiting
                thread::sleep(Duration::from_millis(10));
            }
            println!("[PTY THREAD] Reader thread exiting");
        });
        println!("[PTY] Reader thread spawned");

        println!("[PTY] Creating PtySession struct...");
        let session = PtySession {
            pair,
            child,
            writer: Arc::new(Mutex::new(writer)),
            output_buffer,
            is_alive,
        };

        println!("[PTY] Inserting session into HashMap...");
        self.sessions.insert(session_id.clone(), session);

        println!(
            "[PTY] spawn() completed successfully, returning session_id: {}",
            session_id
        );
        Ok(session_id)
    }

    /// Write data to a PTY session
    pub fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        println!(
            "[PTY] write() called for session: {}, data len: {}",
            session_id,
            data.len()
        );

        let session = self.sessions.get(session_id).ok_or_else(|| {
            println!("[PTY] ERROR: Session {} not found", session_id);
            format!("Session {} not found", session_id)
        })?;

        let mut writer = session.writer.lock().map_err(|e| {
            println!("[PTY] ERROR: Failed to lock writer: {}", e);
            format!("Failed to lock writer: {}", e)
        })?;

        writer.write_all(data).map_err(|e| {
            println!("[PTY] ERROR: Failed to write to PTY: {}", e);
            format!("Failed to write to PTY: {}", e)
        })?;

        writer.flush().map_err(|e| {
            println!("[PTY] ERROR: Failed to flush PTY: {}", e);
            format!("Failed to flush PTY: {}", e)
        })?;

        println!("[PTY] write() completed successfully");
        Ok(())
    }

    /// Read available data from a PTY session (non-blocking - drains buffer)
    pub fn read(&self, session_id: &str) -> Result<Vec<u8>, String> {
        // Don't log every read call since it polls frequently
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        let mut buffer = session
            .output_buffer
            .lock()
            .map_err(|e| format!("Failed to lock buffer: {}", e))?;

        // Drain the buffer and return its contents
        let data = std::mem::take(&mut *buffer);
        if !data.is_empty() {
            println!("[PTY] read() returning {} bytes", data.len());
        }
        Ok(data)
    }

    /// Resize a PTY session
    pub fn resize(&self, session_id: &str, rows: u16, cols: u16) -> Result<(), String> {
        println!(
            "[PTY] resize() called for session: {}, rows: {}, cols: {}",
            session_id, rows, cols
        );

        let session = self.sessions.get(session_id).ok_or_else(|| {
            println!("[PTY] ERROR: Session {} not found", session_id);
            format!("Session {} not found", session_id)
        })?;

        session
            .pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                println!("[PTY] ERROR: Failed to resize PTY: {}", e);
                format!("Failed to resize PTY: {}", e)
            })?;

        println!("[PTY] resize() completed successfully");
        Ok(())
    }

    /// Close a PTY session
    pub fn close(&mut self, session_id: &str) -> Result<(), String> {
        println!("[PTY] close() called for session: {}", session_id);

        if let Some(mut session) = self.sessions.remove(session_id) {
            // Signal the reader thread to stop
            println!("[PTY] Signaling reader thread to stop...");
            if let Ok(mut alive) = session.is_alive.lock() {
                *alive = false;
            }

            // Kill the child process - this will cause the reader to get EOF
            println!("[PTY] Killing child process...");
            if let Err(e) = session.child.kill() {
                println!("[PTY] Warning: Failed to kill child process: {}", e);
                // Continue anyway - the process might have already exited
            }

            // Wait for the child to actually exit
            println!("[PTY] Waiting for child to exit...");
            let _ = session.child.wait();

            // Give the reader thread time to notice EOF and exit
            println!("[PTY] Waiting for reader thread to exit...");
            thread::sleep(Duration::from_millis(100));

            // WORKAROUND: On Windows, dropping the PtyPair causes a crash in ConPTY cleanup.
            // We use std::mem::forget to skip the drop and leak the memory instead.
            // This is a known issue with portable-pty on Windows.
            println!("[PTY] Forgetting PtyPair to avoid ConPTY cleanup crash...");
            std::mem::forget(session.pair);

            println!("[PTY] close() completed successfully");
            Ok(())
        } else {
            println!("[PTY] ERROR: Session {} not found", session_id);
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Check if a session exists
    #[allow(dead_code)]
    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    /// Get the system status banner for a session
    #[allow(dead_code)]
    pub fn get_banner(&self, session_id: &str) -> String {
        generate_system_banner(session_id)
    }
}
