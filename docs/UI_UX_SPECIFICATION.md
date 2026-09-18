# UI/UX Specification

## Meeting Notes Assistant — V1

**Document Status:** Implementation specification  
**Platform:** Windows desktop  
**UI technology:** React + TypeScript inside Tauri  
**Design principle:** Simple, quiet, focused

---

# 1. Purpose

The UI should make one workflow extremely easy:

```text
Detect meeting
      ↓
Ask user
      ↓
Take notes
      ↓
Capture transcript
      ↓
Generate MOM
      ↓
Review / Edit
      ↓
Save / Copy
```

The application is **not** intended to become a full meeting-management platform.

---

# 2. UX Philosophy

The product should feel:

- Simple.
- Professional.
- Quiet.
- Fast.
- Unobtrusive.
- Easy to understand.
- Useful without training.

Avoid:

- Complex dashboards.
- Excessive settings.
- Large navigation systems.
- Unnecessary animations.
- Too many buttons.
- Feature-heavy meeting management.

The user should be able to understand the application within seconds.

---

# 3. Primary User Experience

The application normally stays in the background.

The user should not need to keep the main window open.

Typical experience:

```text
Windows desktop
     ↓
Teams meeting starts
     ↓
Small notification appears
     ↓
"Teams meeting detected.
Would you like to take notes?"
     ↓
[Start Taking Notes] [Not Now]
```

---

# 4. Application States

The UI should represent these high-level states:

```text
IDLE
MEETING_DETECTED
AWAITING_USER
RECORDING
PROCESSING
REVIEW
HISTORY
SETTINGS
ERROR
```

The actual implementation may use more granular internal states.

---

# 5. System Tray

The application should run from the Windows system tray when not actively being used.

Tray menu:

```text
Meeting Notes Assistant

● Running

Open
Settings
About
Exit
```

During an active session:

```text
Meeting Notes Assistant

● Recording
Open Meeting
Stop Taking Notes
Settings
Exit
```

The application should not repeatedly interrupt the user.

---

# 6. Main Window

The main window should be clean and compact.

Suggested layout:

```text
┌─────────────────────────────────────────────┐
│ Meeting Notes Assistant                 — □ X│
├─────────────────────────────────────────────┤
│                                             │
│  Current Meeting                            │
│                                             │
│  No active meeting                          │
│                                             │
│  Waiting for a Microsoft Teams meeting...   │
│                                             │
└─────────────────────────────────────────────┘
```

The main screen should not contain unnecessary statistics or charts.

---

# 7. Meeting Detection Popup

When a Teams meeting is detected, show a lightweight notification/dialog.

Suggested content:

```text
Teams meeting detected

A Microsoft Teams meeting appears to be active.

Would you like to take notes?

[ Start Taking Notes ]     [ Not Now ]
```

Optional small status:

```text
Teams meeting detected
```

Do not show technical information such as process IDs.

---

# 8. Start Taking Notes

When the user selects:

**Start Taking Notes**

the application should:

1. Start the meeting session.
2. Start audio capture.
3. Create the local meeting folder.
4. Open the meeting workspace.
5. Show recording/capture status.

The transition should be immediate.

---

# 9. Not Now

When the user selects:

**Not Now**

the application should:

- Close the notification.
- Not capture audio.
- Continue monitoring Teams.
- Avoid repeatedly prompting for the same meeting.

The application may remember that the user dismissed the prompt for the current meeting.

---

# 10. Active Meeting Workspace

This is the primary working screen.

Recommended layout:

```text
┌────────────────────────────────────────────────────┐
│ Service Review Meeting                 ● Recording │
├───────────────────────────┬────────────────────────┤
│                           │                        │
│ TRANSCRIPT                │ MY NOTES               │
│                           │                        │
│ 10:02                     │ - Review ownership    │
│ Let's start...            │ - Finance follow-up   │
│                           │                        │
│ 10:05                     │                        │
│ Current portfolio...      │                        │
│                           │                        │
├───────────────────────────┴────────────────────────┤
│                         [ End Meeting ]             │
└────────────────────────────────────────────────────┘
```

---

# 11. Active Meeting Header

The header should show:

- Meeting title.
- Capture status.
- Optional elapsed time.

Example:

```text
Service Review Meeting

● Recording     42:17
```

Do not show excessive technical status.

---

# 12. Recording Indicator

The application must clearly communicate that meeting audio is being captured.

Example:

```text
● Recording
```

or:

```text
● Capturing meeting audio
```

The indicator should only appear when capture is actually active.

---

# 13. Transcript Panel

The transcript panel should display the meeting transcript.

For post-meeting transcription, the panel may initially show:

```text
Transcript will appear after the meeting.
```

During future near-live transcription:

```text
Transcript
────────────────────────

10:02
Let's begin the service review.

10:04
We have three major gaps...
```

The UI should not pretend the transcript is live when V1 is using post-meeting transcription.

---

# 14. Manual Notes Panel

The notes panel should be a simple text editor.

Placeholder:

```text
Type your notes here...
```

The user should be able to type freely.

Do not force note categories.

---

# 15. Notes Auto-Save Indicator

A small status can indicate:

```text
Saved
```

or:

```text
Saving...
```

or:

```text
Saved locally
```

This should be subtle.

---

# 16. End Meeting

Primary action:

**End Meeting**

When selected, show a confirmation only if necessary.

Suggested:

```text
End note-taking session?

Your meeting audio and notes will be saved.

[ End Meeting ] [ Cancel ]
```

Avoid confirmation dialogs if accidental activation is unlikely, but do not make it easy to accidentally lose a session.

---

# 17. Meeting Processing Screen

After ending the meeting:

```text
Meeting complete

Saving meeting...
✓ Audio saved
✓ Notes saved

Transcribing meeting...
```

Then:

```text
✓ Transcript ready

Generating MOM...
```

Finally:

```text
✓ MOM ready

[ Review MOM ]
```

---

# 18. Processing States

The user should see simple sequential status.

Example:

```text
Meeting complete

✓ Audio saved
✓ Notes saved
● Transcribing...
○ Generate MOM
```

Then:

```text
✓ Audio saved
✓ Notes saved
✓ Transcript ready
● Generating MOM
```

Then:

```text
✓ Audio saved
✓ Notes saved
✓ Transcript ready
✓ MOM ready
```

---

# 19. Review Screen

The review screen should allow the user to inspect:

- Transcript.
- Manual notes.
- MOM.

Suggested tabs:

```text
[ MOM ] [ Transcript ] [ My Notes ]
```

Default tab:

**MOM**

because that is the primary output.

---

# 20. MOM Editor

The MOM should be editable directly.

Example:

```text
┌─────────────────────────────────────────────┐
│ Minutes of Meeting                          │
│                                             │
│ Date: 11 September 2026                     │
│                                             │
│ Purpose                                     │
│ Review service portfolio...                 │
│                                             │
│ Key Discussion Points                       │
│ • Current portfolio structure               │
│ • Ownership gaps                            │
│                                             │
│ Decisions                                   │
│ • Establish cross-functional review         │
│                                             │
│ Action Items                                │
│ • Confirm ownership — John — Friday         │
│                                             │
└─────────────────────────────────────────────┘
```

---

# 21. MOM Actions

The review screen should provide:

```text
[ Save ]
[ Copy ]
[ Regenerate ]
```

Optional:

```text
[ Export ]
```

Export is not required for the first implementation.

---

# 22. Regenerate Warning

If the user has edited the MOM and selects Regenerate:

```text
Your MOM has been edited.

Regenerating may replace your changes.

[ Regenerate ] [ Cancel ]
```

Never silently destroy user edits.

---

# 23. Copy Behavior

The **Copy** action should copy the final visible MOM.

It should be suitable for pasting into:

- Email.
- Teams.
- Word.
- OneNote.
- Documents.

No external integration is required.

---

# 24. Save Behavior

Save should be automatic where practical, with a visible explicit Save action as reassurance.

Example:

```text
Saved locally
```

The application should save the final MOM in the meeting's local folder.

---

# 25. Meeting History

V1 should include a simple meeting history.

Example:

```text
Meeting History

Today
────────────────────────
Service Review
10:00 AM

Architecture Discussion
8:30 AM

Yesterday
────────────────────────
Customer Planning
4:00 PM
```

Each item should allow:

```text
Open
```

No advanced filtering is required.

---

# 26. History Search

Search is optional for V1.

If implemented, simple text search is sufficient:

```text
Search meetings...
```

Search should operate against local meeting metadata and/or stored text.

Do not introduce a cloud search service.

---

# 27. Empty History

If there are no previous meetings:

```text
No meetings yet.

When you take notes during a Teams meeting,
your meetings will appear here.
```

---

# 28. Settings

Settings should remain minimal.

Suggested sections:

## OpenAI

```text
API Key
[ *************** ]

Transcription Model
[ configured model ]

MOM Model
[ configured model ]

[ Test Connection ]
```

## General

```text
Start with Windows
[ ✓ ]

Show meeting detection notification
[ ✓ ]
```

## Storage

```text
Meeting storage location
[ C:\Users\...\MeetingNotes ]
```

