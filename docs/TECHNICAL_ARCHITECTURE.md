# Technical Architecture

## Meeting Notes Assistant — V1

**Document Status:** Approved for V1 development  
**Platform:** Windows 10/11  
**Primary meeting platform:** Microsoft Teams  
**Deployment:** Local desktop application  
**Backend:** None required  
**Database:** None required for initial V1  
**AI:** OpenAI API

---

# 1. Architecture Objective

The application must be a lightweight Windows desktop utility that operates independently of Microsoft Teams.

Its architecture must support this workflow:

```text
Teams Meeting
      ↓
Detect Meeting
      ↓
User Confirmation
      ↓
Capture Teams Audio
      ↓
Transcription
      ↓
Manual Notes
      ↓
Transcript + Notes
      ↓
OpenAI
      ↓
MOM
      ↓
Local Storage
```

The architecture should prioritize:

1. Reliability.
2. Simplicity.
3. Local operation.
4. Low resource usage.
5. Clear separation between native Windows functionality, UI, AI processing, and local storage.
6. Easy future modification without unnecessary infrastructure.

---

# 2. Architecture Principles

## 2.1 Local-first

The application should operate locally wherever practical.

There is no requirement for:

- Application server.
- Cloud database.
- Cloud storage.
- User account.
- Cloud synchronization.

OpenAI is an external service used only for AI processing.

---

## 2.2 Desktop-native capabilities stay in Rust

Operations requiring Windows APIs or low-level system access should be implemented in the Tauri/Rust layer rather than in browser-based frontend code.

Examples:

- Process detection.
- Windows notifications where appropriate.
- Audio capture.
- File system operations.
- Application lifecycle.
- System tray.
- Secure local configuration handling.

---

## 2.3 UI stays in React

React/TypeScript should handle:

- Screens.
- Forms.
- Meeting workspace.
- Transcript display.
- Notes editor.
- MOM editor.
- Settings.
- Meeting history.
- User interaction.

The UI should not directly implement low-level Windows functionality.

---

## 2.4 AI access is isolated

OpenAI integration should be separated from UI components.

The frontend should not contain scattered API calls.

A dedicated service/module should manage:

- Transcription requests.
- MOM generation.
- Authentication.
- Error handling.
- Retry behavior.
- Response parsing.

---

# 3. Recommended Technology Stack

## Desktop Framework

**Tauri**

Purpose:

- Windows application shell.
- Connect React UI to Rust native functionality.
- Package application as a Windows executable.

---

## Frontend

**React + TypeScript**

Purpose:

- Application interface.
- Meeting workspace.
- Notes.
- Transcript.
- MOM editor.
- Settings.

---

## Native Layer

**Rust**

Purpose:

- Windows integration.
- Teams process detection.
- Audio capture.
- Local file operations.
- Application lifecycle.
- System tray/background operation.

---

## Operating System APIs

Use Windows-native APIs where required.

Primary areas:

- Windows process/window detection.
- Windows notifications.
- Windows audio APIs.
- WASAPI.
- WASAPI Process Loopback Capture where technically appropriate.
- Windows application lifecycle APIs.

---

## AI

**OpenAI API**

Used for:

1. Speech-to-text/transcription.
2. MOM generation.

One API key is sufficient for both purposes.

Different models/capabilities may be used for the two jobs.

---

## Local Persistence

Initial V1 should use:

- Normal local folders.
- Audio files.
- Text/Markdown files.
- JSON metadata.

Do not introduce SQLite unless a later requirement makes structured database storage useful.

---

## Source Control

**Git**

Repository may be hosted on **GitHub**.

---

# 4. High-Level Component Architecture

