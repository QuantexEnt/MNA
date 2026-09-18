# Testing and Acceptance

## Meeting Notes Assistant — V1

**Purpose:** Define how the application is tested and what must be true before V1 is considered complete.

---

# 1. Testing Objective

Testing must prove that the complete workflow works reliably on Windows:

```text
Teams Meeting
    ↓
Detection
    ↓
User Consent
    ↓
Audio Capture
    ↓
Manual Notes
    ↓
Meeting End
    ↓
Transcription
    ↓
MOM Generation
    ↓
Review / Edit
    ↓
Local Save
```

The most important test area is **Teams meeting audio capture**.

---

# 2. Acceptance Philosophy

A feature is not complete merely because:

- The code compiles.
- The UI appears.
- A test with artificial data passes.

A feature is complete when the intended user behavior works in the real Windows environment.

---

# 3. Test Environment

Minimum target:

```text
Windows 10/11
Microsoft Teams
Working internet connection
Working audio output
Working microphone
Headphones/headset available
OpenAI API access
```

Recommended:

- Windows 11
- Teams desktop application
- Wired or Bluetooth headset
- Second participant/device for meeting tests

---

# 4. Test Data

Use two types of test data.

## Synthetic data

For:

- UI testing.
- MOM testing.
- Error testing.
- Transcript testing.

## Real meeting data

For:

- Teams detection.
- Audio capture.
- Mute/unmute behavior.
- End-to-end validation.

Never commit confidential real meeting recordings or transcripts to Git.

---

# 5. Test Priority

| Priority | Meaning |
|---|---|
| P0 | Must pass before V1 |
| P1 | Should pass before release |
| P2 | Nice to have |

Any P0 failure blocks V1 release.

---

# 6. Test Categories

The project should test:

1. Application startup.
2. System tray.
3. Teams detection.
4. Meeting notification.
5. User consent.
6. Audio capture.
7. Session lifecycle.
8. Manual notes.
9. Local storage.
10. Transcription.
11. MOM generation.
12. MOM editing.
13. Meeting history.
14. Settings.
15. Error handling.
16. Recovery.
17. Security/privacy.
18. Performance.
19. Packaging/install.
20. End-to-end workflow.

---

# 7. Application Startup Tests

## TC-001 — First Launch

**Steps**

1. Launch application.
2. Wait for startup.

**Expected**

- Application starts.
- No crash.
- Main UI renders.
- Tray icon appears if tray is enabled.
- No meeting is recorded automatically.

**Priority:** P0

---

## TC-002 — Restart

**Steps**

1. Close application.
2. Launch again.

**Expected**

- Application starts normally.
- Previous local meeting data remains available.
- Settings remain available.

**Priority:** P0

---

## TC-003 — Start with Windows

If enabled:

**Expected**

- Application starts with Windows.
- It remains quiet in the background.
- It does not automatically record.

**Priority:** P1

---

# 8. System Tray Tests

## TC-010 — Minimize to Tray

**Steps**

1. Launch application.
2. Minimize/close the main window.

**Expected**

- Application continues running.
- Tray icon remains.
- No meeting session is affected.

**Priority:** P1

---

## TC-011 — Restore from Tray

**Expected**

- Main window reopens.
- Current state is preserved.

**Priority:** P1

---

## TC-012 — Exit from Tray

**Expected**

- Application shuts down.
- Active capture is handled safely.
- Files are flushed.
- No corrupt meeting state is created.

**Priority:** P0

---

# 9. Teams Detection Tests

## TC-020 — Teams Closed

**Expected**

```text
No meeting detected
No prompt
No audio capture
```

**Priority:** P0

---

## TC-021 — Teams Open but Idle

**Expected**

```text
Teams detected
Meeting not detected
No prompt
No audio capture
```

**Priority:** P0

---

## TC-022 — Meeting Starts

**Steps**

1. Start Teams.
2. Join a test meeting.

**Expected**

```text
Meeting detected
```

A notification should appear according to the defined notification behavior.

**Priority:** P0

---

## TC-023 — Meeting Ends

**Expected**

- Meeting state changes to inactive.
- Active session is handled correctly.
- Capture does not continue indefinitely.

