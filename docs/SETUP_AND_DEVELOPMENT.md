# Setup and Development Guide

## Meeting Notes Assistant — V1

**Platform:** Windows 10/11  
**Application:** Tauri + React + TypeScript + Rust  
**Primary IDE:** VS Code  
**Source control:** Git + GitHub  
**AI:** OpenAI API

---

# 1. Purpose

This document explains how to prepare a Windows development machine, create the project, configure required tools, run the application, and begin implementation.

The goal is to keep development simple and reproducible.

---

# 2. Development Philosophy

The project should be developed in small, testable stages.

Recommended sequence:

```text
Environment
    ↓
Tauri shell
    ↓
Basic React UI
    ↓
Teams detection
    ↓
Audio capture proof of concept
    ↓
Local storage
    ↓
OpenAI transcription
    ↓
Notes
    ↓
MOM generation
    ↓
History / settings
    ↓
Packaging
```

Do not build every feature at once.

---

# 3. Required Operating System

Recommended:

**Windows 11**

Supported target:

- Windows 10
- Windows 11

The application should be tested on the actual Windows versions intended for V1.

---

# 4. Required Development Tools

Install:

- Git
- Node.js
- npm
- Rust
- Tauri prerequisites
- VS Code
- WebView2
- Windows SDK/build tools

Optional:

- Visual Studio Community
- Windows Terminal
- PowerShell 7

---

# 5. Git

Install Git for Windows.

Verify:

```powershell
git --version
```

Expected:

```text
git version ...
```

Configure identity:

```powershell
git config --global user.name "Your Name"
git config --global user.email "your@email.com"
```

Use your own Git identity.

---

# 6. Node.js

Install a current supported Node.js LTS release.

Verify:

```powershell
node --version
npm --version
```

The project should use a current LTS version rather than an experimental release.

---

# 7. Rust

Install Rust using the official Rust toolchain installer.

Verify:

```powershell
rustc --version
cargo --version
```

The Tauri application uses Rust for native Windows functionality.

---

# 8. Tauri Prerequisites

Tauri requires the appropriate Windows development environment.

Install the required:

- Microsoft C++ build tools.
- Windows SDK.
- WebView2 runtime/development support.

Verify the environment using the Tauri documentation/current setup guidance before creating the project.

Do not assume an existing Visual Studio installation has every required component.

---

# 9. VS Code

VS Code is recommended for development.

Useful extensions:

- Rust Analyzer.
- ESLint.
- Prettier.
- TypeScript/JavaScript support.

Avoid installing large numbers of extensions without a reason.

---

# 10. Project Creation

The project should be created using the current official Tauri project scaffolding.

Conceptually:

```text
MeetingNotesAssistant
        │
        ├── React frontend
        │
        └── Tauri/Rust backend
```

The exact CLI command should be taken from the current Tauri documentation because scaffolding commands can change between releases.

---

# 11. Frontend

Recommended:

```text
React
TypeScript
```

The frontend handles:

- UI.
- Meeting workspace.
- Transcript display.
- Notes editor.
- MOM editor.
- Settings.
- History.

It should not directly implement Windows audio APIs.

---

# 12. Native Layer

Rust/Tauri handles:

- Windows process detection.
- Teams detection.
- Audio capture.
- Local filesystem operations where appropriate.
- System tray.
- Native events.
- Native application lifecycle.

---

# 13. Development Architecture

Conceptually:

```text
React / TypeScript
       │
       │ Tauri commands/events
       ▼
Rust / Tauri
       │
       ├── Teams Detection
       ├── Audio Capture
       ├── Local Storage
       └── Application State

React / Rust
       │
       ▼
OpenAI Service
       │
       ├── Transcription
       └── MOM Generation
```

Keep these boundaries clear.

---

# 14. Initial Project Structure

Recommended starting structure:

```text
MeetingNotesAssistant/
│
├── src/
│   ├── components/
│   ├── pages/
│   ├── services/
│   ├── state/
│   ├── types/
│   └── App.tsx
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   ├── teams/
│   │   ├── audio/
│   │   ├── storage/
│   │   └── state/
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── public/
│
├── .env.example
├── .gitignore
├── package.json
└── README.md
```

The structure can evolve as implementation progresses.

---

# 15. Git Repository

Initialize:

```powershell
git init
```

Create a GitHub repository.

Recommended repository name:

```text
meeting-notes-assistant
```

Then connect the local repository to GitHub.

Do not commit secrets or real meeting data.

---

# 16. `.gitignore`

