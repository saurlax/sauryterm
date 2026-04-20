use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter, State};

#[derive(Clone)]
struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn Child + Send>>>,
}

#[derive(Clone, Default)]
pub struct PtyState {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PtyDataEvent {
    pty_id: String,
    data: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PtyExitEvent {
    pty_id: String,
    code: Option<i32>,
}

fn emit_data(app: &AppHandle, pty_id: &str, data: String) {
    let payload = PtyDataEvent {
        pty_id: pty_id.to_string(),
        data,
    };
    let _ = app.emit("pty://data", payload);
}

#[tauri::command]
pub fn open_pty(
    app: AppHandle,
    state: State<PtyState>,
    pty_id: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
) -> Result<String, String> {
    let pty_id = pty_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let pty_system = native_pty_system();
    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 40,
            cols: 140,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|err| format!("failed to create pty: {err}"))?;

    let mut cmd = if let Some(command) = command {
        CommandBuilder::new(command)
    } else if cfg!(target_os = "windows") {
        let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
        CommandBuilder::new(shell)
    } else {
        CommandBuilder::new(std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()))
    };

    if let Some(args) = args {
        for arg in args {
            cmd.arg(arg);
        }
    }

    let child = pty_pair
        .slave
        .spawn_command(cmd)
        .map_err(|err| format!("failed to spawn shell in pty: {err}"))?;

    let mut reader = pty_pair
        .master
        .try_clone_reader()
        .map_err(|err| format!("failed to clone pty reader: {err}"))?;
    let writer = pty_pair
        .master
        .take_writer()
        .map_err(|err| format!("failed to take pty writer: {err}"))?;

    let session = PtySession {
        writer: Arc::new(Mutex::new(writer)),
        child: Arc::new(Mutex::new(child)),
    };

    state
        .sessions
        .lock()
        .map_err(|_| "failed to lock terminal sessions".to_string())?
        .insert(pty_id.clone(), session.clone());

    {
        let app = app.clone();
        let pty_id = pty_id.clone();
        thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(len) => {
                        let chunk = String::from_utf8_lossy(&buf[..len]).into_owned();
                        emit_data(&app, &pty_id, chunk)
                    }
                    Err(_) => break,
                }
            }
        });
    }

    {
        let app = app.clone();
        let pty_id = pty_id.clone();
        let sessions = Arc::clone(&state.sessions);
        let child = Arc::clone(&session.child);
        thread::spawn(move || {
            loop {
                let status = child
                    .lock()
                    .ok()
                    .and_then(|mut child| child.try_wait().ok())
                    .flatten();

                if let Some(status) = status {
                    if let Ok(mut sessions) = sessions.lock() {
                        sessions.remove(&pty_id);
                    }
                    let payload = PtyExitEvent {
                        pty_id: pty_id.clone(),
                        code: i32::try_from(status.exit_code()).ok(),
                    };
                    let _ = app.emit("pty://exit", payload);
                    break;
                }

                thread::sleep(Duration::from_millis(50));
            }
        });
    }

    Ok(pty_id)
}

#[tauri::command]
pub fn write_pty(state: State<PtyState>, pty_id: String, data: String) -> Result<(), String> {
    let session = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock terminal sessions".to_string())?
        .get(&pty_id)
        .cloned()
        .ok_or_else(|| format!("terminal session not found: {pty_id}"))?;

    let mut writer = session
        .writer
        .lock()
        .map_err(|_| "failed to lock terminal writer".to_string())?;
    writer
        .write_all(data.as_bytes())
        .map_err(|err| format!("failed to write to pty: {err}"))?;
    writer
        .flush()
        .map_err(|err| format!("failed to flush pty: {err}"))?;
    Ok(())
}

#[tauri::command]
pub fn close_pty(state: State<PtyState>, pty_id: String) -> Result<(), String> {
    let session = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock terminal sessions".to_string())?
        .remove(&pty_id);

    if let Some(session) = session {
        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
        }
    }

    Ok(())
}