```text
+-------------------------------------------------------------+
|                    WINDOWS DESKTOP                          |
|                                                             |
|  +-------------------------------------------------------+  |
|  |                 TAURI APPLICATION                     |  |
|  |                                                       |  |
|  |  +----------------------+    +----------------------+ |  |
|  |  | React / TypeScript   |    | Rust Native Layer   | |  |
|  |  |                      |    |                      | |  |
|  |  | Meeting UI           |◄──►| Teams Detection     | |  |
|  |  | Transcript           |    | Audio Capture       | |  |
|  |  | Notes                |    | File Operations     | |  |
|  |  | MOM Editor           |    | System Tray        | |  |
|  |  | Settings             |    | Windows APIs       | |  |
|  |  +----------------------+    +----------+-----------+ |  |
|  |                                          |             |  |
|  |                                          ▼             |  |
|  |                                Teams Meeting Audio     |  |
|  |                                                       |  |
|  +----------------------------+--------------------------+  |
|                               |                             |
|                               ▼                             |
|                        Local Audio File                    |
|                               |                             |
|                               ▼                             |
|                       OpenAI Transcription                 |
|                               |                             |
|                               ▼                             |
|                           Transcript                       |
|                               |                             |
|                    +----------+----------+                  |
|                    |                     |                  |
|                    ▼                     ▼                  |
|              Personal Notes        Transcript               |
|                    |                     |                  |
|                    +----------+----------+                  |
|                               ▼                             |
|                       OpenAI MOM Generation                 |
|                               |                             |
|                               ▼                             |
|                           Local MOM                         |
|                                                             |
+-------------------------------------------------------------+
```

---

# 5. Frontend Architecture

Recommended React structure:

```text
src/
├── components/
│   ├── MeetingNotification/
│   ├── MeetingWorkspace/
│   ├── TranscriptPanel/
│   ├── NotesPanel/
│   ├── MomEditor/
│   ├── MeetingHistory/
│   └── Settings/
│
├── pages/
│   ├── Home/
│   ├── Meeting/
│   ├── History/
│   └── Settings/
│
├── services/
│   ├── api/
│   ├── meetings/
│   ├── transcription/
│   └── mom/
│
├── state/
│   └── meetingStore/
│
├── types/
│   └── meeting.ts
│
└── utils/
```

The exact structure can be adjusted during implementation, but responsibilities should remain separated.

---

# 6. Rust Architecture

Recommended conceptual structure:

```text
src-tauri/
├── src/
│   ├── main.rs
│   ├── commands/
│   ├── teams/
│   ├── audio/
│   ├── storage/
│   ├── notifications/
│   ├── settings/
│   └── models/
│
└── tauri.conf.json
```

The exact module structure can evolve.

---

# 7. Frontend ↔ Rust Communication

Tauri commands/events should be used to communicate between React and Rust.

Example:

```text
React
  │
  │ start_note_taking()
  ▼
Tauri Command
  │
  ▼
Rust
  │
  ├── Start audio capture
  └── Start session
```

For status updates:

```text
Rust
  │
  │ MeetingDetected
  ▼
Tauri Event
  │
  ▼
React
  │
  ▼
Show notification
```

Use typed payloads where practical.

Avoid tightly coupling UI components directly to native implementation details.

---

# 8. Teams Detection Architecture

The application does not need to use the Microsoft Teams API for the initial workflow.

Instead, the native Windows layer should identify whether Teams is running and whether it appears to be in an active meeting/call state.

The implementation should be isolated behind an interface such as:

```text
TeamsMeetingDetector
```

Conceptually:

```text
TeamsMeetingDetector
        │
        ├── isTeamsRunning()
        │
        ├── isMeetingActive()
        │
        └── getMeetingState()
```

The exact detection signals should be verified during implementation.

Do not assume a single process/window condition is always sufficient.

The implementation should be tested against the currently installed Microsoft Teams client.

---

# 9. Audio Capture Architecture

Audio capture is a core native component.

The preferred direction is Windows WASAPI-based capture, with Process Loopback Capture used where it can reliably isolate the Teams process.

Conceptually:

```text
Microsoft Teams
       │
       ▼
Windows Audio Engine
       │
       ▼
Teams process audio
       │
       ▼
WASAPI / Process Loopback
       │
       ▼
Audio Capture Module
       │
       ▼
Local audio chunks/file
```

The audio module should expose a simple interface to the rest of the application.

Example conceptual interface:

```text
AudioCapture
├── start()
├── stop()
├── pause()
├── resume()
├── get_status()
└── finalize()
```

The exact Rust API should be determined after testing Windows audio capture.

Detailed requirements are in `AUDIO_CAPTURE.md`.

---

# 10. Important Audio Requirement

The architecture must distinguish:

### Meeting audio

Audio actually being output/transmitted through the Teams meeting.

### Local microphone audio

Speech that may exist locally but is not necessarily being transmitted into Teams.

The desired behavior is:

```text
User unmuted + speaking
        ↓
Voice enters Teams
        ↓
Meeting audio
        ↓
Captured
```

Whereas:

```text
User muted + speaking
        ↓
Voice remains local
        ↓
Should not become meeting audio
```

The application should therefore avoid simply recording the physical microphone as a second independent source unless a future requirement explicitly asks for it.

---

# 11. Audio Pipeline

The recommended pipeline is:

```text
Teams Audio
    ↓
Capture
    ↓
Buffer
    ↓
Audio normalization if required
    ↓
Chunk/file storage
    ↓
Transcription
```

The implementation should avoid holding an entire long meeting in memory.

For long meetings, audio should be written incrementally.

---

# 12. Transcription Architecture

The transcription layer should be independent from the audio-capture implementation.

```text
AudioCapture
      ↓
AudioFile / AudioChunk
      ↓
TranscriptionService
      ↓
TranscriptSegment[]
```

Conceptual data structure:

```text
TranscriptSegment
├── id
├── start_time
├── end_time
├── text
└── speaker (optional)
```

Speaker identification should not be treated as mandatory for the first implementation unless it can be achieved reliably with the selected transcription approach.

---

# 13. Transcription Modes

The architecture should support two possible modes:

### Mode A — Near-live transcription

Audio is processed in chunks during the meeting.

```text
Audio chunk
    ↓
OpenAI
    ↓
Transcript segment
```

Advantages:

- User can see transcript during the meeting.

Challenges:

- More API calls.
- More complexity.
- Handling overlapping/duplicate text.
- Network dependency during meeting.

### Mode B — Post-meeting transcription

The complete recording is processed after the meeting.

```text
Meeting recording
      ↓
End meeting
      ↓
OpenAI transcription
      ↓
Complete transcript
```

Advantages:

- Simpler.
- More reliable for V1.
- Easier to reconstruct a clean transcript.

Challenges:

- Transcript is not available during the meeting.

The implementation should initially favor the simplest reliable approach unless live transcription is a firm V1 requirement.

---

# 14. Manual Notes Architecture

Manual notes are managed by the React UI and persisted locally.

Conceptually:

```text
NotesEditor
    ↓
MeetingSession.notes
    ↓
LocalStorageService
```

Notes should be saved incrementally.

Do not wait until the user closes the meeting to write all notes to disk.

---

# 15. MOM Generation Architecture

MOM generation should be implemented as a separate service:

```text
MomGenerationService
        │
        ├── transcript
        ├── personal notes
        └── meeting metadata
                 │
                 ▼
             OpenAI API
                 │
                 ▼
              MOM text
```

The prompt/template should be stored separately from UI components.

The service should validate the returned result before presenting it to the user.

---

# 16. MOM Data Flow

```text
Meeting Metadata
      +
Transcript
      +
Personal Notes
      ↓
MOM Prompt Builder
      ↓
OpenAI Text Model
      ↓
Generated MOM
      ↓
MOM Editor
      ↓
User edits
      ↓
Local save
```

---

# 17. Local Storage Architecture

The initial application should use a normal folder structure.

Recommended:

```text
MeetingNotesAssistant/
│
├── meetings/
│   ├── 2026-09-11_Project_Review/
│   │   ├── meeting.json
│   │   ├── recording.wav
│   │   ├── transcript.md
│   │   ├── notes.md
│   │   └── MOM.md
│   │
│   └── 2026-09-12_Team_Meeting/
│       ├── meeting.json
│       ├── recording.wav
│       ├── transcript.md
│       ├── notes.md
│       └── MOM.md
│
└── settings/
```

The final file structure is defined in `LOCAL_STORAGE_SPECIFICATION.md`.

---

# 18. Meeting Session Model

A conceptual meeting session should contain:

```text
MeetingSession
├── id
├── title
├── start_time
├── end_time
├── status
├── teams_detected
├── recording_path
├── transcript_path
├── notes_path
├── mom_path
└── processing_status
```

Optional metadata should not be required for the application to work.

