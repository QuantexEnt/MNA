# Decisions

## Meeting Notes Assistant — V1

This document records the important product and technical decisions for V1.

The purpose is to prevent the project from repeatedly revisiting already-agreed choices and to give Claude/developers a clear boundary for implementation.

---

# Decision 001 — Product Scope

**Decision:** Build a small Windows desktop meeting notes assistant.

**Status:** Accepted

The application is not intended to become a complete meeting-management platform.

The core workflow is:

```text
Detect
  ↓
Ask
  ↓
Capture
  ↓
Note
  ↓
Transcribe
  ↓
Generate MOM
  ↓
Review
  ↓
Save
```

---

# Decision 002 — Target Platform

**Decision:** Windows only for V1.

**Status:** Accepted

Supported target:

- Windows 10
- Windows 11

Do not implement macOS or Linux support in V1.

---

# Decision 003 — Desktop Application

**Decision:** Use a Windows desktop application rather than a browser application.

**Status:** Accepted

Reason:

The application needs native access to Windows functionality, particularly:

- Teams process detection.
- Windows audio capture.
- System tray.
- Local filesystem.
- Windows application lifecycle.

---

# Decision 004 — Desktop Framework

**Decision:** Use Tauri.

**Status:** Accepted

Primary stack:

```text
Tauri
+
React
+
TypeScript
+
Rust
```

Reason:

- Lightweight desktop shell.
- Native Rust layer.
- Suitable Windows integration.
- React provides a familiar UI development model.

---

# Decision 005 — Frontend

**Decision:** React + TypeScript.

**Status:** Accepted

React owns:

- UI.
- User interactions.
- Meeting workspace.
- Notes.
- Transcript display.
- MOM editing.
- History.
- Settings.

---

# Decision 006 — Native Layer

**Decision:** Rust is responsible for Windows/native functionality.

**Status:** Accepted

Rust owns or coordinates:

- Teams detection.
- Audio capture.
- Native application state.
- System tray.
- Local filesystem operations where appropriate.
- Tauri commands/events.

React should not directly implement Windows APIs.

---

# Decision 007 — Teams Integration

**Decision:** Do not build a Teams app, Teams bot, meeting participant, or Teams add-in.

**Status:** Accepted

The application remains an external Windows desktop utility.

It detects Teams locally rather than joining or modifying the meeting.

---

# Decision 008 — Teams Meeting Detection

**Decision:** Detect Teams meeting state locally on Windows.

**Status:** Accepted

The application should distinguish:

```text
Teams not running
Teams running
Teams meeting active
Teams meeting ended
```

A running Teams process alone is not sufficient to declare that a meeting is active.

---

# Decision 009 — User Consent

**Decision:** Audio capture requires explicit user action.

**Status:** Accepted

When a meeting is detected:

```text
Teams meeting detected

Would you like to take notes?

[ Start Taking Notes ] [ Not Now ]
```

No recording should begin merely because Teams is running or a meeting has started.

---

# Decision 010 — Audio Source

**Decision:** Capture the Teams meeting audio rather than independently recording the physical microphone.

**Status:** Critical

Required behavior:

```text
Participant speech
        ↓
Captured

User unmuted in Teams
        ↓
Captured

User muted in Teams + speaks locally
        ↓
Not independently captured
```

This distinction is fundamental to the product.

---

# Decision 011 — Audio Technology

**Decision:** Prefer Windows WASAPI / Process Loopback Capture.

**Status:** Accepted direction; implementation must be validated

The preferred approach is process-specific Windows audio capture.

The implementation must prove that the chosen Teams process/session produces the desired meeting audio.

---

# Decision 012 — Audio Capture Validation

**Decision:** Audio capture is a technical gate.

**Status:** Critical

Before building the rest of the application around the audio pipeline, prove:

- Participant audio is captured.
- User's unmuted Teams speech is captured.
- User's muted local speech is not independently captured.
- Headphones work.
- Speakers work.
- Unrelated system audio is minimized/avoided where technically possible.

If this cannot be reliably achieved, document the limitation rather than silently changing the requirement.

---

# Decision 013 — Microphone Recording

**Decision:** Do not implement independent microphone recording.

**Status:** Accepted

The application must not simply record the physical microphone and call that the meeting recording.

This would violate the intended mute/unmute behavior.

---

# Decision 013-A — Gated Microphone Capture (Amendment to Decision 013)

**Decision:** Physical microphone capture is now permitted, but only as a
secondary source, strictly gated by a verified real-time "Teams currently
allows transmission" signal, and always combined with the existing
device-loopback capture — never used alone, never unconditional.