Do not add settings for features that do not exist.

---

# 29. API Key UI

The API key should be entered into a password-style field.

Example:

```text
OpenAI API Key

[ ••••••••••••••••••• ]

[ Test Connection ]
```

Never display the complete key after it has been saved.

---

# 30. Settings Validation

If the key is invalid:

```text
Connection failed.

Please check your OpenAI API key.
```

If valid:

```text
Connection successful.
```

Avoid displaying raw API errors unless the user opens an advanced diagnostic area.

---

# 31. Error UX

Errors should be understandable and actionable.

Bad:

```text
Error 0x80070005
```

Better:

```text
Unable to capture Teams meeting audio.

Your session has not been lost.

[ Retry ] [ Continue with Notes ]
```

---

# 32. Audio Capture Error

During a meeting:

```text
Unable to capture meeting audio.

Your manual notes are still being saved.

[ Retry Capture ] [ Continue with Notes ]
```

If capture cannot be recovered, the user should still be able to preserve manual notes.

---

# 33. Transcription Error

After the meeting:

```text
Transcription could not be completed.

Your meeting audio is saved locally.

[ Retry ] [ Review Notes ]
```

---

# 34. MOM Error

If transcription succeeds but MOM generation fails:

```text
MOM generation failed.

Your transcript and notes are safe.

[ Retry ] [ Review Transcript ]
```

---

# 35. Network Error

If internet connectivity is unavailable:

```text
OpenAI is currently unavailable.

Your meeting has been saved locally.
You can retry transcription later.
```

---

# 36. Application Shutdown

If the user tries to exit while a meeting session is active:

```text
A meeting session is active.

Closing the application may stop audio capture.

[ Keep Running ] [ Stop & Exit ] [ Cancel ]
```

The safest default is to keep the application running in the tray.

---

# 37. Background Behavior

When minimized:

```text
Main window
    ↓
System tray
```

The application continues meeting detection.

During an active recording session, minimizing the application must not stop capture.

---

# 38. Notifications

Notifications should be used sparingly.

Recommended notifications:

### Meeting detected

> Teams meeting detected. Start taking notes?

### Session complete

> Meeting notes are ready for review.

### Processing complete

> Your MOM is ready.

Do not generate repeated notifications for the same event.

---

# 39. Accessibility

Basic accessibility is required.

The UI should support:

- Keyboard navigation.
- Visible focus.
- Readable font sizes.
- Sufficient text contrast.
- Buttons with meaningful labels.
- Clear status messages.

Do not rely solely on color to communicate status.

For example:

```text
● Recording
```

is better than a red dot with no text.

---

# 40. Keyboard Shortcuts

V1 may support a small number of useful shortcuts.

Possible:

```text
Ctrl + S       Save
Ctrl + C       Copy selected text
Esc            Close dialog
```

Do not create a large shortcut system.

---

# 41. Responsive Behavior

The application is a desktop application.

It should work at common Windows desktop resolutions.

The meeting workspace should remain usable when the window is resized.

Recommended minimum:

```text
1000 × 650
```

The application should avoid requiring an extremely large screen.

---

# 42. Window Behavior

Recommended:

- Normal resizable window.
- Minimize to tray.
- Remember last window size.
- Remember last window position where practical.
- Do not force full-screen.
- Do not open automatically in front of other applications except for the meeting-detection prompt.

---

# 43. Visual Style

Recommended style:

- Clean.
- Modern.
- Professional.
- Neutral.
- Minimal.

Use:

- Clear typography.
- Simple spacing.
- Subtle borders.
- Small number of accent elements.
- Consistent button hierarchy.

Avoid:

- Heavy gradients.
- Decorative illustrations.
- Excessive cards.
- Gamification.
- Large dashboard widgets.

---

# 44. Primary Button Hierarchy

The most important action should be visually obvious.

Example:

```text
Primary:
[ Start Taking Notes ]

Secondary:
[ Not Now ]
```

During meeting:

```text
Primary:
[ End Meeting ]

Secondary:
[ Minimize ]
```

After meeting:

```text
Primary:
[ Review MOM ]

Secondary:
[ View Transcript ]
```

---

# 45. UI Component Suggestions

Suggested React components:

```text
AppShell
SystemTrayStatus
MeetingDetectionDialog
MeetingWorkspace
MeetingHeader
TranscriptPanel
NotesPanel
RecordingStatus
ProcessingView
MomReview
MomEditor
MeetingHistory
SettingsView
ErrorMessage
ConfirmDialog
```

The exact component names can change.

---

# 46. Frontend State

The React application should receive application state from the Tauri/Rust layer through a clear interface.

Conceptually:

```text
Rust
  ↓
Tauri Events
  ↓
React State
  ↓
UI
```

Do not duplicate meeting detection or audio-capture state independently in multiple React components.

---

# 47. State Synchronization

The UI should reflect the real native application state.

For example:

```text
Rust says:
capture_status = active

UI:
● Recording
```

If capture stops unexpectedly:

```text
Rust:
capture_status = error

UI:
Unable to capture meeting audio
```

The UI must not maintain a false recording indicator.

---

# 48. Loading States

Every operation that may take more than a moment should have a visible state.

Examples:

```text
Detecting...
Starting...
Saving...
Transcribing...
Generating...
Loading...
```

Avoid frozen-looking screens.

---

# 49. First-Run Experience

The first launch may show a simple setup screen:

```text
Welcome to Meeting Notes Assistant

This app detects Microsoft Teams meetings,
captures meeting audio locally, and creates MOMs.

To use AI transcription and MOM generation,
add your OpenAI API key.

[ Configure OpenAI ]
[ Continue Without AI ]
```

The user should be able to continue without configuring OpenAI immediately if desired.

---

# 50. Permission / Explanation Screen

Before the first recording, explain what is happening.

Example:

```text
Meeting audio capture

When you choose "Start Taking Notes",
the app captures the Teams meeting audio
available on your computer.

It does not independently record your microphone.

[ Continue ]
```

This helps establish user trust.

---

# 51. Recording Consent

The application should require explicit user action to start recording.

The user action:

```text
Start Taking Notes
```

is the trigger.

Do not automatically record simply because a meeting is detected.

---

# 52. Privacy Indicator

During active capture, the UI should clearly show:

```text
● Capturing meeting audio
```

This should remain visible in the meeting workspace.

---

# 53. No Hidden Recording

The application must never:

- Start recording silently.
- Record after the user selects Not Now.
- Continue recording after the session ends.
- Capture microphone audio independently.
- Hide active recording status.

---

# 54. Product Boundary Reflected in UI

The UI should not expose functionality that the product does not support.

Do not show:

- Calendar.
- Teams integration settings.
- Zoom.
- Slack.
- Jira.
- Analytics.
- Sentiment.
- Project dashboards.
- Collaboration features.

Keeping these out of the UI helps prevent scope creep.

---

# 55. Recommended Navigation

Keep navigation extremely small:

```text
Meeting
History
Settings
```

Possible layout:

```text
┌──────────────────────────┐
│ Meeting Notes Assistant  │
├──────────────────────────┤
│ ● Meeting                │
│   History                │
│   Settings               │
├──────────────────────────┤
│                          │
│       Main Content       │
│                          │
└──────────────────────────┘
```

The navigation may be hidden when the application is used primarily through the active meeting workspace.

---

# 56. Main User Journey

The final intended UX is:

```text
Application running quietly
        ↓
Teams meeting detected
        ↓
Small notification
        ↓
User chooses Start Taking Notes
        ↓
Meeting workspace opens
        ↓
Audio capture active
        ↓
User types notes
        ↓
Meeting ends
        ↓
Processing screen
        ↓
Transcript generated
        ↓
MOM generated
        ↓
MOM review screen
        ↓
User edits
        ↓
Save / Copy
```

---

# 57. Acceptance Criteria

The UI/UX implementation is complete for V1 when:

1. The application can run quietly in the Windows background.
2. A Teams meeting detection prompt is clear.
3. Start Taking Notes explicitly begins the session.
4. Not Now does not start recording.
5. The active meeting workspace is simple and usable.
6. Recording status is clearly visible.
7. Manual notes can be entered easily.
8. Notes are visibly saved locally.
9. Processing status is understandable.
10. Transcript can be reviewed.
11. MOM can be edited.
12. MOM can be saved.
13. MOM can be copied.
14. Errors are understandable and actionable.
15. Settings remain minimal.
16. The application can minimize to the system tray.
17. The UI never claims recording/transcription is active when it is not.
18. No out-of-scope functionality appears in the V1 interface.

---

# 58. Definition of Done

A user who has never seen the application before should be able to:

```text
Open application
     ↓
Attend Teams meeting
     ↓
Accept recording prompt
     ↓
Take notes
     ↓
End meeting
     ↓
Wait for processing
     ↓
Review MOM
     ↓
Edit MOM
     ↓
Copy/save MOM
```

without needing documentation or technical knowledge.

---

# 59. Final UX Principle

The application should feel like a **small assistant sitting quietly beside Teams**, not another enterprise application the user has to manage.

The best V1 UI is the one that:

> **gets out of the user's way while making the final MOM dramatically easier to produce.**
