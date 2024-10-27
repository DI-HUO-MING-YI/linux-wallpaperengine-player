use std::path::Path;
use std::{process::Child, thread::sleep, time::Duration};

use nix::libc::{killpg, SIGKILL};

// const MSG_SIZE: usize = 256;
// const MSG_TYPE: i64 = 1;

#[repr(C)]
pub struct Message {
    pub mtype: i64,
    pub mtext: [u8; 256],
}

pub fn kill_process(process: &mut Child) {
    let pid = process.id();
    unsafe {
        if killpg(process.id() as i32, SIGKILL) == 0 {
            println!("Successfully killed process group {}", process.id());
        } else {
            eprintln!("Failed to kill process group {}", process.id());
        }
    } // 等待片刻，检查进程状态
    sleep(Duration::from_millis(100));

    // 再次检查进程是否已终止，如果没有，使用 SIGKILL 强制终止
    unsafe {
        if killpg(pid as i32, 0) == 0 {
            println!("Process group {} still alive, sending SIGKILL", pid);
            if killpg(pid as i32, SIGKILL) == 0 {
                println!("Successfully killed process group {}", pid);
            } else {
                eprintln!("Failed to kill process group {}", pid);
            }
        } else {
            println!("Process group {} has already terminated", pid);
        }
    }
    match process.wait() {
        Ok(status) => println!("Process exited with status: {:?}", status),
        Err(e) => eprintln!("Failed to wait for process: {}", e),
    }
}

pub fn extract_last_directory_name(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if let Some(last_segment) = parent.file_name() {
            return last_segment.to_str().map(|s| s.to_string());
        }
    }
    None
}

pub fn secs_to_nanos(secs: f64) -> u64 {
    (secs * 1_000_000_000.0) as u64
}
