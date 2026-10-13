# PROJECT_STATUS.md — Meeting Note Assistant (MNA)

**Purpose:** Handoff for a new chat. Read this first, then the other project docs. Where this file and the older planning docs disagree, this file reflects what was actually built and decided later.

**Created:** 28 September 2026 (the MNA project folder dates from about 11 September 2026, per file timestamps)
**Last updated:** 28 September 2026, after recording, notes, sessions, history, settings, branding, navigation and Exit were built and verified by the user.

---

## 1. What this is

A Windows desktop app (Tauri v2 + React + TypeScript + Rust) that detects a Microsoft Teams meeting, asks the user for consent, records the meeting audio locally, lets them type notes, and (still to build) transcribes and generates an editable MOM via the OpenAI API. Local-first, no backend, no database, files only.

- **Product name (final):** "Meeting Note Assistant" (singular "Note", matching the logo). Older docs still say "Notes" — treat the singular as correct.
- **Repo:** github.com/QuantexEnt/mna (branch `main`)
- **Project folder:** `C:\Users\Omkar\MNA`
- **Env:** new Windows laptop, PowerShell. Rust, Node LTS, Git, VS Build Tools, WebView2, VS Code, Tauri CLI 2.11.x installed. Key crate versions: `windows` 0.58, `hound` 3.5, `chrono` 0.4, `serde`/`serde_json`.
- Claude works in a Linux sandbox: **it cannot compile or run any of this.** Every code drop is unverified until the user builds it. Never claim test results that the user has not reported.

---

## 2. Status at a glance

| Area | Status |
|---|---|
| Environment + scaffold + Git | Done |
| App shell: window, tray, close-to-tray, Exit | Done, verified |
| Teams meeting detection | Done, verified on one Teams build |
| Consent popup (Start / Not Now) | Done, verified |
| Audio recording (dual-source, gated mic) | Done, verified solo; NOT verified with a second participant in the real app |
| Auto-stop on meeting end; Exit finalizes recording | Done, verified |
| Session + local storage (per-meeting folders, meeting.json) | Done, verified |
| Manual notes with auto-save | Done, verified |
| Branding (logo, icon, palette, Inter font) | Done (logo missing in popup, cosmetic) |
| Navigation (Meeting / History / Settings) + header Exit | Done, verified |
| Settings (persisted) + disabled License placeholder | Done |
| History list | Done (list only, cannot open a meeting) |
| **Transcription (OpenAI)** | **Not started** |
| **MOM generation** | **Not started** |
| **Review screen (MOM / Transcript / Notes tabs)** | **Not started** (exists only as HTML mockup) |
| **Export (.md / .docx / .pdf), Copy** | **Not started** |
| License system backend | Deferred (UI placeholder only) |
| Packaging / installer | Not started |

---

## 3. Architecture as built

### Rust (`src-tauri/src/`)
- `lib.rs` — app setup, tray (Open / Exit), hide-on-close, registers commands, starts Teams detector, `exit_app` command. Both Exit paths call `audio::stop_recording_if_active()` first.
- `teams/mod.rs` — detector thread, polls every 3 s. Emits events `teams-meeting-detected` / `teams-meeting-ended`, brings the window to front on detection, calls auto-stop on meeting end.
- `audio/mod.rs` — `start_recording`, `stop_recording`, `stop_recording_if_active`. Owns the active-recording state and threads. Format probe runs on its own thread.
- `audio/loopback.rs` — WASAPI loopback of the default output device (captures other participants).
- `audio/mic_gated.rs` — captures the physical mic; writes real samples only while the gate is open, silence otherwise. Initialised to the render device's sample rate/channels.
- `audio/mute_watcher.rs` — UI Automation poller (250 ms) reading Teams' mute button; sets the shared gate flag.
- `audio/merge.rs` — offline sum of the two temp WAVs, peak-normalised to 0.95, deletes temp files.
- `session.rs` — creates the meeting folder + `meeting.json`, `finish_session`, `save_notes`, `list_meetings`.
- `settings.rs` — `load_settings` / `save_settings` to `settings.json`.

Registered Tauri commands: `start_recording`, `stop_recording`, `save_notes`, `list_meetings`, `load_settings`, `save_settings`, `exit_app`.

### Frontend (`src/`)
- `App.tsx` — header (logo left, Exit button right), sidebar nav, page switching. `MeetingNotification` is **always mounted** (outside the page switch) so detection works from any tab.
- `components/MeetingNotification.tsx` — state machine `idle → awaiting_decision → recording | dismissed`. Popup, red recording bar with timer, Stop & Save, and a "Meeting in progress" bar with a Start Taking Notes button in the `dismissed` state (lets the user start mid-call).
- `components/NotesEditor.tsx` — textarea, 1 s debounced auto-save to `notes/notes.md`.
- `components/SettingsPage.tsx`, `components/HistoryPage.tsx`.
- `src/assets/Horizontal-_MNA.png` (logo); global theme + Inter font are in `index.html`.
- `tauri.conf.json`: `frontendDist ../dist`, `devUrl http://localhost:1420`, `beforeDevCommand npm run dev`, `beforeBuildCommand npm run build`, `withGlobalTauri: true`.

