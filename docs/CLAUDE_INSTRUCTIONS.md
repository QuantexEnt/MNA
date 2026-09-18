# CLAUDE_INSTRUCTIONS.md

# Meeting Notes Assistant — Instructions for Claude

## 1. Purpose

You are helping build **Meeting Notes Assistant**, a small Windows desktop application for taking notes during Microsoft Teams meetings.

The application is intentionally simple.

The core workflow is:

```text
Detect Teams Meeting
        ↓
Ask User
        ↓
Start Taking Notes
        ↓
Capture Teams Meeting Audio
        ↓
Manual Notes
        ↓
Meeting Ends
        ↓
Transcription
        ↓
Generate MOM
        ↓
Review / Edit
        ↓
Save Locally
```

Your job is to implement this product according to the project documentation without expanding its scope unnecessarily.

---

# 2. Read the Documentation First

Before making significant implementation changes, read the relevant project documents.

At minimum, understand:

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
SETUP_AND_DEVELOPMENT.md
IMPLEMENTATION_PLAN.md
TESTING_AND_ACCEPTANCE.md
DECISIONS.md
```

These documents collectively define the V1 product.

If a task touches multiple areas, read all relevant specifications before coding.

---

# 3. Source of Truth

Use the project documentation as the source of truth.

Do not replace documented behavior with assumptions from:

- Generic tutorials.
- Other meeting applications.
- Old code examples.
- Personal preference.
- A different architecture that happens to be easier.

If documentation and existing code disagree:

1. Identify the conflict.
2. Check `DECISIONS.md`.
3. Explain the conflict.
4. Preserve the agreed product behavior.
5. Update documentation only if the product decision is intentionally changed.

Never silently change a requirement.

---

# 4. Product Boundary

This is a **Meeting Notes Assistant**, not a full meeting-management platform.

V1 includes:

- Windows desktop application.
- Microsoft Teams meeting detection.
- User-controlled meeting capture.
- Teams meeting audio capture.
- Manual notes.
- Transcription.
- AI-generated MOM.
- Editable MOM.
- Local meeting history.
- Basic settings.
- Local storage.

V1 does not include:

- Teams bot.
- Teams app/add-in.
- Meeting participant/bot.
- Zoom.
- Google Meet.
- Webex.
- Calendar integration.
- Cloud backend.
- Cloud database.
- Hostinger/VPS.
- MySQL.
- PostgreSQL.
- Firebase.
- Supabase.
- User accounts.
- Multi-user collaboration.
- Subscription system.
- Mobile app.
- Browser extension.
- Advanced analytics.
- Psychological analysis.
- Sentiment analysis.
- Complex dashboards.
- Project-management functionality.

Do not add these unless the user explicitly changes the V1 scope.

---

# 5. Most Important Technical Requirement

The most important technical requirement is **correct Teams meeting audio capture on Windows**.

The application must capture the intended meeting audio rather than independently recording the user's physical microphone.

Required behavior:

```text
Other participant speaks
        ↓
Captured

User is unmuted in Teams and speaks
        ↓
Captured

User is muted in Teams and speaks locally
        ↓
Must NOT be independently captured
```

This requirement is more important than UI polish.

---

# 6. Audio Capture Rule

Do not implement this as:

```text
Microphone
    ↓
Recorder
```

That is not the intended solution.

The preferred technical direction is:

```text
Teams process/session
        ↓
Windows WASAPI / Process Loopback Capture
        ↓
Meeting audio
```

The exact Windows/Teams behavior must be validated experimentally.

If the chosen implementation cannot reliably satisfy the required behavior:

- Do not silently switch to microphone recording.
- Do not claim the requirement works.
- Report the limitation.
- Investigate a technically appropriate alternative.
- Update the decision record only if the product requirement is intentionally changed.

---

# 7. Audio Proof-of-Concept Gate

Before building a large amount of UI or AI functionality, prove the audio capture path.

The POC must test:

```text
Participant audio
User unmuted audio
User muted local speech
Headphones
Laptop speakers
Unrelated system audio
Start/stop
Playback
```

A successful compile is not proof of successful audio capture.

Always test actual captured audio.

---

# 8. User Consent

Recording must never begin automatically.

When a Teams meeting is detected, the application should ask:

```text
Teams meeting detected

Would you like to take notes?