**Status:** Accepted

This amendment clarifies, rather than reverses, Decision 013. The
original prohibition — do not treat the microphone as a primary,
unconditional, independent recording source — still stands. What changes
is that a conditionally-gated secondary use is now permitted, because a
gap in the original architecture was found empirically.

---

## Background / Problem

Empirical testing (Audio Capture POC + Mute Detection POC, see project
chat log) proved three things that the original architecture did not
anticipate:

1. Device-level WASAPI loopback correctly captures OTHER participants,
   but structurally CANNOT capture the user's OWN voice at all —
   muted or not. Windows/Teams never render a user's own microphone
   back to their own output device (no self-echo). This satisfies the
   "muted speech not captured" half of Decision 010, but fails the
   "unmuted speech must be captured" half.

2. Process-specific loopback (the originally planned enhancement, later
   deferred for unrelated crate-compile reasons) has the IDENTICAL
   limitation. Loopback — device or process level — only ever exposes
   render/output streams, never a process's outbound/network audio.
   Confirmed via research and code, not assumption. Finishing that
   deferred code would not have solved this gap either.

3. The only technically real way to capture the user's own contribution
   to a meeting is the physical microphone input endpoint.

## Real-time mute-state signal established

- Windows OS-level mic-endpoint mute is INDEPENDENT of Teams' own
  in-call mute. Confirmed: toggling Teams' own mute button does not
  change the OS endpoint mute state at all.
- Teams' own UI-exposed mute button (readable via Windows UI Automation)
  DOES reliably reflect Teams' actual transmit state, via its accessible
  name text ("Mute mic" when currently unmuted / transmitting, "Unmute
  mic" when currently muted).
- This UI signal ALSO correctly reflects OS/hardware-level mute: toggling
  a laptop's hardware mic-mute key changes both the OS endpoint AND
  Teams' UI text in sync - Teams appears to listen for and mirror the OS
  mute state into its own display.
- Therefore: **the Teams UI Automation mute-button text is the most
  complete available signal** for "is the user's mic currently allowed
  to transmit," covering both Teams-native and OS/hardware-triggered
  mute paths.

## Tested scenarios and results

| Mute method | UI signal reliable? | Underlying audio actually silenced? |
|---|---|---|
| Teams' own mute button | Yes - clean, ~1s detection lag | N/A (Teams handles internally) |
| Laptop hardware mute key | Yes - Teams auto-syncs to it | Yes (OS-level mute) |
| Bluetooth headset hardware button (one device tested) | No - inconsistent/flaky | Yes - confirmed via direct raw-mic diagnostic recording; audio was hardware-silenced regardless of what any software signal showed |

## New architecture

```
Device-level WASAPI loopback              (other participants)
        +
Physical microphone input                  (user's own voice)
        gated by: Teams UI Automation mute-button text
        - write mic audio into the recording ONLY while the button
          reads "Mute mic" (= currently unmuted / transmitting)
        - write silence (or nothing) while it reads "Unmute mic"
        ↓
Combined meeting recording
```

## Known limitations (documented, not hidden — per this project's own
Known Technical Limitation Policy)

- Validated against ONE installed Teams build and ONE Bluetooth headset.
  A future Teams UI update could change the button's accessible name or
  structure, silently breaking detection. This should be re-verified
  after any Teams update, not assumed permanent.
- The Bluetooth-headset safety margin observed here (hardware-level
  audio silencing) is a property of that specific device, not a
  guarantee for all Bluetooth headsets. A headset whose mute is purely a
  cosmetic LED, with real audio still flowing to Windows, would NOT be
  protected by this design — the UI signal alone was shown to be
  unreliable for at least one real device.
- There is a real, unmeasured lag between an actual mute action and this
  detector catching up (testing used 500ms-1000ms polling). A brief
  window of audio right at the mute transition could theoretically be
  captured. The acceptable bound for V1 needs an explicit decision
  during implementation (e.g. faster polling, and/or discarding a short
  buffer immediately around every detected transition).
- This remains SECONDARY to and dependent on the gating signal. If the
  UI Automation signal cannot be obtained at all (e.g. Teams window not
  found, automation call fails), the correct failure mode is to STOP
  writing microphone audio, not default to capturing it. Fail closed,
  never fail open.

## Impact on other decisions

- **Decision 010 (Audio Source):** now fully satisfiable via this
  dual-source design. Previously only satisfiable via loopback alone,
  which we now know cannot capture the user's own voice under any
  condition.
- **Decision 012 (Audio Capture Validation):** gains a required proof
  point specific to mute-state detection reliability and lag, in
  addition to the original loopback validation.