### Storage layout (dev location: inside the project folder)
```
MNA/meetings/<YYYY-MM-DD_HHMMSS>_Meeting/
    meeting.json          (id, title "Meeting", started_at, ended_at, status ACTIVE|COMPLETED, audio path)
    audio/meeting.wav
    transcript/           (empty until transcription exists)
    notes/notes.md
    mom/                  (empty until MOM exists)
MNA/settings.json         (contains the API key)
```
`.gitignore` must contain: `meetings/`, `*.wav`, `settings.json`, `.env` (and `.env.*` except `.env.example`).

### Brand
Blue `#147DFF`, purple `#5137E8`, navy text `#12244A`, soft lavender `#F1F3FF`, page tint `#F8FAFF`, border `#DCE3F0`, muted text `#6F7F98`, danger `#C73636`. Font: Inter. The uploaded brand kit is a *marketing-website* system; only the colours, font and logo were adopted for the app.

---

## 4. Key decisions and discoveries (why things are the way they are)

1. **Teams detection heuristic** (no Microsoft API exists). A process whose name contains "teams" has 2 or more visible windows AND at least one window title that is not `<Chat|Calendar|Activity|Meet|People|Calls|Files|Apps|Teams|Notifications|Search> | Microsoft Teams`. A dedicated call process did NOT appear on this machine. It also fires on the pre-join lobby screen.
2. **Loopback cannot capture the user's own voice** (Teams never plays your mic back to your speakers). Process-level loopback has the same limit. Therefore the recorder is dual-source: loopback for others + gated physical mic for the user. Formalised as **Decision 013-A** (already added to `docs/DECISIONS.md`).
3. **Mute signal:** Teams' UI Automation button name. `"Mute mic"` = currently unmuted; `"Unmute mic"` = currently muted. Windows' OS-level mic mute does NOT reflect Teams' own mute, but Teams follows OS/laptop-key mute. **Fail closed:** the gate starts shut and stays shut unless "Mute mic" is positively read.
4. **Format mismatch bug:** default output ran at 44.1 kHz, mic at 48 kHz; merging refused. Fixed by initialising the mic capture to the render device's format.
5. **COM apartment bug (`0x80010106`):** any COM code called directly inside a Tauri command conflicts with Tauri's own thread. All COM work runs on its own spawned thread.
6. **Volume:** raw mic bypasses Teams' AGC, so output was quiet. Fixed with peak normalisation in `merge.rs`.
7. Closing the window hides to tray by design (Decision 037). Exit (header button or tray) quits fully and finalizes any recording first.
8. Speaker diarization, live transcription, calendar, Zoom, cloud backend: out of V1 scope per `DECISIONS.md`.

---

## 5. Known limitations and risks (be honest about these)

- **Other participants' audio has not been re-verified inside the real app.** It was verified only in the old standalone POC. Needs one real two-person test.
- **Bluetooth headset:** the Teams mute-button signal was flaky with the tested headset. That headset's hardware mute silences audio at the source, so nothing leaked, but other headsets may differ. Untested elsewhere.
- **Teams UI dependency:** detection and the mute gate depend on Teams' window titles and button names. A Teams update or a non-English Teams UI (untested) could break them. Re-verify after Teams updates.
- **Long recordings (unverified concern):** temp files are 48 kHz float stereo (roughly 1.4 GB per hour per file) and `merge.rs` loads both fully into memory. Likely a problem for long meetings. Also far too large to upload to OpenAI as-is, so the transcription phase needs downmix to mono, resample (about 16 kHz), compress and chunk.
- API key is stored as plain JSON in `settings.json`. Acceptable for personal use only; must move to Windows Credential Manager before any distribution.
- Recordings live inside the project folder (dev convenience). Move to a proper user-data location before real use.
- Not tested: sleep/resume, audio device switching mid-call, crash recovery, app launched mid-call, two overlapping calls.
- Mic gate lag is up to about 250 ms at mute transitions (documented in Decision 013-A).
- `meeting.json` `duration_seconds` is always null. Title is always "Meeting".
- Cosmetic: logo missing on the popup; `package.json` name is still `temp-scaffold`; harmless compiler warning about unused `Interface` import in `mute_watcher.rs`; the shutdown log line `Failed to unregister class Chrome_WidgetWin_0. Error = 1412` is a harmless WebView2 message.

---

## 6. Remaining work (suggested order)

1. **Settings groundwork for AI:** confirm a real OpenAI *API* key with billing exists (a ChatGPT subscription is not an API key, Decision 025). Verify current model names against OpenAI docs; the model IDs in the docs and Settings defaults are placeholders.
2. **Transcription (Phase 9):**
   - `TranscriptionService` abstraction; **mock mode first** (fake transcript, no key needed); then the real API.
   - Audio prep: mono, about 16 kHz, compress, chunk to API limits.
   - Status states, retry with limits, errors that never delete audio/notes.
   - Save `transcript/transcript.json` and `transcript.md`. Post-meeting only.
