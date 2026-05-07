use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::config::{self, Allowlist};

const SOCKET_PATH: &str = "/tmp/zen.sock";

pub fn start_server(allowlist: Arc<RwLock<Allowlist>>, self_pid: u32) {
    // Remove existing socket if it exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("\x1b[31m[IPC ERROR]\x1b[0m Failed to bind IPC socket: {}", e);
            return;
        }
    };

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = String::new();
                    if let Ok(_) = stream.read_to_string(&mut buffer) {
                        let response = handle_ipc_command(&buffer, &allowlist, self_pid);
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
                Err(err) => {
                    eprintln!("\x1b[31m[IPC ERROR]\x1b[0m Connection failed: {}", err);
                }
            }
        }
    });
}

fn handle_ipc_command(command: &str, allowlist_lock: &Arc<RwLock<Allowlist>>, self_pid: u32) -> String {
    let cmd = command.trim();
    if cmd.starts_with("SET_PROFILE ") {
        let profile_name = &cmd["SET_PROFILE ".len()..];
        let path_str = if profile_name.contains('/') || profile_name.ends_with(".txt") {
            profile_name.to_string()
        } else {
            format!("profiles/{}.txt", profile_name)
        };
        
        let path = PathBuf::from(&path_str);
        match config::load_allowlist(&path) {
            Ok(new_allowlist) => {
                if let Ok(mut lock) = allowlist_lock.write() {
                    *lock = new_allowlist;
                    format!("OK: Profile updated to {}\n", profile_name)
                } else {
                    "ERROR: Failed to acquire write lock on allowlist\n".to_string()
                }
            }
            Err(e) => format!("ERROR: Failed to load profile {}: {}\n", profile_name, e),
        }
    } else if cmd == "STATUS" {
        let (rules, teams) = if let Ok(lock) = allowlist_lock.read() {
            (lock.names.len() + lock.paths.len(), lock.teams.len())
        } else {
            (0, 0)
        };

        let mem_info = if let Ok(sample) = crate::platform::sample_process(self_pid) {
            format!("{:.2} MB", sample.rss_bytes as f64 / 1024.0 / 1024.0)
        } else {
            "Unknown".to_string()
        };

        format!(
            "OK: Zen Daemon is running\nPID: {}\nMemory (RSS): {}\nRules: {} names/paths, {} teams\n",
            self_pid, mem_info, rules, teams
        )
    } else {
        format!("ERROR: Unknown command: {}\n", cmd)
    }
}

pub fn send_command(command: &str) -> Result<String, String> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .map_err(|e| format!("Failed to connect to daemon socket at {}: {}", SOCKET_PATH, e))?;
    
    stream.write_all(command.as_bytes())
        .map_err(|e| format!("Failed to send command: {}", e))?;
    
    // Shutdown write so the server knows we're done sending
    stream.shutdown(std::net::Shutdown::Write)
        .map_err(|e| format!("Failed to shutdown socket stream: {}", e))?;

    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(response)
}
