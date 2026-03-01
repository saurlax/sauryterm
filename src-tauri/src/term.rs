use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter, State};

#[derive(Clone)]
struct TerminalSession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn Child + Send>>>,
}

#[derive(Clone, Default)]
pub struct TerminalState {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TermDataEvent {
    term_id: String,
    data: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TermExitEvent {
    term_id: String,
    code: Option<i32>,
}

fn emit_data(app: &AppHandle, term_id: &str, data: String) {
    let payload = TermDataEvent {
        term_id: term_id.to_string(),
        data,
    };
    let _ = app.emit("term://data", payload);
}

#[tauri::command]
pub fn open_term(
    app: AppHandle,
    state: State<TerminalState>,
    term_id: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
) -> Result<String, String> {
    let term_id = term_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

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
        let mut default = CommandBuilder::new("powershell.exe");
        default.arg("-NoLogo");
        default
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

    let session = TerminalSession {
        writer: Arc::new(Mutex::new(writer)),
        child: Arc::new(Mutex::new(child)),
    };

    state
        .sessions
        .lock()
        .map_err(|_| "failed to lock terminal sessions".to_string())?
        .insert(term_id.clone(), session.clone());

    {
        let app = app.clone();
        let term_id = term_id.clone();
        thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(len) => {
                        emit_data(&app, &term_id, String::from_utf8_lossy(&buf[..len]).into_owned())
                    }
                    Err(_) => break,
                }
            }
        });
    }

    {
        let app = app.clone();
        let term_id = term_id.clone();
        let sessions = Arc::clone(&state.sessions);
        let child = Arc::clone(&session.child);
        thread::spawn(move || {
            let status = child.lock().ok().and_then(|mut child| child.wait().ok());
            if let Ok(mut sessions) = sessions.lock() {
                sessions.remove(&term_id);
            }
            let payload = TermExitEvent {
                term_id: term_id.clone(),
                code: status.and_then(|s| i32::try_from(s.exit_code()).ok()),
            };
            let _ = app.emit("term://exit", payload);
        });
    }

    Ok(term_id)
}

#[tauri::command]
pub fn write_term(state: State<TerminalState>, term_id: String, data: String) -> Result<(), String> {
    let session = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock terminal sessions".to_string())?
        .get(&term_id)
        .cloned()
        .ok_or_else(|| format!("terminal session not found: {term_id}"))?;

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
pub fn close_term(state: State<TerminalState>, term_id: String) -> Result<(), String> {
    let session = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock terminal sessions".to_string())?
        .remove(&term_id);

    if let Some(session) = session {
        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
        }
    }

    Ok(())
}
