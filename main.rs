use std::fs::OpenOptions;
use std::io::Write;
use std::ffi::CStr;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use libc;

const LOG_PATH: &str = "/media/card/crash.log";
const DAEMON_PATH: &str = "/media/card/rayhunter-daemon";
const CONFIG_PATH: &str = "/data/rayhunter/config.toml";
const CHECK_INTERVAL_SECS: u64 = 5;

unsafe extern "C" {
    fn time(t: *mut libc::time_t) -> libc::time_t;
    fn localtime(t: *const libc::time_t) -> *mut libc::tm;
    fn strftime(s: *mut libc::c_char, max: usize, format: *const libc::c_char, tm: *const libc::tm) -> usize;
}

fn get_timestamp() -> String {
    unsafe {
        let mut t: libc::time_t = 0;
        time(&mut t);

        let tm = localtime(&t);
        let mut buffer = [0u8; 32];
        let fmt = CStr::from_bytes_with_nul(b"%Y-%m-%d %H:%M:%S\0").unwrap();

        let len = strftime(
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
            fmt.as_ptr(),
            tm,
        );

        String::from_utf8_lossy(&buffer[..len]).to_string()
    }
}

fn log_crash(message: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_PATH) {
        let timestamp = get_timestamp();
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}

fn is_daemon_running() -> bool {
    // Simple check using `/proc` to see if the daemon is running
    for entry in std::fs::read_dir("/proc").unwrap() {
        if let Ok(entry) = entry {
            if let Ok(pid_str) = entry.file_name().into_string() {
                if pid_str.chars().all(|c| c.is_digit(10)) {
                    let cmdline_path = format!("/proc/{}/cmdline", pid_str);
                    if let Ok(cmdline) = std::fs::read_to_string(cmdline_path) {
                        if cmdline.contains("rayhunter-daemon") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn spawn_daemon() {
    match Command::new(DAEMON_PATH)
        .arg(CONFIG_PATH)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            log_crash("Daemon started.");
        }
        Err(e) => {
            log_crash(&format!("Failed to start daemon: {}", e));
        }
    }
}

fn main() {
    loop {
        if !is_daemon_running() {
            log_crash("Daemon not running. Restarting...");
            spawn_daemon();
        }
        thread::sleep(Duration::from_secs(CHECK_INTERVAL_SECS));
    }
}
