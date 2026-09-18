# Implementation Plan

## Meeting Notes Assistant — V1

**Purpose:** Provide a practical implementation sequence for building the Windows desktop Meeting Notes Assistant without unnecessary scope.

---

# 1. Implementation Objective

Build the smallest reliable application that supports:

```text
Detect Teams Meeting
        ↓
Ask User
        ↓
Start Notes Session
        ↓
Capture Teams Meeting Audio
        ↓
Save Audio + Notes
        ↓
Transcribe
        ↓
Generate MOM
        ↓
Review / Edit
        ↓
Save
```

The implementation should be incremental.

Each phase must produce something testable before the next phase begins.

---

# 2. Priority Order

Implementation priority:

| Priority | Area | Importance |
|---|---|---|
| P0 | Windows/Tauri foundation | Required |
| P0 | Teams meeting detection | Required |
| P0 | Teams audio capture | Critical |
| P0 | Local meeting storage | Required |
| P0 | Manual notes | Required |
| P0 | Transcription | Required |
| P0 | MOM generation | Required |
| P0 | Review/edit/save | Required |
| P1 | System tray | Important |
| P1 | Meeting history | Important |
| P1 | Settings | Important |
| P1 | Recovery/error handling | Important |
| P2 | UI polish | Later |
| P2 | Advanced optimization | Later |

---

# 3. Phase 0 — Project Foundation

## Goal

Create a clean, runnable Tauri application.

## Tasks

- Create Tauri project.
- Configure React + TypeScript.
- Configure Rust.
- Configure Git.
- Create `.gitignore`.
- Create `.env.example`.
- Establish source structure.
- Confirm frontend builds.
- Confirm Rust builds.
- Confirm Tauri launches.

## Deliverable

A desktop application window displaying a basic placeholder UI.

## Acceptance

```text
Tauri launches successfully.
React renders.
Rust backend runs.
Git repository is clean.
```

---

# 4. Phase 1 — Application Shell

## Goal

Create the basic application behavior.

## Tasks

- Create application window.
- Add application title.
- Add system tray.
- Add open/show behavior.
- Add minimize-to-tray behavior.
- Add exit behavior.
- Add basic application state.

## Deliverable

The application can run quietly in the Windows background.

## Acceptance

```text
Launch
 ↓
Window
 ↓
Minimize
 ↓
Tray
 ↓
Restore
 ↓
Exit
```

---

# 5. Phase 2 — Teams Detection

## Goal

Detect when a Microsoft Teams meeting is active.

## Tasks

- Detect Teams processes.
- Identify relevant Teams process/session.
- Determine whether a meeting is actually active.
- Maintain detection state.
- Detect transition into meeting.
- Detect transition out of meeting.
- Avoid repeated notifications.

## Application state

Example:

```text
TeamsNotRunning
TeamsRunning
MeetingActive
MeetingEnded
```

## Deliverable

A reliable internal Teams meeting state.

## Acceptance

The application correctly distinguishes:

```text
Teams closed
Teams open
Teams open but idle
Teams in active meeting
Meeting ended
```

---

# 6. Phase 3 — Meeting Detection Notification

## Goal

Ask the user whether to take notes.

## UI

```text
Teams meeting detected

Would you like to take notes?

[ Start Taking Notes ] [ Not Now ]
```

## Tasks

- Display notification/dialog.
- Start session on confirmation.
- Suppress session on Not Now.
- Prevent duplicate prompts.
- Track current meeting session.

## Deliverable

A user-controlled recording/notes start point.

## Acceptance

No audio capture starts before the user explicitly selects:

```text
Start Taking Notes
```

---

# 7. Phase 4 — Audio Capture Proof of Concept

## Goal

Prove the most important technical requirement before building the full application.

## Approach

Preferred:

```text
Teams Process
      ↓
Windows Process Loopback / WASAPI
      ↓
Meeting Audio
      ↓
Local Audio File
```

## Tasks

- Identify Teams audio process/session.
- Implement process-specific capture.
- Start capture.
- Stop capture.
- Save test audio.
- Validate audio format.
- Validate playback.
- Handle headphones.
- Test multiple Teams process scenarios.

## Critical tests

### Participant speaks

Expected:

```text
Captured
```

### User speaks while unmuted

Expected:

```text
Captured
```

### User speaks while muted

Expected:

```text
Not independently captured
```

### Headphones

Expected:

```text
Meeting audio captured
```

## Deliverable

A standalone or isolated audio capture implementation that produces a valid meeting recording.