**Priority:** P0

---

## TC-024 — Duplicate Detection

**Steps**

Allow Teams state polling/events to detect the same meeting multiple times.

**Expected**

- User is not repeatedly prompted.
- Only one active session exists.

**Priority:** P0

---

# 10. Consent Tests

## TC-030 — User Selects Not Now

**Steps**

1. Meeting detected.
2. Select **Not Now**.

**Expected**

- No meeting audio capture starts.
- No recording file is created for the declined session.
- User may continue the meeting normally.

**Priority:** P0

---

## TC-031 — User Selects Start Taking Notes

**Expected**

- Meeting session starts.
- Recording indicator appears.
- Manual notes become available.
- Audio capture begins.

**Priority:** P0

---

# 11. Audio Capture Tests

This is the most important test group.

---

## TC-040 — Participant Audio

**Steps**

1. Start a Teams meeting.
2. Another participant speaks.
3. Stop recording.
4. Play the captured audio.

**Expected**

Participant speech is present.

**Priority:** P0

---

## TC-041 — User Unmuted

**Steps**

1. User is unmuted in Teams.
2. User speaks.
3. Stop recording.
4. Play audio.

**Expected**

User's transmitted meeting speech is captured.

**Priority:** P0

---

## TC-042 — User Muted

**Steps**

1. User mutes themselves in Teams.
2. User speaks locally.
3. Stop recording.
4. Play captured audio.

**Expected**

The application must not independently record the physical microphone and include the muted local speech as meeting audio.

**Priority:** P0

---

## TC-043 — Headphones

**Steps**

1. Connect headset/headphones.
2. Join meeting.
3. Start capture.
4. Participant speaks.

**Expected**

Participant audio is captured.

**Priority:** P0

---

## TC-044 — Laptop Speakers

**Expected**

Meeting audio is captured correctly.

**Priority:** P1

---

## TC-045 — Unrelated System Audio

**Steps**

1. Play unrelated audio from another application.
2. Run Teams meeting capture.

**Expected**

The captured stream should contain only the intended Teams meeting audio, to the extent supported by the chosen Windows capture method.

If unrelated audio is captured, document the limitation and investigate process/session isolation.

**Priority:** P0

---

## TC-046 — Audio Quality

**Expected**

- Speech is intelligible.
- No unexpected clipping.
- No continuous silence.
- File is playable.
- Duration approximately matches the capture session.

**Priority:** P0

---

## TC-047 — Start/Stop

**Expected**

- Capture begins only after Start.
- Capture stops when requested.
- File is finalized correctly.
- No audio continues after stop.

**Priority:** P0

---

## TC-048 — Long Meeting

Test:

```text
30 minutes
60 minutes
```

Expected:

- No memory growth that makes the application unusable.
- Audio remains valid.
- File remains playable.
- Processing can complete.

**Priority:** P1

---

# 12. Audio Failure Tests

## TC-050 — No Audio Device

**Expected**

- Clear error.
- Application does not crash.
- User can continue with notes if possible.

**Priority:** P0

---

## TC-051 — Audio Capture Initialization Failure

**Expected**

- Error is shown.
- Session state is consistent.
- No corrupted recording is presented as valid.

**Priority:** P0

---

## TC-052 — Capture Stops Unexpectedly

**Expected**

- Application detects failure.
- User is informed.
- Already captured audio is preserved.
- Notes remain available.

**Priority:** P0

---

# 13. Meeting Session Tests

## TC-060 — Session Creation

**Expected**

A unique meeting/session ID is created.

**Priority:** P0

---

## TC-061 — Session End

**Expected**

The session records:

```text
start time
end time
duration
recording status
```

**Priority:** P0

---

## TC-062 — Teams Ends Unexpectedly

**Expected**

- Application detects meeting end where possible.
- Capture is stopped safely.
- Session is saved.
- Processing can continue.

**Priority:** P0

---

# 14. Manual Notes Tests

## TC-070 — Enter Notes

**Expected**

- User can type freely.
- Notes are readable.
- Transcript is not modified.

**Priority:** P0

---

## TC-071 — Auto-save

**Steps**

1. Type notes.
2. Wait.
3. Close/restart application.