[ Start Taking Notes ] [ Not Now ]
```

Only:

```text
Start Taking Notes
```

starts the meeting capture session.

If the user chooses:

```text
Not Now
```

there must be no recording.

---

# 9. Recording Visibility

The user must always be able to tell whether capture is active.

Use a clear recording state such as:

```text
● Recording
```

or an equivalent obvious indicator.

Never hide recording state.

---

# 10. Technology Stack

Use:

```text
Tauri
React
TypeScript
Rust
Windows APIs
WASAPI / Process Loopback where appropriate
Local files
OpenAI API
Git/GitHub
```

React is for the UI.

Rust/Tauri is for native Windows functionality.

---

# 11. Frontend Responsibilities

React/TypeScript should handle:

- Application UI.
- Meeting workspace.
- Transcript display.
- Manual notes.
- MOM display/editing.
- History.
- Settings.
- User interactions.
- UI state.

Do not put Windows process/audio API logic directly into React.

---

# 12. Rust Responsibilities

Rust/Tauri should handle or coordinate:

- Teams detection.
- Windows audio capture.
- Native filesystem operations where appropriate.
- System tray.
- Native application lifecycle.
- Application state.
- Tauri commands/events.
- Native error handling.

Keep native functionality isolated and testable.

---

# 13. OpenAI

Use OpenAI for:

```text
Audio → Transcript
Transcript + Notes → MOM
```

One OpenAI API key is sufficient for both services.

Do not confuse:

```text
ChatGPT subscription
```

with:

```text
OpenAI API access
```

They are separate.

---

# 14. OpenAI API Key Security

Never hard-code an API key.

Never commit it to Git.

Never put it in:

- Source code.
- Public configuration.
- Logs.
- Test data.
- Screenshots.
- Documentation.

Use local configuration/environment mechanisms during development.

For a future distributed commercial application, do not embed a personal API key into the desktop binary.

---

# 15. Model Configuration

Do not scatter model IDs throughout the code.

Use configuration such as:

```text
transcription_model
mom_model
```

Keep model selection isolated so models can be changed without rewriting business logic.

Verify current model/API availability when implementation requires it.

---

# 16. Transcription

V1 baseline:

```text
Meeting audio
    ↓
Saved locally
    ↓
OpenAI transcription
    ↓
Transcript
```

Post-meeting transcription is the required baseline.

Near-live transcription is optional and should not delay V1.

Do not fabricate speaker identities.

---

# 17. MOM Generation

MOM generation uses:

```text
Transcript
+
Manual Notes
```

The generated MOM should normally contain:

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

The exact structure should follow `NOTES_AND_MOM_SPECIFICATION.md`.

---

# 18. MOM Accuracy

The AI must not invent:

- Participants.
- Decisions.
- Owners.
- Deadlines.
- Commitments.
- Facts not supported by the inputs.

Distinguish between:

```text
Explicit commitment
Suggestion
Discussion
Decision
Open question
```

For example:

```text
"We could review this next week."
```

must not automatically become:

```text
Action Item:
John will review this next week.
```

unless the meeting actually establishes that commitment.

---

# 19. MOM Is Editable

The AI-generated MOM is a draft.

The user must be able to:

- Edit it.
- Save it.
- Copy it.
- Review it.
- Regenerate it explicitly.

Regeneration must not silently destroy user edits.

---

# 20. Local Storage

V1 is local-first.

Do not introduce a backend or database unless the user explicitly changes the architecture.

Preferred storage:

```text
Folders
JSON
Markdown
Audio files
```

Conceptually:

```text
Meetings/
└── Meeting_ID/
    ├── meeting.json
    ├── audio.wav
    ├── transcript.json
    ├── transcript.md
    ├── notes.md
    └── mom.md