## Gate

**Do not continue to full application integration until this works acceptably.**

---

# 8. Phase 5 — Meeting Session Management

## Goal

Create a consistent session object.

Example:

```text
MeetingSession
├── id
├── startedAt
├── endedAt
├── teamsDetected
├── recordingStarted
├── recordingStopped
├── notes
├── transcript
└── mom
```

## Tasks

- Generate unique session ID.
- Start session.
- Track state.
- Stop session.
- Handle unexpected Teams meeting end.
- Save session metadata.

## Deliverable

A complete local representation of one meeting.

---

# 9. Phase 6 — Local Storage

## Goal

Persist meeting information without a database.

## Recommended structure

```text
MeetingNotesAssistant/
└── Meetings/
    └── YYYY-MM-DD_MeetingName_ID/
        ├── meeting.json
        ├── audio.wav
        ├── transcript.json
        ├── transcript.md
        ├── notes.md
        └── mom.md
```

The exact root location should be configurable.

## Tasks

- Create meeting directory.
- Write metadata.
- Write audio.
- Write notes.
- Write transcript.
- Write MOM.
- Implement safe writes.
- Implement auto-save.
- Implement recovery.

## Deliverable

Meeting data survives application restart.

---

# 10. Phase 7 — Meeting Workspace UI

## Goal

Create the primary working screen.

Suggested layout:

```text
┌───────────────────────────────────────────┐
│ Meeting title        ● Recording   Stop   │
├──────────────────────┬────────────────────┤
│                      │                    │
│ Transcript           │ My Notes           │
│                      │                    │
│ Speaker/timestamp    │ User notes         │
│ Transcript text      │                    │
│                      │                    │
├──────────────────────┴────────────────────┤
│ Status / Save status                       │
└───────────────────────────────────────────┘
```

## Tasks

- Transcript panel.
- Notes panel.
- Recording status.
- Stop/end control.
- Save indicator.
- Meeting duration.

## Deliverable

A usable meeting workspace.

---

# 11. Phase 8 — Manual Notes

## Goal

Allow the user to capture personal notes while the meeting is happening.

## Tasks

- Notes editor.
- Auto-save.
- Restore notes.
- Keyboard-friendly editing.
- Unsaved-state indicator.
- Preserve notes separately from transcript.

## Important rule

The user's notes are authoritative user input.

The AI must not silently rewrite them.

---

# 12. Phase 9 — Transcription Service

## Goal

Convert recorded meeting audio into text.

## Pipeline

```text
audio.wav
   ↓
Audio validation
   ↓
OpenAI transcription
   ↓
Transcript result
   ↓
Validation
   ↓
Local storage
```

## Tasks

- Create `TranscriptionService`.
- Configure transcription model.
- Send audio.
- Handle response.
- Store transcript.
- Store timestamps where available.
- Handle errors.
- Handle retries.
- Support long recordings/chunking if required.

## Deliverable

A transcript generated from a completed meeting recording.

---

# 13. Phase 10 — Transcript Model

## Goal

Use a structured internal transcript representation.

Example:

```json
{
  "segments": [
    {
      "start": 12.4,
      "end": 18.9,
      "speaker": "Speaker 1",
      "text": "We should review the service portfolio."
    }
  ]
}
```

Speaker labels must not be invented.

If reliable speaker identification is unavailable:

```text
Speaker 1
Speaker 2
```

or no speaker label should be used.

---

# 14. Phase 11 — MOM Generation

## Goal

Generate a useful Minutes of Meeting document from:

```text
Transcript
+
Manual Notes
```

## Pipeline

```text
Transcript
      +
Manual Notes
      ↓
Prompt construction
      ↓
OpenAI generation
      ↓
Structured MOM
      ↓
Validation
      ↓
Editable MOM
```

## Recommended output

```text
Meeting Title
Date
Participants
Purpose / Agenda
Key Discussion Points
Decisions
Action Items
Open Questions
Next Steps
```

---

# 15. Phase 12 — MOM Accuracy Controls

## Goal

Prevent the AI from inventing commitments.

The AI should distinguish between:

```text
Explicit decision
Explicit action
Suggestion
Discussion
Open question
```

Example:

If transcript says:

```text
"We could review this next week."
```

Do not automatically create:

```text
Action Item:
John will review this next week.
```

unless ownership/commitment is actually established.

---

# 16. Phase 13 — MOM Review UI

## Goal

Allow the user to review and edit AI output.

## Tasks

