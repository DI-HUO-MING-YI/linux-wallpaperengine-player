use core::time;
use std::{
    ffi::{CStr, CString},
    io,
    str::FromStr,
    thread,
    time::Instant,
};

use log::info;
use nix::libc::{self, IPC_CREAT, IPC_NOWAIT};
use strum_macros::{Display, EnumString};

use crate::util::Message;

#[derive(EnumString, Display)]
pub enum ControlAction {
    Next,
    Prev,
    Reload,
    Stop,
    Continue,
}

impl ControlAction {
    pub fn to_state(current_state: Option<String>, action: &ControlAction) -> PlayState {
        match action {
            ControlAction::Stop => PlayState::Stopped,
            ControlAction::Continue => PlayState::Playing,
            _ => current_state.and_then(|state| PlayState::from_str(&state).ok()).unwrap_or(PlayState::Playing),
        }
    }

    pub fn form(message_string: &str) -> Option<ControlAction> {
        ControlAction::from_str(message_string).map_or(None, |action| Some(action))
    }
}

#[derive(Clone, Debug, EnumString, Display)]
pub enum PlayState {
    Playing,
    Stopped,
}
impl PlayState {
    pub(crate) fn is_stopped(state_str: &str) -> bool {
        state_str == PlayState::Stopped.to_string()
    }
}

pub fn control(action: Option<&str>) {
    if let Some(action) = action {
        let key = 22333;
        let msg_id = unsafe { libc::msgget(key, IPC_CREAT | 0o666) };

        info!("msg_id: {msg_id}");
        if msg_id == -1 {
            let err = io::Error::last_os_error(); // 获取错误信息
            eprintln!("Failed to create message queue: {}", err);
            panic!("Failed to create message queue");
        }

        let c_string = CString::new(action).expect("CString::new failed");
        let mut msg = Message {
            mtype: 1,
            mtext: [0; 256],
        };

        let bytes = c_string.as_bytes_with_nul();
        let len = bytes.len().min(256);
        msg.mtext[..len].copy_from_slice(bytes);

        let result = unsafe {
            libc::msgsnd(
                msg_id,
                &msg as *const _ as *const libc::c_void,
                len,
                IPC_CREAT,
            )
        };

        if result == -1 {
            let err = io::Error::last_os_error(); // 获取错误信息
            eprintln!("Failed to send message queue: {}", err);
            panic!("Failed to send message");
        }

        println!("Message sent!");
    }
}
pub fn wait_for_control_message(delay: &time::Duration) -> Option<ControlAction> {
    let start_time = Instant::now();

    loop {
        if &start_time.elapsed() >= delay {
            return None;
        }
        let mut msg = Message {
            mtype: 0,
            mtext: [0; 256],
        };

        let key = 22333;
        let msg_id = unsafe { libc::msgget(key, IPC_CREAT | 0o666) };

        info!("msg_id: {msg_id}");
        let result = unsafe {
            libc::msgrcv(
                msg_id,
                &mut msg as *mut _ as *mut libc::c_void,
                std::mem::size_of::<Message>() as libc::size_t,
                1,
                IPC_NOWAIT,
            )
        };

        if result == -1 {
            let err = io::Error::last_os_error();
            if unsafe { *libc::__errno_location() } == libc::EAGAIN {
                info!("No message available, retrying...");
            } else {
                eprintln!("Error receiving message: {}", err);
            }
            thread::sleep(time::Duration::from_millis(100));
            continue;
        }
        let received_message = unsafe { CStr::from_ptr(msg.mtext.as_ptr() as *const i8) };

        let message_string = received_message.to_string_lossy().into_owned();

        info!("Received message: {:?}", message_string);
        return ControlAction::form(&message_string);
    }
}