- **Decision 013 (Microphone Recording):** clarified, not reversed.
  Independent/unconditional microphone recording remains prohibited.
  Conditionally-gated microphone recording, continuously verified
  against real-time meeting mute state, is now permitted as a secondary
  source only.

## Decision Change Process followed (per §61)

1. Current decision: identified (Decision 013, "do not implement
   independent microphone recording").
2. Problem explained: device/process loopback cannot capture the user's
   own voice under any circumstances (Section "Background/Problem"
   above).
3. Proposed alternative explained: gated dual-source capture (Section
   "New architecture" above).
4. Impact explained: Section "Impact on other decisions" above.
5. Alternative tested: yes — Audio Capture POC + Mute Detection POC,
   including a direct raw-microphone diagnostic recording to verify
   ground truth on the Bluetooth headset case.
6. Document updated: this entry (pending sign-off).
7. Implementation: NOT YET STARTED — waiting for explicit approval of
   this document first, per project convention.

---

# Decision 014 — Headphones

**Decision:** Headsets/headphones must be supported.

**Status:** Required

The implementation should not depend on laptop speakers being active.

---

# Decision 015 — Local-First Architecture

**Decision:** Store meeting information locally.

**Status:** Accepted

The application should not require a cloud backend for V1.

Local data includes:

- Audio.
- Transcript.
- Notes.
- MOM.
- Meeting metadata.
- Settings.

---

# Decision 016 — No Cloud Backend

**Decision:** No application backend/server for V1.

**Status:** Accepted

Do not build:

- VPS.
- Hostinger server.
- API backend.
- Cloud database.
- Cloud storage service.

The application is primarily local.

---

# Decision 017 — Database

**Decision:** Do not use SQLite or another database for V1 unless a demonstrated requirement appears.

**Status:** Accepted

Use:

```text
Folders
+
JSON
+
Markdown
+
Audio files
```

Reason:

The expected V1 data volume is manageable without database complexity.

A database can be introduced later if history/search requirements justify it.

---

# Decision 018 — Local File Structure

**Decision:** Each meeting gets its own directory.

**Status:** Accepted

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

This keeps data understandable and portable.

---

# Decision 019 — Human-Readable Data

**Decision:** Use human-readable Markdown where practical.

**Status:** Accepted

Meeting notes, transcript and MOM should be accessible as ordinary local files.

---

# Decision 020 — Auto-Save

**Decision:** Manual notes should auto-save.

**Status:** Accepted

The application should minimize the chance of losing notes due to:

- Crash.
- Restart.
- Unexpected meeting end.

---

# Decision 021 — Transcript Timing

**Decision:** Post-meeting transcription is the V1 baseline.

**Status:** Accepted

The first reliable workflow is:

```text
Meeting
 ↓
Audio file
 ↓
Transcription after meeting
```

Near-live transcription may be added later.

---

# Decision 022 — Live Transcript

**Decision:** Near-live transcription is optional and not required for V1 acceptance.

**Status:** Deferred

Do not allow live transcription complexity to delay the core product.

---

# Decision 023 — OpenAI

**Decision:** Use OpenAI API for AI functionality.

**Status:** Accepted

OpenAI is used for:

```text
Audio → Transcript
Transcript + Notes → MOM
```

---

# Decision 024 — OpenAI API Key

**Decision:** One OpenAI API key is sufficient for the application.

**Status:** Accepted

The same API key can authorize both:

```text
Transcription model
+
MOM generation model
```

Different models can be configured for each function.

---

# Decision 025 — ChatGPT Subscription vs API

**Decision:** Treat ChatGPT and OpenAI API billing as separate services.

**Status:** Accepted

Having a ChatGPT subscription does not automatically mean the desktop application can use the API.

The application requires valid API access and API billing/credits as applicable.

---

# Decision 026 — API Key Storage

**Decision:** Never commit the OpenAI API key to Git.

**Status:** Critical

Development should support environment/configuration-based setup.

The key must not appear in:

- Source code.
- Git history.
- Public repositories.
- Logs.
- Screenshots.
- Test data.

For future commercial distribution, a personal API key must not be embedded into the distributed application.

---

# Decision 027 — Transcription Service

**Decision:** Keep transcription behind a dedicated service abstraction.

**Status:** Accepted

Conceptually:

```text
TranscriptionService
        ↓
OpenAI transcription API
```

This allows the transcription provider/model to change without rewriting the application.

---

# Decision 028 — MOM Service

**Decision:** Keep MOM generation behind a dedicated service abstraction.

**Status:** Accepted

Conceptually:

```text
MOMService
        ↓
OpenAI generation API
```

---

# Decision 029 — MOM Input

**Decision:** MOM generation uses both transcript and user notes.

**Status:** Accepted

```text
Transcript
+
Manual Notes
        ↓
MOM
```

Manual notes are treated as important user-provided context.

---

# Decision 030 — MOM Is a Draft

**Decision:** AI-generated MOM is always editable.

**Status:** Accepted

The user remains the final authority.

The application must not present AI-generated meeting minutes as automatically authoritative.

---

# Decision 031 — Hallucination Control

**Decision:** The AI must not invent meeting commitments.

**Status:** Critical

The system should distinguish:

```text
Decision
Action
Suggestion
Discussion
Open Question
```

It should not invent:

- Participants.
- Owners.
- Deadlines.
- Decisions.
- Commitments.

---

# Decision 032 — Unknown Information

**Decision:** Unknown information should remain unknown.

**Status:** Accepted

Example:

If an action exists but no owner is established:

```text
Owner: Unassigned
```

Do not guess.

---

# Decision 033 — Speaker Identification

**Decision:** Do not fabricate speaker identity.

**Status:** Accepted

If reliable speaker identification is unavailable, use generic labels or omit speaker labels.

---

# Decision 034 — Psychological Analysis

**Decision:** Do not perform psychological analysis of meeting participants.

**Status:** Explicitly out of scope

The application should not infer:

- Personality.
- Emotion.
- Mental state.
- Motivation.
- Psychological traits.

The product is a meeting notes tool, not a psychological analysis system.

---

# Decision 035 — Meeting History

**Decision:** Provide basic local meeting history.

**Status:** Accepted

History should initially support:

- Date.
- Meeting title.
- Duration.
- Open meeting.
- View notes.
- View transcript.
- View MOM.

No advanced analytics are required.

---

# Decision 036 — Settings

**Decision:** Keep settings minimal.

**Status:** Accepted

Initial settings:

- OpenAI API key.
- Transcription model.
- MOM model.
- Storage location.
- Start with Windows.
- Meeting notification.

---

# Decision 037 — System Tray

**Decision:** Application should support quiet background operation through the Windows system tray.

**Status:** Accepted

The user should not need to keep the main window open during normal use.

---

# Decision 038 — No Hidden Recording

**Decision:** The application must never silently record meetings.

**Status:** Critical

Recording state must be visible.

The user should always be able to determine:

```text
Recording: ON
```

or:

```text
Recording: OFF
```

---

# Decision 039 — No Telemetry by Default

**Decision:** Do not add product analytics or tracking to V1.

**Status:** Accepted

No unnecessary collection of:

- Usage analytics.
- Meeting content.
- Transcript content.
- Audio.
- User behavior.

---

# Decision 040 — OpenAI Data Flow

**Decision:** Only data required for AI processing should leave the local machine.

**Status:** Accepted

The normal flow is:

```text
Local Audio
     ↓
OpenAI transcription
     ↓
Transcript
     ↓
Local
     ↓
OpenAI MOM generation
     ↓
MOM
     ↓
Local
```

Other application data should remain local.

---

# Decision 041 — No Calendar Integration

**Decision:** Do not integrate with Outlook/Google Calendar in V1.

**Status:** Out of scope

Meeting detection should work independently of calendar data.

---

# Decision 042 — No Zoom/Google Meet/Webex

**Decision:** Microsoft Teams only for V1.

**Status:** Out of scope

Other meeting platforms may be considered later.

---

# Decision 043 — No Browser Extension

**Decision:** Do not build a browser extension.

**Status:** Out of scope

---

# Decision 044 — No Mobile App

**Decision:** No mobile application for V1.

**Status:** Out of scope

---

# Decision 045 — No Collaboration

**Decision:** V1 is a single-user local application.

**Status:** Accepted

No:

- Multi-user accounts.
- Shared workspaces.
- Team collaboration.
- Cloud collaboration.

---

# Decision 046 — No Subscription System

**Decision:** Do not implement licensing/subscriptions/payment functionality in V1.

**Status:** Out of scope

---

# Decision 047 — No Enterprise Backend

**Decision:** Do not design V1 around enterprise server infrastructure.

**Status:** Accepted

Future enterprise requirements can be addressed separately if the product evolves.

---

# Decision 048 — Local Data Retention

**Decision:** The user controls local meeting data retention.

**Status:** Accepted

The application should make it possible to delete meeting data locally.

Future versions may provide automatic retention rules.

---

# Decision 049 — Audio Retention

**Decision:** Keep audio available in V1 unless the user deletes it.

**Status:** Accepted

Audio is useful for:

- Re-transcription.
- Debugging.
- Verification.

Automatic deletion can be added later as an explicit setting.

---

# Decision 050 — Re-Transcription

**Decision:** Preserve audio so a meeting can be transcribed again if needed.

**Status:** Accepted

Example:

```text
Existing Audio
      ↓
New transcription model
      ↓
New Transcript
```

This avoids losing the original source recording.

---

# Decision 051 — AI Mock Mode

**Decision:** Development should support mock AI responses.

**Status:** Recommended

Example:

```text
OPENAI_MOCK=true
```

This allows UI and workflow development without consuming API credits.

---

# Decision 052 — Cost Control

**Decision:** Use short recordings during development.

**Status:** Accepted

Development should normally use:

```text
2–5 minute recordings
```

before testing long meetings.

---

# Decision 053 — Dependency Philosophy

**Decision:** Keep dependencies minimal.

**Status:** Accepted

Before adding a dependency, determine whether:

- Windows already provides the capability.
- Tauri provides the capability.
- An existing dependency can solve it.
- The dependency is truly necessary.

---

# Decision 054 — No Premature Optimization

**Decision:** Do not optimize for large-scale enterprise workloads in V1.

**Status:** Accepted

Optimize for:

```text
Correctness
Reliability
Simplicity
Privacy
```

---

# Decision 055 — Audio Capture Is the Highest-Risk Component

**Decision:** Prove audio capture before investing heavily in the remaining product.

**Status:** Critical

The implementation plan therefore prioritizes:

```text
Audio POC
      ↓
Validation
      ↓
Integration
```

---

# Decision 056 — Implementation Style

**Decision:** Implement incrementally with small, testable changes.

**Status:** Accepted

Claude/developers should avoid requests such as:

```text
Build the entire application.
```

Prefer:

```text
Implement one defined phase.
Run tests.
Validate.
Continue.
```

---

# Decision 057 — Specification-Driven Development

**Decision:** Existing project documentation is the source of truth for V1 behavior.

**Status:** Accepted

Relevant specifications should be read before implementation.

When documents conflict, update the decision record rather than silently choosing a different behavior.

---

# Decision 058 — Scope Protection

**Decision:** Features outside V1 must not be added automatically.

**Status:** Accepted

Examples:

```text
Calendar
Zoom
Cloud sync
Integrations
Analytics
Bots
Collaboration
Mobile
```

remain deferred unless the V1 scope is explicitly changed.

---

# Decision 059 — Database Later, If Needed

**Decision:** A database may be introduced in a future version if local file storage becomes insufficient.

**Status:** Deferred

Possible triggers:

- Large meeting history.
- Fast full-text search.
- Advanced filtering.
- Structured analytics.
- Large-scale indexing.

Until then:

```text
Files are enough.
```

---

# Decision 060 — Cloud Architecture Later, If Needed

**Decision:** Cloud architecture may be considered only if future requirements justify it.

**Status:** Deferred

Possible future requirements:

- Multi-device synchronization.
- Team sharing.
- Enterprise administration.
- Centralized storage.
- Collaboration.

None are V1 requirements.

---

# 61. Decision Change Process

If a developer believes an existing decision should change:

1. Identify the current decision.
2. Explain the problem.
3. Explain the proposed alternative.
4. Explain the impact.
5. Test the alternative where practical.
6. Update this document.
7. Only then implement the changed direction.

Do not silently override an accepted decision.

---

# 62. Current Architecture Summary

The current agreed architecture is:

```text
                 Windows
                    │
             ┌──────▼──────┐
             │    Tauri    │
             │   Desktop   │
             └──────┬──────┘
                    │
          ┌─────────┴─────────┐
          │                   │
     React/TS             Rust
          │                   │
       UI/UX          ┌───────┼────────┐
                      │       │        │
                   Teams    Audio    Storage
                   Detect   Capture   Files
                      │       │
                      └───┬───┘
                          │
                     Local Session
                          │
                 ┌────────┴────────┐
                 │                 │
             Transcription       Notes
                 │                 │
                 └────────┬────────┘
                          │
                     MOM Generation
                          │
                          ▼
                    Editable MOM
```

---

# 63. Current V1 Boundary

The product boundary is:

```text
Windows
Teams
Meeting Audio
Manual Notes
Transcription
MOM
Local Storage
OpenAI
```

Everything else is secondary.

---

# 64. Final Product Principle

The application should remain:

> **Small enough to understand, reliable enough to trust, and useful enough to use after every Teams meeting.**

Do not turn a simple meeting-notes assistant into a large meeting-management platform before the core workflow is proven.