- Display generated MOM.
- Make content editable.
- Save edits.
- Copy MOM.
- Regenerate MOM.
- Preserve latest saved version.
- Show generation status.

## Important rule

AI-generated content is a draft.

The user remains the final authority.

---

# 17. Phase 14 — End-to-End Workflow

Connect all components:

```text
Teams meeting starts
        ↓
Detection
        ↓
Popup
        ↓
User selects Start
        ↓
Meeting session created
        ↓
Audio capture
        +
Manual notes
        ↓
Meeting ends / user stops
        ↓
Audio saved
        ↓
Transcription
        ↓
Transcript saved
        ↓
Transcript + notes
        ↓
MOM generation
        ↓
MOM review
        ↓
Edit
        ↓
Save
```

## Deliverable

A complete working V1 workflow.

---

# 18. Phase 15 — Meeting History

## Goal

Allow the user to find previous meetings.

## Tasks

- Scan meeting directories.
- Read `meeting.json`.
- Display:
  - Date
  - Meeting title
  - Duration
  - Status
- Open a meeting.
- View transcript.
- View notes.
- View MOM.

## Do not add

- Advanced search engine.
- Database.
- Analytics.
- Tags.
- Complex filtering.

Unless later required.

---

# 19. Phase 16 — Settings

## Goal

Provide only essential configuration.

Settings:

```text
OpenAI API Key
Transcription Model
MOM Model
Storage Location
Start with Windows
Meeting Notification
```

## Optional

```text
Delete audio after transcription
```

if retention requirements justify it.

---

# 20. Phase 17 — Error Handling

Implement clear user-facing states.

Examples:

```text
Teams not detected
Audio capture unavailable
Audio capture stopped unexpectedly
No audio captured
Transcription failed
OpenAI unavailable
MOM generation failed
Storage unavailable
Disk space low
```

Every error should provide a useful next step.

---

# 21. Phase 18 — Recovery

The application must tolerate:

- Teams closing unexpectedly.
- Teams meeting ending unexpectedly.
- Application crash.
- Computer sleep.
- Temporary network failure.
- OpenAI failure.
- Disk write failure.

## Recovery principle

Never lose already-saved notes merely because a later processing step fails.

Example:

```text
Transcription failed
```

must not delete:

```text
Audio
+
Manual Notes
```

---

# 22. Phase 19 — Resource Management

Monitor:

- CPU.
- Memory.
- Disk usage.
- Audio buffer size.
- Temporary files.

The application should remain lightweight when idle.

---

# 23. Phase 20 — Security Review

Before release verify:

```text
[ ] API key protected
[ ] No secrets in Git
[ ] No hidden recording
[ ] User explicitly starts capture
[ ] No independent microphone capture
[ ] Local files protected appropriately
[ ] Logs do not contain meeting content
[ ] Temporary files cleaned
[ ] OpenAI communication uses HTTPS
```

---

# 24. Phase 21 — UI Polish

Only after functionality is stable:

- Improve spacing.
- Improve typography.
- Add loading indicators.
- Improve empty states.
- Improve error messages.
- Improve tray behavior.
- Add keyboard shortcuts.
- Improve accessibility.

Do not let visual polish delay the audio proof of concept.

---

# 25. Phase 22 — Testing

Testing should happen at multiple levels.

## Unit tests

Examples:

```text
Meeting ID generation
State transitions
Path generation
MOM parsing
Configuration validation
```

## Integration tests

Examples:

```text
React ↔ Rust
Teams detection ↔ application state
Audio capture ↔ storage
Audio ↔ transcription
Transcript + notes ↔ MOM
```

## Manual tests

Real Windows environment:

```text
Teams
Headphones
Microphone
Mute/unmute
Meeting start/end
Application restart
```

---

# 26. Audio Test Matrix

The audio capture test matrix is a release-critical artifact.

| Scenario | Expected |
|---|---|
| Teams closed | No capture |
| Teams open idle | No capture |
| Meeting active, not started | No capture |
| User selects Start | Capture begins |
| Participant speaks | Captured |
| User unmuted and speaks | Captured |
| User muted and speaks locally | Not independently captured |
| Headphones | Works |
| Speakers | Works |
| Meeting ends | Capture stops/ends appropriately |
| User manually stops | Capture stops |
| Capture failure | Clear error/recovery |

---

# 27. End-to-End Acceptance Test

Perform a real test meeting.

### Before meeting

```text
Application running
Teams running
Application idle
```

### Meeting starts

