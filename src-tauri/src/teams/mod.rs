//! Teams meeting detection (Phase 2).
//!
//! DETECTION RULE — established empirically against the actually-installed
//! "new" Teams client (see project chat log for the raw evidence):
//!
//!   1. Find all processes whose name contains "teams" (do NOT assume a
//!      fixed exe name — AUDIO_CAPTURE.md §19, DECISIONS.md #008).
//!   2. For each such process, count its VISIBLE top-level windows.
//!   3. A meeting is considered ACTIVE if a process has >= 2 visible
//!      windows AND at least one window's title does not match a known
//!      static Teams navigation label (Chat, Calendar, Activity, Meet,
//!      People, Calls, Files, Apps, Teams, Notifications, Search).
//!
//! Why two signals combined: a Team/channel name is also arbitrary text
//! and could alone produce an "unrecognized title" false positive. But
//! browsing to a channel does NOT add a second window in testing — only
//! an actual call did. Requiring both conditions together protects
//! against that specific false-positive path.
//!
//! KNOWN LIMITATION (be honest about this, don't silently paper over it
//! later — DECISIONS.md "Known Technical Limitation Policy"): this was
//! validated against ONE Teams installation/version with ONE 1:1 test
//! meeting. It has not been tested against: channel/group calls, calls
//! initiated from a channel, Teams updates changing window behavior, or
//! the classic (non-"new") Teams client. If meeting detection later
//! misfires, check this heuristic first before assuming something else
//! is wrong.
//!
//! Phase 2 scope only: this module does NOT start audio capture, does
//! NOT show any notification UI (that's Phase 3), and does NOT create a
//! meeting session. It only maintains and reports detection state via
//! Tauri events.

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{BOOL, CloseHandle, HWND, LPARAM};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};

const KNOWN_IDLE_LABELS: &[&str] = &[
    "Chat",
    "Calendar",
    "Activity",
    "Meet",
    "People",
    "Calls",
    "Files",
    "Apps",
    "Teams",
    "Notifications",
    "Search",
];

const POLL_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectorState {
    NotRunning,
    Running,
    MeetingActive,
}

fn find_teams_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return pids,
        };
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let end = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..end]).to_lowercase();
                if name.contains("teams") {
                    pids.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    pids
}

struct EnumState {
    target_pid: u32,
    titles: Vec<String>,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);

    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid != state.target_pid {
        return true.into();
    }

    let mut title_buf = [0u16; 512];
    let title_len = GetWindowTextW(hwnd, &mut title_buf);
    if title_len > 0 {
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
        if !title.trim().is_empty() {
            state.titles.push(title);
        }
    }
    true.into()
}

fn get_visible_titles_for_pid(pid: u32) -> Vec<String> {
    let mut state = EnumState {
        target_pid: pid,
        titles: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut state as *mut _ as isize),
        );
    }
    state.titles
}

fn is_known_idle_title(title: &str) -> bool {
    KNOWN_IDLE_LABELS
        .iter()
        .any(|label| title == format!("{label} | Microsoft Teams"))
}

/// True if ANY teams-related process currently looks like it's in an
/// active meeting, per the heuristic documented at the top of this file.
fn is_any_meeting_active(pids: &[u32]) -> bool {
    pids.iter().any(|&pid| {
        let titles = get_visible_titles_for_pid(pid);
        titles.len() >= 2 && titles.iter().any(|t| !is_known_idle_title(t))
    })
}

fn poll_once(current: &mut DetectorState, app: &AppHandle) {
    let pids = find_teams_pids();

    let new_state = if pids.is_empty() {
        DetectorState::NotRunning
    } else if is_any_meeting_active(&pids) {
        DetectorState::MeetingActive
    } else {
        DetectorState::Running
    };

    if *current != new_state {
        // Console log for now - Phase 2 has no notification UI yet
        // (that's Phase 3). This is our visual confirmation while testing.
        println!("[teams-detector] {:?} -> {:?}", current, new_state);

        if new_state == DetectorState::MeetingActive {
            let _ = app.emit("teams-meeting-detected", ());
        } else if *current == DetectorState::MeetingActive {
            // Leaving MeetingActive either to Running or NotRunning both
            // count as "meeting ended" - covers Teams closing abruptly
            // mid-meeting too (USER_FLOW.md §14).
            let _ = app.emit("teams-meeting-ended", ());
        }

        *current = new_state;
    }
}

/// Starts a background polling thread that watches Teams meeting state
/// and emits "teams-meeting-detected" / "teams-meeting-ended" events to
/// the frontend on transitions only (not every poll - avoids duplicate
/// notifications per CLAUDE_INSTRUCTIONS.md §26).
///
/// Call once from lib.rs's .setup().
pub fn start_detector(app: AppHandle) {
    thread::spawn(move || {
        let mut state = DetectorState::NotRunning;
        loop {
            poll_once(&mut state, &app);
            thread::sleep(POLL_INTERVAL);
        }
    });
}
