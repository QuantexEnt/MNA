//! Real-time Teams mute-state watcher, feeding a shared atomic flag.
//! Ported from the validated mute_detection_poc (ui_watch.rs). Same
//! detection rule proven there: find the button whose accessible name
//! contains "mute mic" - "Mute mic" text = currently unmuted and
//! transmitting, "Unmute mic" text = currently muted.
//!
//! See DECISIONS.md #013-A for the full rationale and known limitations
//! (validated against one Teams build and one Bluetooth headset; a
//! future Teams UI update could change this and needs re-verification).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use windows::core::{Interface, BSTR};
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, TreeScope_Descendants,
    UIA_ButtonControlTypeId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
};

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
    hwnds: Vec<HWND>,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);
    if IsWindowVisible(hwnd).as_bool() {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == state.target_pid {
            state.hwnds.push(hwnd);
        }
    }
    true.into()
}

fn find_visible_hwnds_for_pid(pid: u32) -> Vec<HWND> {
    let mut state = EnumState {
        target_pid: pid,
        hwnds: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut state as *mut _ as isize),
        );
    }
    state.hwnds
}

fn find_mute_button_text(automation: &IUIAutomation, hwnd: HWND) -> Option<String> {
    let root: IUIAutomationElement = unsafe { automation.ElementFromHandle(hwnd).ok()? };
    let condition = unsafe { automation.CreateTrueCondition().ok()? };
    let elements = unsafe { root.FindAll(TreeScope_Descendants, &condition).ok()? };
    let count = unsafe { elements.Length().ok()? };

    for i in 0..count {
        let element = unsafe { elements.GetElement(i) }.ok()?;
        let control_type = unsafe { element.CurrentControlType() }.ok()?;
        if control_type != UIA_ButtonControlTypeId {
            continue;
        }
        let name: BSTR = match unsafe { element.CurrentName() } {
            Ok(n) => n,
            Err(_) => continue,
        };
        let name = name.to_string();
        if name.to_lowercase().contains("mute mic") {
            return Some(name);
        }
    }
    None
}

/// Starts a background thread polling Teams' mute button text and updating
/// `is_unmuted` accordingly. Stops when `should_stop` is set to true.
///
/// FAIL-CLOSED: `is_unmuted` starts false and is only ever set true once
/// we positively read "Mute mic" text (meaning currently unmuted). If the
/// button can't be found, or UI Automation fails entirely, it stays/goes
/// false - the mic capture gate stays shut. DECISIONS.md #013-A.
pub fn start_watching(is_unmuted: Arc<AtomicBool>, should_stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let automation: IUIAutomation =
            match unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL) } {
                Ok(a) => a,
                Err(_) => {
                    eprintln!(
                        "[audio] Could not start UI Automation - mic capture stays disabled."
                    );
                    return;
                }
            };

        while !should_stop.load(Ordering::SeqCst) {
            let pids = find_teams_pids();
            let mut found_unmuted = false;
            let mut found_any_button = false;

            for pid in pids {
                for hwnd in find_visible_hwnds_for_pid(pid) {
                    if let Some(text) = find_mute_button_text(&automation, hwnd) {
                        found_any_button = true;
                        if text.to_lowercase() == "mute mic" {
                            found_unmuted = true;
                        }
                    }
                }
            }

            is_unmuted.store(found_any_button && found_unmuted, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(250));
        }
    });
}