```

Keep meeting data understandable and portable.

---

# 21. Data Safety

Never delete useful meeting data because a later processing step failed.

For example:

```text
Transcription failed
```

must not delete:

```text
Audio
+
Manual Notes
```

The user should be able to retry processing.

---

# 22. Error Handling

Errors must be explicit and useful.

Examples:

```text
Teams not detected
Audio capture unavailable
No audio captured
Transcription failed
OpenAI unavailable
MOM generation failed
Storage unavailable
Disk space low
```

Avoid generic errors such as:

```text
Something went wrong.
```

when a more useful explanation is possible.

---

# 23. Async Behavior

Do not block the UI with:

- Audio capture.
- Large file operations.
- OpenAI requests.
- Transcription.
- MOM generation.
- Long-running processing.

Use asynchronous/background operations.

The application should remain responsive.

---

# 24. Application States

Maintain clear states.

Examples:

```text
IDLE
TEAMS_DETECTED
WAITING_FOR_USER
RECORDING
STOPPING
PROCESSING
TRANSCRIBING
GENERATING_MOM
REVIEW
ERROR
```

Avoid scattered boolean flags when a clear state machine is more appropriate.

---

# 25. Teams Detection

Do not assume:

```text
Teams process exists
=
Meeting is active
```

Teams may be running while the user is:

- At the home screen.
- Chatting.
- Browsing Teams.
- In another non-meeting state.

Meeting detection must be handled separately from simple process detection.

---

# 26. Duplicate Meetings / Events

Windows and Teams state changes can occur repeatedly.

Prevent:

- Duplicate prompts.
- Duplicate recordings.
- Multiple active sessions for one meeting.

Rule:

```text
One real meeting
=
One active meeting session
```

unless the user explicitly starts another session.

---

# 27. Session Management

Each meeting should have a unique ID.

A session should track at least:

```text
id
startedAt
endedAt
duration
recording state
notes
transcript state
MOM state
```

Use the existing specifications for the complete data model.

---

# 28. Notes

Manual notes are first-class user input.

Requirements:

- Easy to type.
- Auto-save.
- Preserve formatting where practical.
- Survive restart/crash as far as possible.
- Remain separate from transcript.
- Never silently rewrite user notes.

---

# 29. UI Philosophy

The UI should be:

- Simple.
- Quiet.
- Clean.
- Fast.
- Understandable.
- Functional.

Do not build a dashboard-heavy product.

The primary workspace should make these obvious:

```text
Recording state
Transcript
My Notes
Meeting duration
Stop/end control
```

---

# 30. System Tray

The application should be able to remain in the Windows background.

Expected:

```text
Launch
 ↓
Tray
 ↓
Teams meeting detected
 ↓
Notification
 ↓
User opens meeting workspace
```

Closing the main window should not necessarily terminate the application if tray mode is active.

The explicit Exit action should terminate it safely.

---

# 31. Privacy

Default behavior:

```text
No hidden recording
No independent microphone recording
No telemetry
No unnecessary cloud storage
No meeting-content logging
```

Only send data to OpenAI when required for the requested AI operation.

---

# 32. Logging

Logs are for technical diagnostics.

Do not log:

- Full transcripts.
- Full meeting notes.
- Audio content.
- API keys.
- Sensitive meeting information.

Prefer events such as:

```text
Teams detected
Session started
Audio capture initialized
Audio capture stopped
Transcription started
Transcription completed
MOM generation failed
```

---

# 33. Dependency Rules

Before adding a dependency, ask:

1. Is it actually necessary?
2. Does Tauri/Windows already provide this?
3. Does the project already have a suitable dependency?
4. Does it significantly increase maintenance?
5. Does it introduce security/privacy concerns?

Prefer the smallest reasonable dependency set.

---

# 34. No Premature Infrastructure

Do not introduce:

```text
Hostinger
VPS
Docker
Kubernetes
MySQL
PostgreSQL
Redis
Firebase
Supabase
Cloud backend
```

for V1.

The application does not need them.

---

# 35. Implementation Style

Implement small pieces.

For each task:

```text
Read specification
        ↓
Inspect existing code
        ↓
Identify affected files
        ↓
Implement smallest change
        ↓
Build/test
        ↓