Expected:

```text
Teams meeting detected
```

### User selects Start

Expected:

```text
Recording indicator visible
Meeting workspace opens
```

### During meeting

Expected:

```text
Participant audio captured
User unmuted speech captured
Manual notes can be entered
Notes auto-save
```

### Meeting ends

Expected:

```text
Audio saved
Session closed
Transcription begins
```

### Processing completes

Expected:

```text
Transcript available
MOM generated
```

### Review

Expected:

```text
MOM editable
User can correct content
User can save/copy
```

---

# 28. Implementation Order for Claude

Claude should implement tasks in this order:

```text
01 Foundation
02 App Shell
03 System Tray
04 Teams Detection
05 Detection Notification
06 Audio POC
07 Session Management
08 Local Storage
09 Meeting Workspace
10 Manual Notes
11 Transcription
12 MOM Generation
13 Review/Edit
14 History
15 Settings
16 Recovery
17 Testing
18 Packaging
```

---

# 29. One Task at a Time

Each Claude request should have:

```text
Objective
Relevant specification
Files allowed to change
Expected behavior
Acceptance criteria
```

Example:

> Implement Phase 4 Audio Capture POC according to `AUDIO_CAPTURE.md`. Only modify the native audio-related files and required configuration. Do not implement transcription or MOM generation. Provide a test procedure and report any Windows-specific limitation.

---

# 30. Avoid Uncontrolled Refactoring

Claude should not rewrite unrelated parts of the application while implementing a feature.

If a structural refactor becomes necessary:

1. Explain why.
2. Identify affected files.
3. Keep the refactor small.
4. Run existing tests.
5. Continue implementation only after the application remains stable.

---

# 31. Dependency Policy

Before adding a package:

Ask:

```text
Is this required?
Is there already a project dependency that solves it?
Does Windows/Tauri already provide the capability?
Does adding it significantly increase complexity?
```

Prefer fewer dependencies.

---

# 32. No Unnecessary Infrastructure

V1 does not require:

```text
Hostinger
VPS
Cloud server
Backend server
MySQL
PostgreSQL
Firebase
Supabase
Redis
Kubernetes
Docker
```

It also does not require:

```text
Teams bot
Teams app
Meeting participant
Calendar integration
```

---

# 33. Development Milestones

Suggested milestone grouping:

## Milestone 1 — Foundation

```text
Tauri
React
Rust
Git
Tray
```

## Milestone 2 — Detection

```text
Teams detection
Meeting state
Notification
```

## Milestone 3 — Capture

```text
Audio POC
Capture validation
```

## Milestone 4 — Notes

```text
Session
Storage
Workspace
Manual notes
```

## Milestone 5 — AI

```text
Transcription
MOM
```

## Milestone 6 — Product Completion

```text
Review
History
Settings
Recovery
```

## Milestone 7 — Release

```text
Testing
Packaging
Windows validation
```

---

# 34. Definition of V1 Complete

V1 is complete when a user can:

1. Launch the Windows application.
2. Leave it running in the background.
3. Join a Microsoft Teams meeting.
4. Receive a meeting-detected prompt.
5. Explicitly choose to take notes.
6. Capture the intended Teams meeting audio.
7. Type personal notes.
8. End the session.
9. Generate a transcript.
10. Generate an AI-assisted MOM.
11. Edit the MOM.
12. Save the meeting locally.
13. Reopen the meeting later.
14. Review notes, transcript, and MOM.

---

# 35. V1 Release Gate

Do not call the application V1 complete if any of these remain fundamentally broken:

```text
Teams detection
Audio capture
User consent
Local storage
Transcription
MOM generation
MOM editing
Meeting recovery
```

Minor visual imperfections can remain.

Core workflow reliability cannot.

---

# 36. Future Enhancements

Potential future versions may consider:

```text
Near-live transcription
Speaker diarization
Calendar integration
Zoom support
Google Meet support
Cloud synchronization
Search
Advanced meeting history
Integrations
Team collaboration
Enterprise deployment
```

These are intentionally deferred.

---

# 37. Final Implementation Principle

The implementation should follow one rule:

> **Prove the risky technical assumptions first, then build the user experience around what is proven.**

For this project, the highest-risk assumption is the ability to capture the correct Teams meeting audio on Windows while respecting the user's mute state.

Therefore:

```text
Audio Capture POC
        ↓
Prove It
        ↓
Integrate It
        ↓
Build Everything Else
```

Keep V1 small, local, understandable, and reliable.