**Expected**

Notes remain available.

**Priority:** P0

---

## TC-072 — Long Notes

**Expected**

Large notes do not cause UI instability or data loss.

**Priority:** P1

---

# 15. Local Storage Tests

## TC-080 — Meeting Directory

**Expected**

A meeting creates its own local directory.

**Priority:** P0

---

## TC-081 — Metadata

`meeting.json` should contain sufficient metadata to reopen the meeting.

**Priority:** P0

---

## TC-082 — Audio Persistence

**Expected**

Saved audio remains playable after application restart.

**Priority:** P0

---

## TC-083 — Transcript Persistence

**Expected**

Transcript can be reopened without calling OpenAI again.

**Priority:** P0

---

## TC-084 — MOM Persistence

**Expected**

Saved MOM remains editable/reviewable after restart.

**Priority:** P0

---

## TC-085 — Corrupt/Incomplete File

**Expected**

Application reports the affected meeting/file clearly without crashing the entire application.

**Priority:** P1

---

# 16. Transcription Tests

## TC-090 — Valid Audio

**Expected**

Audio is submitted successfully and a transcript is returned.

**Priority:** P0

---

## TC-091 — Transcript Accuracy

Use a test recording containing:

- Normal conversation.
- Technical terms.
- Numbers.
- Dates.
- Names.
- Acronyms.

**Expected**

Speech is transcribed with useful accuracy.

**Priority:** P0

---

## TC-092 — Timestamps

**Expected**

Where timestamps are provided, they remain correctly associated with transcript segments.

**Priority:** P1

---

## TC-093 — Speaker Labels

**Expected**

The application does not fabricate speaker identities.

**Priority:** P0

---

## TC-094 — Long Audio

**Expected**

Long recordings are handled through the defined chunking/processing strategy if required.

**Priority:** P1

---

## TC-095 — OpenAI Failure

Simulate:

- No internet.
- Invalid API key.
- API error.
- Timeout.

**Expected**

- Useful error.
- Audio preserved.
- Notes preserved.
- User can retry later.

**Priority:** P0

---

# 17. MOM Generation Tests

## TC-100 — Basic MOM

Provide a synthetic transcript and notes.

**Expected**

A structured MOM is generated.

**Priority:** P0

---

## TC-101 — Decisions

If transcript contains an explicit decision:

**Expected**

It appears under Decisions.

**Priority:** P0

---

## TC-102 — Action Items

If transcript contains an explicit commitment:

**Expected**

The action appears under Action Items.

**Priority:** P0

---

## TC-103 — Avoid False Actions

Input:

```text
"We could look at this next week."
```

**Expected**

It should not automatically become a committed action item.

**Priority:** P0

---

## TC-104 — Unknown Owner

If an action is clearly agreed but owner is not known:

**Expected**

Owner is shown as:

```text
Unassigned
```

or equivalent.

Do not invent a person.

**Priority:** P0

---

## TC-105 — Missing Information

**Expected**

The MOM clearly indicates unknown/missing information instead of fabricating it.

**Priority:** P0

---

# 18. MOM Editing Tests

## TC-110 — Edit MOM

**Expected**

User can change generated text.

**Priority:** P0

---

## TC-111 — Save Edited MOM

**Expected**

User edits remain after reopening the meeting.

**Priority:** P0

---

## TC-112 — Regenerate MOM

**Expected**

Regeneration is explicit.

Existing user edits must not be silently destroyed.

**Priority:** P0

---

## TC-113 — Copy MOM

**Expected**

Complete MOM can be copied to clipboard.

**Priority:** P1

---

# 19. History Tests

## TC-120 — Meeting List

**Expected**

Previously saved meetings appear.

**Priority:** P1

---

## TC-121 — Open Meeting

**Expected**

User can reopen:

```text
Notes
Transcript
MOM
```

**Priority:** P1

---

# 20. Settings Tests

## TC-130 — API Key

**Expected**

User can configure the API key without exposing it in normal UI/logs.

**Priority:** P0

---

## TC-131 — Invalid API Key

**Expected**

Connection test reports failure clearly.

**Priority:** P0

---

## TC-132 — Storage Location

**Expected**