Report result
```

Do not rewrite the entire application to implement one feature.

---

# 36. Before Editing Code

Before making a change:

- Read the relevant specification.
- Inspect the existing implementation.
- Check related types/interfaces.
- Check current state management.
- Check whether a similar capability already exists.

Do not blindly create duplicate services/components.

---

# 37. File Modification Discipline

Modify only files necessary for the task.

If unrelated cleanup is noticed:

- Mention it.
- Do not automatically include it in the current change.

Avoid large unrelated refactors.

---

# 38. Testing Requirement

Every meaningful implementation change should be followed by appropriate validation.

At minimum:

```text
Build
+
Relevant test
+
Manual verification where required
```

For Windows/audio behavior, actual manual testing is essential.

---

# 39. Audio Testing Requirement

Never declare audio capture complete because the application successfully creates an audio file.

Verify the content.

The test must include:

```text
Participant speech
User unmuted speech
User muted local speech
Headphones
Speakers
```

---

# 40. Mock AI

When implementing UI or workflows that do not require real OpenAI calls, use mock responses where practical.

Example:

```text
OPENAI_MOCK=true
```

Mock mode should allow development of:

- Transcript UI.
- MOM UI.
- Loading states.
- Error states.
- Review/edit workflow.

This reduces unnecessary API usage.

---

# 41. Development Cost Control

During development, prefer short test recordings.

Use:

```text
2–5 minute recordings
```

before testing long meetings.

Do not repeatedly process large recordings while debugging UI behavior.

---

# 42. Git

Use small meaningful commits.

Examples:

```text
chore: initialize tauri application
feat: add system tray
feat: detect teams meeting
feat: add meeting detection dialog
feat: add audio capture poc
feat: add local meeting storage
feat: add transcription service
feat: add mom generation
```

Do not commit:

- `.env`
- API keys.
- Real meeting audio.
- Confidential transcripts.
- Confidential MOM files.
- Temporary debug data.

---

# 43. Branching

Simple Git is sufficient.

Use:

```text
main
```

and feature branches when useful.

Do not create a complicated Git workflow for this project.

---

# 44. How to Handle Ambiguity

If a requirement is unclear:

1. Check all relevant project documents.
2. Check `DECISIONS.md`.
3. Prefer the smallest interpretation consistent with V1.
4. If the ambiguity affects a critical architectural decision, stop and ask the user before implementing it.

Do not invent major product requirements.

---

# 45. How to Handle Conflicting Requirements

If two documents appear to conflict:

```text
Do not guess.
Do not silently choose.
```

Instead:

1. Identify the conflict.
2. Explain the two interpretations.
3. Recommend the smallest safe option.
4. Ask for a decision if needed.
5. Update the documentation after the decision.

---

# 46. Out-of-Scope Request Handling

If the user asks for a feature that is outside V1, do not automatically implement it.

State briefly:

```text
This is currently outside the V1 scope.
```

Then explain whether it should be:

- Deferred.
- Added as a future enhancement.
- Accepted as a deliberate scope change.

If the user explicitly approves the scope change, update the relevant documentation.

---

# 47. Critical Priority

Use this order when making tradeoffs:

```text
1. Correctness
2. Privacy / consent
3. Reliability
4. Core functionality
5. Simplicity
6. Maintainability
7. UI polish
8. Extra features
```

Never sacrifice recording consent or capture correctness for convenience.

---

# 48. Definition of Done

A task is done only when:

- Implementation is complete.
- Relevant code builds.
- Relevant tests pass.
- The behavior matches the specification.
- Errors are handled appropriately.
- No unrelated scope was introduced.
- Security/privacy requirements are respected.

For UI changes, manually verify the actual application.

For audio changes, manually verify actual captured audio.

---

# 49. V1 Release Gate

Do not call V1 complete until the following work reliably:

```text
[ ] Windows application launches
[ ] Teams meeting detection works
[ ] User explicitly starts capture
[ ] Participant audio is captured
[ ] User unmuted speech is captured
[ ] User muted local speech is not independently captured
[ ] Headphones work
[ ] Manual notes work
[ ] Notes auto-save
[ ] Audio is stored locally
[ ] Transcription works
[ ] MOM generation works
[ ] MOM does not invent commitments
[ ] MOM is editable
[ ] Meeting can be reopened
[ ] Failures do not unnecessarily lose audio/notes
[ ] API key is protected
[ ] No hidden recording
[ ] No unnecessary cloud backend
```

---

# 50. Recommended Implementation Order

Follow this order unless there is a strong technical reason not to:

```text
1. Tauri foundation
2. React UI
3. Rust ↔ React communication
4. System tray
5. Teams detection
6. Meeting detection notification
7. Audio capture POC
8. Session management
9. Local storage
10. Meeting workspace
11. Manual notes
12. Transcription
13. MOM generation
14. MOM review/edit
15. Meeting history
16. Settings
17. Error/recovery handling
18. Testing
19. Packaging
20. Final acceptance
```

The audio POC is a gate.

---

# 51. Important: Do Not Overbuild

Do not turn this project into:

```text
A meeting intelligence platform
```

It is:

```text
A simple Windows meeting notes assistant
```

The user should be able to install it, leave it running, join a Teams meeting, start taking notes, finish the meeting, and receive a useful editable MOM.

That is the product.

---

# 52. Final Instruction

When uncertain, prefer:

```text
Smaller
Simpler
Local
Testable
Explicit
Reliable
```

over:

```text
Bigger
More automated
Cloud-based
Complex
Over-engineered
```

The guiding principle is:

> **Build only what is needed to make the core Teams meeting → audio → notes → transcript → MOM workflow reliable.**

