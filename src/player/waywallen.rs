use std::{
    env,
    net::TcpStream,
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use prost::Message;
use tungstenite::{client::client, Message as WebSocketMessage, WebSocket};
use zbus::blocking::{Connection, Proxy};

const DBUS_SERVICE: &str = "org.waywallen.waywallen.Daemon";
const DBUS_PATH: &str = "/org/waywallen/waywallen/Daemon";
const DBUS_INTERFACE: &str = "org.waywallen.waywallen.Daemon1";
const DEFAULT_WAYWALLEN_EXECUTABLE: &str =
    "/home/narcissus/Application/waywallen-0.6.0-x86_64.AppImage";
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const WEBSOCKET_TIMEOUT: Duration = Duration::from_secs(5);

pub fn next() -> Result<()> {
    call_without_arguments("Next")
}

pub fn previous() -> Result<()> {
    call_without_arguments("Previous")
}

pub fn open_ui() -> Result<()> {
    call_without_arguments("OpenUi")
}

pub fn toggle_pause() -> Result<()> {
    let paused = WaywallenWebSocket::connect()?.toggle_pause()?;
    println!("Waywallen paused: {paused}");
    Ok(())
}

pub fn toggle_mute() -> Result<()> {
    let mut websocket = WaywallenWebSocket::connect()?;
    let muted = websocket.global_muted()?;
    let muted = websocket.set_muted(!muted)?;
    println!("Waywallen muted: {muted}");
    Ok(())
}

pub fn restart() -> Result<()> {
    let connection = Connection::session().context("Failed to connect to the session bus")?;
    let previous_pid = daemon_pid(&connection)?;

    if let Some(pid) = previous_pid {
        waywallen_proxy(&connection)?.call::<_, _, ()>("Quit", &())?;
        wait_for_shutdown(&connection, pid)?;
    }

    let mut child = spawn_waywallen()?;
    let new_pid = wait_until_ready(&connection, &mut child, previous_pid)?;
    println!("Waywallen restarted with PID {new_pid}");
    Ok(())
}

fn call_without_arguments(method: &str) -> Result<()> {
    let connection = Connection::session().context("Failed to connect to the session bus")?;
    waywallen_proxy(&connection)?.call::<_, _, ()>(method, &())?;
    Ok(())
}

fn waywallen_proxy(connection: &Connection) -> Result<Proxy<'_>> {
    Proxy::new(connection, DBUS_SERVICE, DBUS_PATH, DBUS_INTERFACE)
        .context("Failed to create the Waywallen D-Bus proxy")
}

fn daemon_pid(connection: &Connection) -> Result<Option<u32>> {
    let proxy = Proxy::new(
        connection,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )?;
    let has_owner: bool = proxy.call("NameHasOwner", &(DBUS_SERVICE,))?;
    if !has_owner {
        return Ok(None);
    }
    Ok(Some(
        proxy.call("GetConnectionUnixProcessID", &(DBUS_SERVICE,))?,
    ))
}

fn wait_for_shutdown(connection: &Connection, previous_pid: u32) -> Result<()> {
    let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
    while Instant::now() < deadline {
        if daemon_pid(connection)?.is_none() {
            return Ok(());
        }
        thread::sleep(POLL_INTERVAL);
    }
    bail!("Waywallen PID {previous_pid} did not exit within {SHUTDOWN_TIMEOUT:?}")
}