3. **MOM generation:** `MomGenerationService` (mock first), structured JSON output, anti-hallucination prompt rules (never invent owners, dates, decisions; suggestions are not commitments; unknown = "Not specified"), save `mom_generated.md` and `mom_final.md`. Notes-only and transcript-only paths.
4. **Review screen:** MOM / Transcript / My Notes tabs, editable MOM, Save, Copy, Regenerate with an overwrite warning. Processing screen with step statuses.
5. **History:** open a meeting to view notes/transcript/MOM; delete meeting with confirmation.
6. **Export:** Markdown and Copy are firmly in scope. **DOCX/PDF: pending decision** (docs call them optional).
7. **Settings completion:** working Test Connection, real Start-with-Windows registration, configurable storage location, secure API key storage.
8. **Recovery:** detect unfinished sessions on launch, clean leftover temp WAVs.
9. **Validation:** two-person test in the real app, headset tests, long-meeting test (memory and file size), Teams-update re-check.
10. **Packaging:** installer, uninstall keeps user data, release checks from `TESTING_AND_ACCEPTANCE.md`.
11. **Docs sync:** update README/other docs to the singular name and to what was actually built; keep `DECISIONS.md` current.
12. **Cosmetic:** logo on popup, `package.json` name, unused import.

---

## 7. Future commercial idea (NOT V1 — scope change to record when it starts)

The user plans to sell licenses, with each customer using their own OpenAI key. `DECISIONS.md` #046 currently rules out licensing for V1, so this needs a formal amendment when work begins. Settings already shows a disabled License Key field, a "Not Activated" badge and an Activate button.

Planned behaviour: server-validated key bound to one device and an email; activating on a new device prompts "move license here?", updates the DB, and the old device loses access. Design notes already given to the user:
- Use Windows `MachineGuid` (HKLM\SOFTWARE\Microsoft\Cryptography) as the device ID, **not MAC address** (multiple adapters, Wi-Fi randomisation, easily spoofed).
- Needs a separate backend project (API + DB), which conflicts with the "no backend" V1 decision, so it stays a separate product decision.
- Old-device cut-off only works if the app re-validates periodically. Decide: check on launch only or also while running, and offline grace-period behaviour.
- Add an Email field next to License Key. Treat stored emails and device IDs as personal data.
- Never embed a shared API key in the distributed app.

---

## 8. Pending user decisions

1. DOCX and PDF export: required for V1, or nice-to-have?
2. Confirm the OpenAI API key and billing are set up.
3. Whether "You vs Others" speaker labels are wanted later. Note this needs the two audio sources kept as separate tracks; today they are merged into one file.
4. A consent/legal notice wording for recording (SECURITY_AND_PRIVACY.md §39).

---

## 9. How to work with the user (lessons from this session)

- Give files with exact target paths. Downloads arrive as `*.txt`; the user renames them. Several past errors were wrong location, wrong extension (`.rb` instead of `.rs`) or a file left as `.txt`. Ask them to check `dir` output when a module or import is "not found".
- Ask for the **Rust terminal output** for backend problems and the **browser devtools console (F12)** for frontend problems. The terminal cannot show JavaScript errors.
- Do one phase at a time, test it, then continue (DECISIONS.md #056). Ask before any decision that changes scope.
- PowerShell may block `npm`; workaround `npm.cmd`, or `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`.
- A stale exe icon in the taskbar is Windows icon caching; fix with a full quit, and `cargo clean` if needed.
- `cargo tauri dev` keeps running until the app is exited; JS/TS changes hot-reload, Rust changes rebuild.
- `windows` 0.58 notes: `use windows::core::Interface` is required for `.cast()`; `WAVE_FORMAT_IEEE_FLOAT` is not exported from `Media::Audio` (hardcode 3); tray uses `show_menu_on_left_click`.
- The two standalone tools in `Downloads` (`audio_capture_poc`, `mute_detection_poc`) were disposable experiments. They never merge into MNA and can be deleted. `mute_detection_poc` includes a raw-mic diagnostic recorder; delete any recordings it made.
- Do not generate fake test results, and do not promise API keys or credits.

---

## 10. Before closing this chat (checklist)

- [ ] Commit and push all current code: `git add .` then `git commit -m "feat: navigation, settings, history, exit, recording lifecycle"` then `git push`
- [ ] Confirm `.gitignore` contains `meetings/`, `*.wav`, `settings.json` and `.env` (an API key or a real recording must never reach GitHub)
- [ ] Save this file as `MNA\docs\PROJECT_STATUS.md` and commit it
- [ ] Add this file to the Claude project's files so new chats can read it
- [ ] Delete any leftover test recordings you don't need (real voices)

## 11. Suggested first message for the new chat

> Read PROJECT_STATUS.md first, then DECISIONS.md. We are continuing the Meeting Note Assistant build. Next step is Transcription, mock mode first (section 6, item 2). Give me exact file paths for every file, and ask me for the terminal output or the F12 console output when something fails.

## 12. Update log (append a new dated entry at the end of every session)

| Date | Change |
|---|---|
| 28 Sep 2026 | File created. Records everything built through Exit-button and recording-finalize-on-exit. Transcription, MOM, Review screen and Export not started. |
