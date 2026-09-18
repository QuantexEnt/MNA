# Product Requirements Document (PRD)

## Meeting Notes Assistant — V1

**Document Status:** Approved for V1 development  
**Product Type:** Windows desktop application  
**Primary Meeting Platform:** Microsoft Teams  
**Deployment Model:** Local desktop application  
**Cloud Backend:** None required for V1  
**Primary AI Service:** OpenAI API

---

# 1. Product Purpose

Meeting Notes Assistant is a lightweight Windows desktop application that helps a user create useful meeting transcripts and Minutes of Meeting (MOM) from Microsoft Teams meetings.

The application runs independently on the user's Windows computer.

It does **not** join the Teams meeting, become a meeting participant, install itself into the meeting, or require a Teams bot.

The application detects when a Teams meeting is active and asks the user whether they want to take notes.

If the user agrees, the application captures the meeting audio, creates a transcript, allows the user to enter personal notes, and generates an editable MOM at the end of the meeting.

---

# 2. Problem Statement

During meetings, users often need to:

- Listen to the discussion.
- Remember important points.
- Write their own notes.
- Capture decisions.
- Remember action items.
- Prepare a Minutes of Meeting afterward.

Doing all of these manually can cause important information to be missed.

The application should reduce this effort without becoming a complicated meeting-management system.

The desired experience is:

> **Join meeting → Start note taking → Listen and capture → Add personal notes → Generate MOM**

---

# 3. Product Vision

The product should be a small, reliable desktop utility that stays in the background and becomes useful only when the user is in a meeting.

The core value proposition is:

> **Capture the conversation, add your own thoughts, and turn both into a clean MOM.**

The application should feel like a personal note-taking assistant rather than a meeting participant.

---

# 4. Target User

The initial target user is an individual professional who:

- Uses Windows.
- Uses Microsoft Teams for meetings.
- Wants a transcript of meetings.
- Wants to write personal notes during meetings.
- Frequently needs to prepare MOMs.
- Does not want to add a bot or meeting assistant to meetings.
- Prefers meeting information to remain on their own computer.

The V1 product is primarily designed for personal use.

---

# 5. Target Platform

## Required

- Windows 10 or later.
- Microsoft Teams desktop application.
- Internet access when using the OpenAI API.

## Not required

- Hostinger.
- Cloud server.
- Cloud database.
- Microsoft 365 developer account.
- Teams bot account.
- Teams app registration.
- Zoom account.
- Google Meet account.

---

# 6. Core User Journey

The complete V1 workflow is:

```text
User starts application
        ↓
Application runs in background
        ↓
User joins Microsoft Teams meeting
        ↓
Application detects active meeting
        ↓
Notification appears
        ↓
"Would you like to take notes?"
        ↓
     ┌──┴──┐
     │     │
   Start  Not Now
     │
     ▼
Meeting workspace opens
     ↓
Meeting audio capture starts
     ↓
Transcription begins
     ↓
User can type personal notes
     ↓
Meeting ends / user stops session
     ↓
Transcript + personal notes
     ↓
Generate MOM
     ↓
User reviews and edits MOM
     ↓
Save/export locally
```

---

# 7. Functional Requirements

## FR-001 — Application Startup

The application shall start as a Windows desktop application.

The application should be able to remain running in the background.

The user should be able to minimize the application without closing it.

---

## FR-002 — Background Operation

The application shall support background operation.

When no meeting is active, the application should consume minimal system resources.

The application should not continuously record audio when the user is not actively taking notes.

---

## FR-003 — Teams Meeting Detection

The application shall detect when the user is participating in an active Microsoft Teams meeting.

The detection mechanism should work without requiring the application to join the meeting.

The application should not require Teams credentials for basic meeting detection.

The detection implementation should be isolated from the rest of the application so it can be improved later without redesigning the entire product.

---

## FR-004 — Meeting Detection Notification

When an active Teams meeting is detected, the application shall display a small notification/popup.

Example:

> **Microsoft Teams meeting detected**  
> Would you like to take notes?

Actions:

- **Start Taking Notes**
- **Not Now**

The notification should be unobtrusive.

---

## FR-005 — User Consent Before Capture

The application shall not start meeting audio capture automatically merely because a Teams meeting is detected.

The user must explicitly select:

**Start Taking Notes**

before meeting audio capture begins.

Selecting:

**Not Now**

must leave audio capture disabled for that meeting.

---

## FR-006 — Meeting Audio Capture

The application shall capture the audio associated with the Teams meeting.

The primary objective is to capture the audio being transmitted/played by Teams rather than independently recording the user's microphone.

Expected behavior:

### Other participant speaking

Their voice should be captured.

### User speaking while unmuted

The user's voice should be captured because it is being transmitted into the Teams meeting.

### User speaking while muted

The user's local speech should not be captured as meeting audio because it is not transmitted into the Teams meeting.

### Headphones

The application should support meetings where the user uses headphones/headsets.

### Unrelated system sounds

The application should avoid capturing unrelated system audio where technically possible.

The detailed technical requirements are defined in `AUDIO_CAPTURE.md`.

---

## FR-007 — Recording State