---

# 19. Application State Management

The frontend should maintain a single authoritative state for the active meeting session.

Conceptual states:

```text
idle
meeting_detected
awaiting_confirmation
recording
processing
mom_ready
error
```

The UI should derive its display from application state rather than maintaining independent conflicting states in multiple components.

---

# 20. Event Flow

Important events include:

```text
TEAMS_MEETING_DETECTED
TEAMS_MEETING_ENDED
NOTE_SESSION_STARTED
AUDIO_CAPTURE_STARTED
AUDIO_CAPTURE_STOPPED
TRANSCRIPTION_STARTED
TRANSCRIPTION_PROGRESS
TRANSCRIPTION_COMPLETED
MOM_GENERATION_STARTED
MOM_GENERATION_COMPLETED
SESSION_SAVED
ERROR_OCCURRED
```

The exact event names may be changed during implementation, but the responsibilities should remain clear.

---

# 21. Error Handling Architecture

Errors should be handled at the service boundary.

Example:

```text
Audio Capture
     ↓
Error
     ↓
Native layer catches error
     ↓
Structured error event
     ↓
React UI
     ↓
User-friendly message
```

Do not expose raw stack traces or technical implementation details to normal users.

Technical details should be available in developer logs when needed.

---

# 22. OpenAI API Architecture

The application should have a dedicated OpenAI client/service.

Conceptually:

```text
OpenAIService
├── transcribe_audio()
├── generate_mom()
└── handle_api_error()
```

API configuration should not be scattered across the application.

The API key should be supplied through secure local configuration rather than committed to source control.

The application should not include a hard-coded personal API key in the repository.

---

# 23. API Processing Flow

### Transcription

```text
Local recording
      ↓
OpenAI transcription request
      ↓
Transcript response
      ↓
Local transcript file
```

### MOM

```text
Transcript
+
Personal notes
      ↓
OpenAI text request
      ↓
MOM response
      ↓
Local MOM file
```

No application server is required between the desktop application and OpenAI for V1.

---

# 24. Security Boundary

The main security boundaries are:

```text
Windows User
     │
     ▼
Desktop Application
     │
     ├── Local Meeting Data
     │
     └── OpenAI API
```

The application should minimize the amount of data sent to OpenAI.

For MOM generation, unrelated local meeting files must never be included.

---

# 25. Background Process Architecture

The application should have a lightweight background mode.

The system tray/background component should:

- Keep Teams detection active.
- Receive meeting state changes.
- Show meeting notifications.
- Allow the user to open the application.
- Allow the user to exit the application.

When no meeting is active, there should be no recording or transcription activity.

---

# 26. Startup Behavior

The application should support optional Windows startup.

Recommended setting:

> **Start Meeting Notes Assistant with Windows**

If enabled, the application starts in background/system-tray mode.

If disabled, the user starts the application manually.

---

# 27. Resource Management

The application should avoid unnecessary resource consumption.

### Idle

- No audio capture.
- No OpenAI requests.
- Low CPU usage.
- Low memory usage.

### Active meeting

- Audio capture active.
- Transcription processing active when configured.
- Notes persistence active.

### After meeting

- Audio capture stopped.
- Transcription completed.
- AI request performed only when requested.
- Resources released.

---

# 28. Threading / Async Behavior

Long-running operations must not block the UI.

Examples:

- Audio capture.
- File writing.
- Transcription.
- OpenAI API calls.
- MOM generation.

Use appropriate asynchronous/background execution.

The React interface must remain responsive while these operations run.

---

# 29. Logging

The application should have lightweight application logging.

Logs may contain:

- Application startup.
- Teams detection state changes.
- Recording start/stop.
- Transcription status.
- MOM generation status.
- Errors.

Logs must not contain:

- API keys.
- Full meeting transcripts by default.
- Sensitive meeting content unnecessarily.

---

# 30. Configuration

Configuration should be centralized.

Possible configuration:

```text
OpenAI API key
Transcription model
MOM model
Meeting storage location
Start with Windows
Keep recordings after transcription
```

Only settings that are actually required should be exposed in V1.

---

# 31. Installation Architecture

The final application should be packaged as a Windows installer.

The installer should:

- Install the desktop application.
- Create necessary application folders.
- Register required application configuration.
- Provide uninstall support.

The installer should not require a server.

---

# 32. Development Architecture

The repository should be structured so that frontend and native code can be developed together.

Conceptually:

```text
meeting-notes-assistant/
│
├── src/                  # React/TypeScript
├── src-tauri/            # Rust/Tauri
├── docs/                 # Project specifications
├── public/
├── package.json
├── Cargo.toml
├── tauri.conf.json
└── README.md
```

---

# 33. Dependency Principle

Every dependency added to the project must have a clear reason.

Before adding a library, ask:

1. Does the standard library or existing framework already solve this?
2. Is the dependency maintained?
3. Does it increase security risk?
4. Does it increase application size?
5. Does it introduce unnecessary complexity?
6. Is it necessary for V1?

Prefer fewer dependencies.

---

# 34. Architecture Decision: No Cloud Backend

V1 intentionally does not include:

- Node.js backend server.
- Python backend server.
- Hostinger server.
- MySQL.
- PostgreSQL.
- Firebase.
- Supabase.
- AWS.
- Azure.
- Other cloud infrastructure.

The desktop application communicates directly with OpenAI when AI processing is required.

---

# 35. Architecture Decision: No Database Initially

The application should use local files and JSON metadata initially.

A database may be introduced later if actual requirements demonstrate a need for:

- Large meeting libraries.
- Complex search.
- Structured filtering.
- Large-scale metadata.
- Multi-user functionality.

Do not introduce SQLite merely because it is available.

---

# 36. Architecture Decision: No Teams Bot

The application must not:

- Join meetings.
- Appear as a meeting participant.
- Use a bot identity.
- Require a Teams app installation.
- Depend on Microsoft Graph meeting APIs for the core recording workflow.

The desired architecture is an independent desktop application.

---

# 37. Critical Technical Risks

The following areas require early proof-of-concept testing:

## Risk 1 — Teams meeting detection

Microsoft Teams behavior can vary by client version.

Detection must be tested against the currently installed Teams application.

## Risk 2 — Process-specific audio capture

Windows audio routing can vary depending on:

- Speakers.
- Headphones.
- Bluetooth devices.
- Teams version.
- Windows audio configuration.

Process Loopback Capture must be tested thoroughly.

## Risk 3 — User's own transmitted voice

The requirement is to capture the user's voice only when it becomes part of the Teams meeting audio.

This behavior must be verified experimentally.

## Risk 4 — Long meetings

The application must not keep large audio recordings entirely in RAM.

## Risk 5 — API/network failure

The application must preserve local data and permit retry.

---

# 38. Recommended Implementation Order

Do not begin by building the complete UI.

Build and validate the technically risky components first:

```text
1. Tauri Windows shell
        ↓
2. Teams detection proof of concept
        ↓
3. Teams process audio capture proof of concept
        ↓
4. Save captured audio locally
        ↓
5. Verify audio quality
        ↓
6. OpenAI transcription proof of concept
        ↓
7. Local transcript storage
        ↓
8. Manual notes UI
        ↓
9. MOM generation
        ↓
10. Complete meeting workflow
        ↓
11. UI refinement
        ↓
12. Installer
```

This reduces the risk of spending time on UI before confirming the core technical requirement.

---

# 39. Definition of Done for Architecture

The architecture is considered validated when the prototype can demonstrate:

1. Teams meeting is detected.
2. User is prompted.
3. User starts note taking.
4. Teams meeting audio is captured.
5. Captured audio contains expected meeting voices.
6. Muted local speech is not independently captured.
7. Audio is saved locally.
8. Audio can be transcribed through OpenAI.
9. User notes can be saved locally.
10. Transcript and notes can be passed to MOM generation.
11. MOM is returned and editable.
12. All meeting artifacts remain locally accessible.
13. No cloud backend is required.
14. The application remains usable while background processing occurs.

---

# 40. Architecture Boundary

The application architecture ends at:

```text
Teams
  ↓
Audio
  ↓
Transcript
  +
Personal Notes
  ↓
MOM
  ↓
Local Files
```

Any future capability outside this flow should be evaluated separately and should not be introduced into V1 automatically.