Changing the storage location does not silently lose existing meetings.

**Priority:** P1

---

# 21. Privacy Tests

## TC-140 — No Automatic Recording

**Expected**

Application never starts meeting audio capture merely because Teams is running or a meeting has started.

**Priority:** P0

---

## TC-141 — No Microphone Recording

**Expected**

The application does not create an independent microphone recording.

**Priority:** P0

---

## TC-142 — Local Storage

**Expected**

Meeting files remain local except for explicitly required OpenAI API processing.

**Priority:** P0

---

## TC-143 — Logs

**Expected**

Logs do not contain:

- Full meeting transcripts.
- Full audio.
- API keys.
- Sensitive meeting content.

**Priority:** P0

---

# 22. Security Tests

Verify:

```text
[ ] API key not committed
[ ] .env excluded
[ ] Path traversal prevented
[ ] Invalid configuration rejected
[ ] Unexpected API responses handled
[ ] Temporary files cleaned
[ ] No debug secrets in release
```

**Priority:** P0

---

# 23. Performance Tests

The application should remain responsive during:

- Teams detection.
- Audio capture.
- Manual note entry.
- Transcription.
- MOM generation.

The UI must not freeze while AI processing is occurring.

**Priority:** P1

---

# 24. Offline Tests

## TC-150 — Offline During Meeting

**Expected**

If the application is already capturing:

- Audio remains local.
- Notes remain usable.
- Meeting can finish normally.

**Priority:** P0

---

## TC-151 — Offline During Transcription

**Expected**

- Transcription fails gracefully.
- Audio remains available.
- User can retry when online.

**Priority:** P0

---

# 25. Crash Recovery Tests

## TC-160 — Application Crash During Meeting

Simulate application termination while recording.

**Expected**

- Recoverable meeting data remains.
- Already-written notes are preserved.
- Application can identify an incomplete session.

**Priority:** P0

---

## TC-161 — Restart After Crash

**Expected**

User is given a clear recovery path.

Possible state:

```text
Incomplete meeting found.
Continue processing?
```

**Priority:** P1

---

# 26. Sleep/Resume Test

## TC-170 — Windows Sleep

If practical, test:

```text
Meeting active
 ↓
Windows sleep
 ↓
Resume
```

Expected behavior should be documented.

If audio capture cannot safely continue across sleep, the application must fail clearly rather than silently producing misleading output.

**Priority:** P1

---

# 27. Multiple Meeting Tests

## TC-180 — Consecutive Meetings

Test:

```text
Meeting A
 ↓
Meeting ends
 ↓
Meeting B
```

Expected:

- Separate session IDs.
- Separate audio.
- Separate notes.
- Separate transcript.
- Separate MOM.

**Priority:** P0

---

# 28. Duplicate/Repeated Events

Windows and Teams state changes may generate repeated events.

Expected:

```text
One real meeting
=
One active session
```

No duplicate recordings.

**Priority:** P0

---

# 29. UI Acceptance

The application should be:

- Simple.
- Quiet.
- Understandable.
- Responsive.
- Keyboard-friendly.
- Clear about recording state.

The user should always know:

```text
Am I recording?
Is Teams detected?
Is processing happening?
Is my data saved?
```

---

# 30. Accessibility Acceptance

Minimum:

- Keyboard navigation.
- Readable text.
- Visible focus state.
- Clear button labels.
- No information communicated by color alone.

---

# 31. Installer Tests

Before release:

## TC-190 — Fresh Install

**Expected**

Application installs successfully on a clean supported Windows environment.

**Priority:** P0

---

## TC-191 — Launch After Install

**Expected**

Application launches without requiring development tools.

**Priority:** P0

---

## TC-192 — Upgrade

If an existing version exists:

**Expected**

Existing meeting data is preserved.

**Priority:** P1

---

## TC-193 — Uninstall

**Expected**

Application binaries are removed.

User meeting data should not be silently deleted unless the user explicitly chooses to remove it.

**Priority:** P1

---

# 32. Release Regression Test

Before every release, rerun all P0 tests.

At minimum:

```text
Startup
Teams detection
Consent
Audio capture
Mute/unmute
Headphones
Session lifecycle
Notes
Storage
Transcription
MOM
MOM editing
Recovery
Privacy
Security
Installer
```

---

# 33. P0 Release Checklist

```text
[ ] Application starts
[ ] Teams meeting detection works
[ ] User must explicitly start capture
[ ] Participant audio captured
[ ] User unmuted speech captured
[ ] User muted local speech not independently captured
[ ] Headphones work
[ ] Audio file is valid
[ ] Meeting session saved
[ ] Manual notes saved
[ ] Transcription works
[ ] MOM generation works
[ ] MOM does not invent commitments
[ ] MOM is editable
[ ] Saved MOM survives restart
[ ] OpenAI failures are recoverable
[ ] Crash does not unnecessarily lose notes/audio
[ ] API key protected
[ ] No meeting content in logs
[ ] No unexpected cloud storage
[ ] Installer works
```

---

# 34. P1 Checklist

```text
[ ] System tray polished
[ ] Meeting history works
[ ] Settings polished
[ ] Long meeting tested
[ ] Sleep/resume behavior documented
[ ] Accessibility reviewed
[ ] Upgrade tested
```

---

# 35. Defect Severity

## Critical

Blocks core workflow or creates privacy/security risk.

Examples:

- Wrong audio captured.
- Muted microphone independently recorded.
- Recording starts without consent.
- Data loss.
- API key exposed.

**Must fix before release.**

---

## High

Major feature does not work but no immediate privacy/security risk.

Examples:

- Transcription consistently fails.
- MOM cannot be generated.
- Meeting cannot be reopened.

**Must fix before V1.**

---

## Medium

Feature works with limitations.

Examples:

- History display issue.
- Minor recovery problem.
- Non-critical UI issue.

Can be evaluated for release.

---

## Low

Cosmetic or minor usability issue.

Examples:

- Spacing.
- Icon alignment.
- Minor wording.

May be deferred.

---

# 36. Acceptance Sign-Off

V1 should only be considered accepted when:

```text
P0 tests = PASS
Critical defects = 0
High defects affecting core workflow = 0
```

Known limitations must be documented.

---

# 37. Known Technical Limitation Policy

If Windows/Teams behavior prevents a requirement from being fully guaranteed:

1. Do not hide the limitation.
2. Document the observed behavior.
3. Identify affected scenarios.
4. Determine whether a technical alternative exists.
5. Do not silently replace the agreed audio-capture requirement with microphone recording.
6. Mark the requirement as unresolved until technically validated.

---

# 38. Final End-to-End Acceptance Scenario

Use a real Teams test meeting.

### Step 1

Application is running in background.

Expected:

```text
No recording
```

### Step 2

Teams meeting begins.

Expected:

```text
Meeting detected
Prompt displayed
```

### Step 3

Select:

```text
Start Taking Notes
```

Expected:

```text
Recording active
Meeting workspace visible
```

### Step 4

Participant speaks.

Expected:

```text
Participant audio captured
```

### Step 5

User unmutes and speaks.

Expected:

```text
User speech captured
```

### Step 6

User mutes and speaks locally.

Expected:

```text
Local muted speech is not independently captured
```

### Step 7

User writes notes.

Expected:

```text
Notes auto-save
```

### Step 8

Meeting ends.

Expected:

```text
Capture stops
Audio saved
Session saved
```

### Step 9

Transcription runs.

Expected:

```text
Transcript generated
```

### Step 10

MOM generation runs.

Expected:

```text
MOM generated
```

### Step 11

User edits MOM.

Expected:

```text
Changes saved
```

### Step 12

Application restarts.

Expected:

```text
Meeting remains available
Notes remain
Transcript remains
MOM remains
```

This scenario is the primary V1 acceptance test.

---

# 39. Final Quality Principle

The application should optimize for:

```text
Correctness
    >
Reliability
    >
Privacy
    >
Simplicity
    >
Visual polish
    >
Extra features
```

The most important outcome is not that the application has many features.

It is that the core workflow can be trusted:

> **When the user intentionally starts taking notes in a Teams meeting, the application captures the intended meeting audio, preserves their notes, produces a useful transcript and MOM, and keeps the resulting information safely available locally.**