fn spawn_waywallen() -> Result<Child> {
    let executable = env::var_os("WAYWALLEN_EXECUTABLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_WAYWALLEN_EXECUTABLE));
    let mut command = Command::new(&executable);
    command
        .arg("--no-ui")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command
        .spawn()
        .with_context(|| format!("Failed to start {}", executable.display()))
}

fn wait_until_ready(
    connection: &Connection,
    child: &mut Child,
    previous_pid: Option<u32>,
) -> Result<u32> {
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            bail!("Waywallen exited during startup with {status}");
        }

        if let Some(pid) = daemon_pid(connection)? {
            if Some(pid) != previous_pid && websocket_port(connection).is_ok() {
                return Ok(pid);
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
    bail!(
        "Waywallen did not become ready within {STARTUP_TIMEOUT:?}; spawned process ID is {}",
        child.id()
    )
}

fn websocket_port(connection: &Connection) -> Result<u16> {
    Ok(waywallen_proxy(connection)?.get_property("WsPort")?)
}

struct WaywallenWebSocket {
    socket: WebSocket<TcpStream>,
    next_request_id: u64,
}

impl WaywallenWebSocket {
    fn connect() -> Result<Self> {
        let connection = Connection::session().context("Failed to connect to the session bus")?;
        let port = websocket_port(&connection)?;
        let stream = TcpStream::connect(("127.0.0.1", port))
            .with_context(|| format!("Failed to connect to Waywallen WebSocket port {port}"))?;
        stream.set_read_timeout(Some(WEBSOCKET_TIMEOUT))?;
        stream.set_write_timeout(Some(WEBSOCKET_TIMEOUT))?;
        let url = format!("ws://127.0.0.1:{port}");
        let (socket, _) = client(url.as_str(), stream)
            .context("Failed to establish the Waywallen WebSocket connection")?;
        Ok(Self {
            socket,
            next_request_id: 0,
        })
    }

    fn toggle_pause(&mut self) -> Result<bool> {
        let response = self.call(request::Payload::GlobalPauseToggle(
            GlobalPauseToggleRequest {},
        ))?;
        response
            .global_pause_toggle
            .map(|toggle| toggle.paused)
            .context("Waywallen returned no pause state")
    }

    fn global_muted(&mut self) -> Result<bool> {
        loop {
            match self.socket.read()? {
                WebSocketMessage::Binary(data) => {
                    let frame = ServerFrame::decode(data.as_ref())?;
                    if let Some(status) = frame.event.and_then(|event| event.status_sync) {
                        return Ok(status.global_muted);
                    }
                }
                WebSocketMessage::Close(_) => {
                    bail!("Waywallen closed the WebSocket before sending its status")
                }
                _ => {}
            }
        }
    }

    fn set_muted(&mut self, muted: bool) -> Result<bool> {
        let response = self.call(request::Payload::GlobalMuteSet(GlobalMuteSetRequest {
            muted,
        }))?;
        response
            .global_mute_set
            .map(|set| set.muted)
            .context("Waywallen returned no mute state")
    }

    fn call(&mut self, payload: request::Payload) -> Result<Response> {
        self.next_request_id += 1;
        let request_id = self.next_request_id;
        let request = Request {
            request_id,
            payload: Some(payload),
        };
        self.socket
            .send(WebSocketMessage::Binary(request.encode_to_vec().into()))?;

        loop {
            match self.socket.read()? {
                WebSocketMessage::Binary(data) => {
                    let frame = ServerFrame::decode(data.as_ref())?;
                    let Some(response) = frame.response else {
                        continue;
                    };
                    if response.request_id != request_id {
                        continue;
                    }
                    if response.status != Status::Ok as i32 {
                        bail!(
                            "Waywallen request {request_id} failed with status {}: {}",
                            response.status,
                            response.message
                        );
                    }
                    return Ok(response);
                }
                WebSocketMessage::Close(_) => {
                    bail!("Waywallen closed the WebSocket before answering request {request_id}")
                }
                _ => {}
            }
        }
    }
}

#[derive(Message)]
struct Request {
    #[prost(uint64, tag = "1")]
    request_id: u64,
    #[prost(oneof = "request::Payload", tags = "75, 77")]
    payload: Option<request::Payload>,
}

mod request {
    #[derive(prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "75")]
        GlobalPauseToggle(super::GlobalPauseToggleRequest),
        #[prost(message, tag = "77")]
        GlobalMuteSet(super::GlobalMuteSetRequest),
    }
}

#[derive(Message)]
struct ServerFrame {
    #[prost(message, optional, tag = "1")]
    response: Option<Response>,
    #[prost(message, optional, tag = "2")]
    event: Option<Event>,
}

#[derive(Message)]
struct Event {
    #[prost(message, optional, tag = "13")]
    status_sync: Option<StatusSync>,
}

#[derive(Message)]
struct StatusSync {
    #[prost(bool, tag = "6")]
    global_muted: bool,
}

#[derive(Message)]
struct Response {
    #[prost(uint64, tag = "1")]
    request_id: u64,
    #[prost(enumeration = "Status", tag = "2")]
    status: i32,
    #[prost(string, tag = "3")]
    message: String,
    #[prost(message, optional, tag = "75")]
    global_pause_toggle: Option<GlobalPauseToggleResponse>,
    #[prost(message, optional, tag = "77")]
    global_mute_set: Option<GlobalMuteSetResponse>,
}

#[derive(Clone, Copy, Debug, prost::Enumeration)]
#[repr(i32)]
enum Status {
    Unspecified = 0,
    Ok = 1,
    InvalidArgument = 2,
    NotFound = 3,
    Internal = 4,
    FailedPrecondition = 5,
}

#[derive(Message)]
struct GlobalPauseToggleRequest {}

#[derive(Message)]
struct GlobalPauseToggleResponse {
    #[prost(bool, tag = "1")]
    paused: bool,
}

#[derive(Message)]
struct GlobalMuteSetRequest {
    #[prost(bool, tag = "1")]
    muted: bool,
}

#[derive(Message)]
struct GlobalMuteSetResponse {
    #[prost(bool, tag = "1")]
    muted: bool,
}