The repository should ignore at minimum:

```text
node_modules/
target/
dist/
.env
.env.*
!.env.example
*.log
meeting data directories
temporary audio files
```

Do not accidentally commit real:

```text
.wav
.mp3
transcript files
MOM files
```

from personal meetings.

---

# 17. Environment Configuration

Development may use:

```text
.env
```

Example:

```text
OPENAI_API_KEY=your_api_key_here
```

The real file must remain local.

Commit:

```text
.env.example
```

instead.

---

# 18. OpenAI API Setup

The application needs one OpenAI API key.

The key can be used for:

```text
Transcription
+
MOM generation
```

The ChatGPT subscription and API billing are separate.

The developer should verify API access before testing AI features.

---

# 19. API Model Configuration

Do not hard-code model IDs throughout the application.

Use configuration:

```text
transcription_model
mom_model
```

Example:

```json
{
  "transcription_model": "configured-transcription-model",
  "mom_model": "configured-generation-model"
}
```

Model availability should be verified against the current OpenAI API documentation when implementation begins.

---

# 20. Running the Application

The development workflow should support:

```text
Start development server
        ↓
React frontend
        +
Tauri native process
        ↓
Windows desktop window
```

Use the current Tauri development command generated by the project scaffold.

Do not copy commands from old tutorials if they conflict with the installed Tauri version.

---

# 21. First Development Milestone

Before implementing Teams functionality, confirm:

```text
Application launches
        ↓
Window appears
        ↓
React UI renders
        ↓
Tauri/Rust backend runs
        ↓
Frontend can call a test Rust command
```

Example test command:

```text
get_app_version()
```

This proves the frontend/native communication works.

---

# 22. Second Milestone — System Tray

Implement:

- Tray icon.
- Open.
- Settings.
- Exit.

Confirm that:

```text
Minimize
    ↓
Application remains in tray
```

The application should continue running in the background.

---

# 23. Third Milestone — Teams Detection

Implement Teams detection independently.

Test:

```text
Teams closed
Teams open, no meeting
Teams meeting active
Teams meeting ended
```

The detector should expose a simple state to the application.

Example:

```text
teams_meeting_active = true
```

Do not connect audio capture yet.

---

# 24. Fourth Milestone — Detection Popup

When Teams meeting becomes active:

```text
Teams meeting detected

Would you like to take notes?

[ Start Taking Notes ] [ Not Now ]
```

Test that:

- Start opens a session.
- Not Now does not start capture.
- Repeated prompts are suppressed appropriately.

---

# 25. Fifth Milestone — Audio Proof of Concept

This is the most important technical milestone.

Before integrating the complete application:

```text
Teams
 ↓
Process identification
 ↓
WASAPI / Process Loopback
 ↓
Audio file
```

Test the actual captured audio.

Do not proceed based only on compilation.

---

# 26. Audio Proof-of-Concept Test

The prototype must prove:

### Test A

Another participant speaks.

Expected:

```text
Captured
```

### Test B

User speaks while unmuted.

Expected:

```text
Captured
```

### Test C

User speaks while Teams is muted.

Expected:

```text
Not independently captured
```

### Test D

User uses headphones.

Expected:

```text
Meeting audio still captured
```

This is a technical gate for the project.

---

# 27. Sixth Milestone — Local Storage

Once audio works:

Implement:

```text
Meeting folder
meeting.json
audio
notes
```

Verify that data survives application restart.

---

# 28. Seventh Milestone — Manual Notes

Add the notes panel.

Test:

```text
Type
 ↓
Auto-save
 ↓
Close application
 ↓
Reopen
 ↓
Notes preserved
```

---

# 29. Eighth Milestone — Transcription

Connect:

```text
Saved audio
     ↓
OpenAI transcription
     ↓
Transcript
     ↓
Local storage
```

First test with a short 2–5 minute recording.

Do not start with a two-hour recording.

---

# 30. Ninth Milestone — MOM Generation

Connect:

```text
Transcript
+
Manual Notes
     ↓
OpenAI generation
     ↓
Structured MOM
     ↓
Editable UI
```

Test with synthetic meeting data first.

Then test with a real meeting.

---

# 31. Tenth Milestone — Review

Implement:

```text
MOM
Transcript
My Notes
```

as review sections/tabs.

Confirm:

- MOM is editable.
- Save works.
- Copy works.
- Regeneration does not silently destroy edits.

---

# 32. Eleventh Milestone — History

Implement simple local meeting history.

Start with:

```text
Read meetings/
     ↓
Read meeting.json
     ↓
Display list
```

Do not add SQLite just because history exists.

---

# 33. Twelfth Milestone — Settings

Add:

- OpenAI API key configuration.
- Test connection.
- Model configuration.
- Storage location.
- Start with Windows.
- Meeting notification setting.

Keep settings minimal.

---

# 34. Development Sequence Summary

The recommended build order is:

```text
1. Environment
2. Tauri shell
3. React UI
4. Rust ↔ React communication
5. System tray
6. Teams detection
7. Detection popup
8. Audio POC
9. Local storage
10. Manual notes
11. Transcription
12. MOM generation
13. Review/edit
14. History
15. Settings
16. Recovery/error handling
17. Packaging
18. Final testing
```

---

# 35. Development Rule

Do not implement future features while a P0 feature is still unproven.

Example:

Do not spend time building:

```text
Beautiful MOM editor
```

if:

```text
Teams audio capture
```

has not been proven reliable.

---

# 36. Claude Development Workflow

The project is intended to be vibe-coded with Claude.

Claude should be given the project documentation before major implementation work.

Recommended order:

```text
README.md
PRODUCT_REQUIREMENTS.md
USER_FLOW.md
TECHNICAL_ARCHITECTURE.md
AUDIO_CAPTURE.md
OPENAI_INTEGRATION.md
TRANSCRIPTION_SPECIFICATION.md
NOTES_AND_MOM_SPECIFICATION.md
UI_UX_SPECIFICATION.md
LOCAL_STORAGE_SPECIFICATION.md
SECURITY_AND_PRIVACY.md
```

Then use the remaining project instructions and implementation plan.

---

# 37. Claude Task Style

Give Claude small implementation tasks.

Good:

> Implement Teams process detection according to the existing architecture. Do not modify unrelated files.

Bad:

> Build the entire application.

Small tasks make debugging and review much easier.

---

# 38. Before Every Major Change

Claude should:

1. Read the relevant specification.
2. Inspect existing code.
3. Identify affected files.
4. Explain the intended change briefly.
5. Implement the smallest appropriate change.
6. Run tests/build.
7. Report what changed.
8. Report any unresolved issue.

---

# 39. Scope Control

Claude must not add features simply because they seem useful.

If a requested feature is outside V1:

```text
Identify as out of scope.
Do not implement automatically.
```

This is particularly important for:

- Calendar.
- Zoom.
- Google Meet.
- Teams bot.
- Cloud backend.
- Analytics.
- Integrations.
- User accounts.

---

# 40. Code Quality

Prefer:

- Small modules.
- Clear interfaces.
- Strong typing.
- Error handling.
- Comments for non-obvious Windows behavior.
- Minimal dependencies.

Avoid:

- Giant files.
- Global mutable state.
- Duplicate business logic.
- Hard-coded paths.
- Hard-coded API keys.
- Hard-coded Teams assumptions scattered across the project.

---

# 41. Rust Development

Rust should own native Windows functionality.

Suggested areas:

```text
teams/
audio/
storage/
state/
commands/
```

React should request native functionality through Tauri commands/events.

---

# 42. React Development

React should own:

```text
components
views
user interactions
display state
form validation
MOM editing
notes editing
```

Do not put Windows API calls directly into React.

---

# 43. Async Rules

Never block the UI with:

- Audio operations.
- File operations that may take significant time.
- OpenAI requests.
- Long transcription processing.

Use asynchronous/background operations.

---

# 44. Testing During Development

After each meaningful change:

```text
Build
 ↓
Run
 ↓
Test changed behavior
```

Do not wait until the end of the project to test everything.

---

# 45. Git Commit Strategy

Use small commits.

Examples:

```text
chore: initialize tauri app
feat: add system tray
feat: detect teams process
feat: add meeting detection dialog
feat: add audio capture poc
feat: save meeting notes locally
feat: add transcription service
feat: add mom generation
```

Avoid huge commits such as:

```text
final application
```

---

# 46. Branching

For a personal V1 project, simple Git workflow is sufficient.

Possible:

```text
main
```

plus short-lived feature branches when useful.

Do not create a complex branching strategy unless the project grows.

---

# 47. Sample Data

Use synthetic test data during development.

Example:

```text
Sample Meeting
Participants:
Alice
Bob
Charlie

Discussion:
Service portfolio review...

Decision:
Architecture review required...

Action:
Bob to confirm timeline.
```

Never put confidential real meeting content in the repository.

---

# 48. Local Test Environment