The application shall clearly indicate when meeting audio capture is active.

The user should be able to:

- Start capture.
- Pause capture if supported by the selected implementation.
- Resume capture.
- Stop/end capture.

The application should prevent accidental continuation of recording after the meeting session has ended.

---

## FR-008 — Meeting Workspace

After the user starts note taking, the application shall provide a simple meeting workspace.

The workspace should contain, at minimum:

1. Meeting information.
2. Transcript area.
3. Personal notes area.
4. Recording/capture status.
5. End Meeting / Stop button.

The interface should remain simple and focused.

---

## FR-009 — Transcription

The application shall convert captured meeting audio into text.

The transcript should be associated with the current meeting.

Where technically practical, transcript segments should contain timestamps.

The transcript should be readable and editable.

The application should handle transcription errors gracefully.

Detailed transcription requirements are defined in `TRANSCRIPTION_SPECIFICATION.md`.

---

## FR-010 — Personal Notes

The user shall be able to type their own notes while the meeting is taking place.

Notes should be saved locally during the session to reduce the risk of losing information.

The user should be able to edit their notes.

Personal notes are separate from the automatically generated transcript.

---

## FR-011 — Transcript and Notes Review

Before generating the MOM, the user should be able to review:

- Transcript.
- Personal notes.

The user should be able to make corrections where necessary.

---

## FR-012 — Generate MOM

The application shall provide a clear action such as:

**Generate MOM**

The MOM-generation process shall use:

- Meeting transcript.
- User's personal notes.
- Basic meeting metadata, where available.

The application shall send only the information required for MOM generation to the configured OpenAI API.

---

## FR-013 — MOM Structure

The generated MOM should normally contain:

### Meeting Information

- Meeting title.
- Date.
- Time, where available.
- Participants, where reliably available.

### Meeting Objective / Context

A concise description of why the meeting occurred, when inferable.

### Discussion Summary

A concise summary of important discussion points.

### Key Decisions

Decisions explicitly made during the meeting.

### Action Items

For each identifiable action:

- Action.
- Owner, if identifiable.
- Due date, if stated.
- Status, if meaningful.

### Open Points / Questions

Important unresolved questions or items.

### Next Steps

Relevant follow-up activities.

The AI must not invent facts that are not supported by the transcript or user notes.

---

## FR-014 — Editable MOM

The generated MOM shall be editable before saving or exporting.

The user must be able to correct:

- Names.
- Dates.
- Actions.
- Decisions.
- Wording.
- Formatting.

---

## FR-015 — Regenerate MOM

The application may provide a regenerate option.

If regeneration is implemented, the user should be warned that manual edits to the current MOM may be overwritten.

The user should be able to choose whether to regenerate.

---

## FR-016 — Local Storage

Meeting information shall be stored locally.

At minimum, the application should preserve:

- Meeting metadata.
- Transcript.
- Personal notes.
- Generated MOM.
- Recording, if the recording is retained.

The exact file structure is defined in `LOCAL_STORAGE_SPECIFICATION.md`.

---

## FR-017 — Meeting History

The application should provide a basic list of previous meetings.

Each meeting entry should allow the user to identify:

- Meeting title.
- Date.
- Time.
- Whether a transcript exists.
- Whether a MOM exists.

The history should remain simple.

No advanced analytics are required.

---

## FR-018 — Search

A basic search capability may be included in V1 if it can be implemented without unnecessary complexity.

The initial search should be able to locate meetings by:

- Title.
- Date.
- Transcript text.
- Notes text.

Advanced semantic search is out of scope.

---

## FR-019 — Export

The user should be able to save/copy the final MOM.

At minimum:

- Copy to clipboard.
- Save as a local text/Markdown document.

A DOCX export may be included if practical.

PDF export is optional for V1 and should not delay the core application.

---

## FR-020 — Application Settings

The initial settings should be limited to useful configuration.

Possible settings include:

- OpenAI API key.
- Storage location.
- Default MOM format.
- Audio/transcription preferences.
- Application startup behavior.

Settings should not become a large administration area.

---

# 8. AI Requirements

## AI-001 — Single API Key

The application shall support one OpenAI API key.

The same API key may be used for:

- Transcription.
- MOM generation.

Different OpenAI models/capabilities may be selected for those jobs.

The application does not require separate API keys for transcription and MOM generation.

---

## AI-002 — Transcription Model

The application should use an appropriate OpenAI speech-to-text/transcription model.

The model name must be configurable in code/configuration where practical.

Do not hard-code the architecture so that changing the transcription model requires major application changes.

---

## AI-003 — MOM Model

The application should use an appropriate OpenAI text-generation model for MOM generation.

The MOM prompt should be maintained separately from UI code.

---

## AI-004 — No Hallucination

The AI-generated MOM must not invent:

- Participants.
- Decisions.
- Action owners.
- Due dates.
- Commitments.
- Meeting outcomes.

When information is unknown, the output should say it is not specified rather than inventing an answer.

---

# 9. Privacy Requirements

The application shall follow a local-first design.

The application should not upload meeting audio or meeting information unless required by the configured AI workflow.

When cloud-based OpenAI processing is used:

```text
Local audio
     ↓
OpenAI transcription
     ↓
Transcript returned
     ↓
Transcript + notes
     ↓
OpenAI MOM generation
     ↓
MOM returned
```

No general cloud storage is required.

The user should be able to delete local meeting data.

Detailed requirements are defined in `SECURITY_AND_PRIVACY.md`.

---

# 10. Performance Requirements

The application should be lightweight.

When idle:

- Minimal CPU usage.
- Minimal memory usage.
- No unnecessary network traffic.
- No audio recording.

During an active meeting:

- Audio capture should be stable.
- Transcript processing should not make the UI unusable.
- The UI should remain responsive.

MOM generation can take a short amount of time because it requires AI processing.

---

# 11. Reliability Requirements

The application should protect the user's work against common failures.

Examples:

### Application crash

Previously saved notes should not be lost.

### Network failure

The application should retain local audio/notes and clearly inform the user that AI processing cannot currently continue.

### OpenAI API failure

The user should receive a useful error and be able to retry.

### Teams closes unexpectedly

The application should detect the meeting/session state change where possible and offer to stop or preserve the current session.

### User closes application

The application should warn if an active note-taking session is in progress.

---

# 12. Security Requirements

The application must:

- Avoid exposing the API key in source code.
- Avoid committing API keys to Git.
- Use environment/configuration mechanisms appropriate for local desktop applications.
- Avoid logging the API key.
- Avoid sending unnecessary information to external services.
- Keep recordings and notes in the user's configured local storage location.

The detailed security approach is defined in `SECURITY_AND_PRIVACY.md`.

---

# 13. User Interface Requirements

The UI should be:

- Simple.
- Clean.
- Professional.
- Lightweight.
- Keyboard-friendly.
- Easy to understand without training.

The application should prioritize functionality over visual effects.

The application should not contain:

- Large analytics dashboards.
- Complicated navigation.
- Unnecessary charts.
- Excessive settings.
- Social/collaboration functionality.

Detailed UI requirements are defined in `UI_UX_SPECIFICATION.md`.

---

# 14. Out-of-Scope Requirements

The following are explicitly not V1 requirements:

### Meeting integrations

- Teams bot.
- Teams app.
- Teams meeting participant.
- Zoom.
- Google Meet.
- Webex.

### Cloud

- Cloud database.
- Cloud storage.
- Cloud synchronization.
- User account system.
- Multi-device synchronization.

### Productivity integrations

- Jira.
- Azure DevOps.
- Slack.
- Asana.
- Trello.
- Notion.
- Confluence.

### Advanced AI

- Sentiment analysis.
- Emotion detection.
- Personality analysis.
- Speaker personality profiles.
- Meeting quality scoring.
- Enterprise knowledge graph.
- Cross-company knowledge base.
- Autonomous task execution.

### Product/business features

- Subscription billing.
- Team administration.
- Organization management.
- User licensing.
- Multi-tenant backend.

---

# 15. MVP Priority

Not all requirements have equal priority.

## P0 — Must Work

These are required for the first usable build:

1. Windows desktop application.
2. Background operation.
3. Teams meeting detection.
4. Meeting notification.
5. User-controlled start.
6. Reliable Teams audio capture.
7. Recording lifecycle.
8. Transcription.
9. Personal notes.
10. Transcript + notes review.
11. MOM generation.
12. Editable MOM.
13. Local storage.
14. Basic error handling.

## P1 — Important

1. Basic meeting history.
2. Timestamped transcript.
3. Copy MOM.
4. Local Markdown/text export.
5. Application settings.
6. Automatic session recovery.

## P2 — Optional

1. DOCX export.
2. Basic search.
3. Additional UI refinements.
4. Advanced transcript editing.

P2 features must not delay P0 functionality.

---

# 16. Acceptance Criteria

V1 should not be considered complete until the following end-to-end scenario works:

### Scenario

1. User launches the application.
2. Application remains available in the background.
3. User joins a Teams meeting.
4. Application detects the meeting.
5. Application displays a notification.
6. User selects **Start Taking Notes**.
7. Application begins capturing the Teams meeting audio.
8. Other participants' voices are captured.
9. User's transmitted/unmuted voice is captured.
10. User's local speech while muted is not captured as meeting audio.
11. Transcript is produced.
12. User can enter manual notes.
13. User ends the note-taking session.
14. Application preserves transcript and notes.
15. User selects **Generate MOM**.
16. OpenAI generates a structured MOM.
17. User can edit the MOM.
18. User can copy/save the MOM.
19. Meeting information remains available locally.

---

# 17. Product Boundary

The product should stop at:

> **Transcript + Personal Notes → MOM**

The application is not intended to become a full meeting-management product.

Any proposed feature beyond this boundary should be treated as a future enhancement and must not be added to V1 automatically.

---

# 18. Guiding Principle for Development

When choosing between two implementations:

> **Choose the simplest implementation that reliably satisfies the requirement.**

Avoid introducing:

- New services.
- New databases.
- Cloud infrastructure.
- External integrations.
- Complex frameworks.

unless they are genuinely required.

The objective is to create a small, reliable desktop application—not a large software platform.
