mod ai_notes;
mod audio;
mod generation_input;
mod mom;
mod openai_http;
mod paths;
mod session;
mod settings;
mod teams;
mod transcription;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    if let Some(result) = audio::stop_recording_if_active() {
        match result {
            Ok(path) => println!("[audio] Finalized recording on exit: {path}"),
            Err(e) => eprintln!("[audio] Could not finalize recording on exit: {e}"),
        }
    }
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            paths::init(&handle)?;

            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Meeting Note Assistant")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        if let Some(result) = audio::stop_recording_if_active() {
                            match result {
                                Ok(path) => println!("[audio] Finalized recording on exit: {path}"),
                                Err(e) => eprintln!("[audio] Could not finalize recording on exit: {e}"),
                            }
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            teams::start_detector(handle);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            audio::start_recording,
            audio::stop_recording,
            session::save_notes,
            session::list_meetings,
            session::get_notes,
            settings::load_settings,
            settings::save_settings,
            transcription::transcribe_meeting,
            transcription::get_transcript,
            transcription::save_speaker_names,
            transcription::get_speaker_names,
            transcription::test_openai_connection,
            mom::generate_mom,
            mom::regenerate_mom,
            mom::get_mom,
            mom::get_mom_markdown,
            mom::save_mom,
            ai_notes::generate_ai_notes,
            ai_notes::regenerate_ai_notes,
            ai_notes::get_ai_notes,
            ai_notes::get_ai_notes_markdown,
            ai_notes::save_ai_notes,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}