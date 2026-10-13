//! Central app-data-directory resolution. Computed once at startup
//! (lib.rs's setup()) and cached in a process-wide OnceLock, so the
//! many existing helper functions across session.rs/settings.rs can
//! resolve storage paths without needing Tauri's AppHandle threaded
//! through every function signature across multiple files.
//!
//! Fixes a real bug: paths were previously computed from
//! CARGO_MANIFEST_DIR, a compile-time constant that bakes in this dev
//! machine's literal project folder path. A built app run on any other
//! machine would try to read/write to a folder that doesn't exist
//! there. This uses Tauri's runtime path resolver instead, which
//! correctly resolves per-user on whatever machine actually runs the
//! app.
//!
//! Uses the LOCAL (non-roaming) app data directory, matching
//! LOCAL_STORAGE_SPECIFICATION.md §3's %LOCALAPPDATA% recommendation,
//! not roaming %APPDATA% - relevant on corporate Windows machines with
//! roaming profiles, where syncing meeting audio to a server on
//! login/logoff would be slow and likely against IT policy.

use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Manager;

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Must be called exactly once, early in lib.rs's setup(), before any
/// command that touches storage could possibly run (the frontend can't
/// invoke anything until setup() completes, so this is safe).
pub fn init(app: &tauri::AppHandle) -> Result<(), String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("resolve app local data dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create app data dir: {e}"))?;
    APP_DATA_DIR
        .set(dir)
        .map_err(|_| "app data dir already initialized".to_string())?;
    Ok(())
}

/// Root folder for all app storage. Panics if init() was not called
/// first - a programming error (missing setup() wiring), not a runtime
/// condition the app should try to recover from.
fn app_data_dir() -> &'static PathBuf {
    APP_DATA_DIR
        .get()
        .expect("paths::init() must be called before any storage access")
}

pub fn meetings_root() -> PathBuf {
    app_data_dir().join("meetings")
}

pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}