Keep a dedicated test meeting workflow.

Useful setup:

```text
Teams test meeting
+
Second participant/device
+
Headset
+
Laptop speakers
```

Use this to validate:

- Detection.
- Audio capture.
- Muting.
- Unmuting.
- Meeting end.
- Transcription.

---

# 49. OpenAI Test Cost

Use short recordings while developing.

Recommended:

```text
2–5 minute test meeting
```

before testing:

```text
60–120 minute meeting
```

This reduces unnecessary API usage during development.

---

# 50. Mock AI Mode

Support a development mode such as:

```text
OPENAI_MOCK=true
```

This allows:

```text
Fake transcript
+
Fake MOM
```

without calling OpenAI.

Use mock mode for UI development.

---

# 51. Debugging Priority

When the complete workflow fails, debug from the source forward:

```text
1. Teams detection
2. Audio capture
3. Audio file
4. Transcription
5. Transcript
6. Manual notes
7. MOM generation
8. MOM UI
9. Storage
```

Do not debug the final output without validating earlier stages.

---

# 52. Build Validation

Before considering a feature complete:

```text
npm/build validation
+
Rust/Cargo validation
+
Tauri application launch
+
Manual feature test
```

The exact commands should match the current project/tool versions.

---

# 53. Release Build

Before producing a release:

- Build frontend.
- Build Rust/Tauri application.
- Verify Windows executable.
- Verify installer if used.
- Verify local storage.
- Verify API configuration.
- Verify tray behavior.
- Verify audio capture.

Do not distribute the first successful compile as a finished release.

---

# 54. Production Configuration

Before release:

- Disable debug logging.
- Remove test data.
- Remove mock mode unless intentionally retained.
- Verify API key is not embedded.
- Verify secrets are not committed.
- Verify error messages.
- Verify storage location.
- Verify installer behavior.

---

# 55. Developer Checklist

Before starting implementation:

```text
[ ] Windows development environment ready
[ ] Node.js installed
[ ] Rust installed
[ ] Tauri prerequisites installed
[ ] VS Code configured
[ ] Git configured
[ ] GitHub repository created
[ ] .gitignore created
[ ] .env.example created
[ ] OpenAI API access verified
```

---

# 56. Critical Gate Checklist

Before building the rest of the application:

```text
[ ] Teams process can be detected
[ ] Teams meeting can be distinguished from Teams being open
[ ] Process-specific audio capture works
[ ] Participant audio is captured
[ ] User unmuted audio is captured
[ ] User muted local speech is not independently captured
[ ] Headphones work
[ ] Recording can be saved
[ ] Recording can be played back
```

If these are not proven, stop and solve audio capture first.

---

# 57. Troubleshooting

## Application will not start

Check:

- Node.js.
- Rust.
- Windows SDK.
- Build tools.
- WebView2.
- Tauri setup.

---

## Teams is detected but no meeting is active

Improve the meeting-state detection logic.

Do not assume that a running Teams process means a meeting is active.

---

## Audio file is silent

Check:

1. Correct Teams process.
2. Windows audio session.
3. Output device.
4. Process Loopback configuration.
5. Audio stream availability.

Do not immediately switch to microphone recording.

---

## User's muted speech appears

This indicates that the implementation may be capturing the physical microphone or an unintended audio source.

Treat this as a capture-isolation defect.

---

## Participant audio is missing

Check:

1. Teams output path.
2. Selected process/session.
3. Headphone routing.
4. WASAPI configuration.
5. Process identification.

---

## Transcription fails

Check:

- API key.
- API access/billing.
- Model configuration.
- Audio format.
- File size.
- Network connection.
- Current API limits.

---

## MOM is inaccurate

Check in order:

```text
Audio
 ↓
Transcript
 ↓
Manual Notes
 ↓
Prompt
 ↓
MOM model
```

Do not assume the MOM prompt is the problem.

---

# 58. Development Definition of Done

The development environment is ready when:

1. The Tauri app launches on Windows.
2. React renders successfully.
3. Rust and React communicate.
4. The system tray works.
5. Git repository is cleanly configured.
6. Secrets are excluded.
7. A Teams test environment is available.
8. Audio capture POC can be tested.
9. OpenAI API access can be tested.
10. Local meeting storage can be created.

---

# 59. Final Development Principle

Build the smallest thing that proves the next technical assumption.

The most important early assumption is:

> **Windows can reliably expose the Teams meeting audio we need without independently recording the user's microphone.**

Prove that first.

Then build the rest of the application around the proven behavior.
