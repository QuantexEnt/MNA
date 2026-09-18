# User Flow Specification

## Meeting Notes Assistant — V1

**Document Status:** Approved for V1 development  
**Platform:** Windows  
**Primary meeting platform:** Microsoft Teams

---

# 1. Purpose

This document defines the expected user experience and application behavior for Meeting Notes Assistant.

The objective is to make the application feel like a small desktop utility that quietly waits for a Teams meeting and offers the user a simple way to capture the meeting transcript and personal notes.

The application should not interrupt the user unnecessarily.

The complete experience should remain focused on:

> **Detect → Ask → Capture → Note → Generate MOM → Review → Save**

---

# 2. Application States

The application should have clear internal states.

```text
IDLE
  ↓
TEAMS_MEETING_DETECTED
  ↓
AWAITING_USER_DECISION
  ↓
NOTE_TAKING_ACTIVE
  ↓
MEETING_ENDED / USER_STOPPED
  ↓
PROCESSING
  ↓
MOM_READY
  ↓
SAVED
```

There may also be error/recovery states.

---

# 3. State: IDLE

## Description

The application is running but no active note-taking session exists.

The application may be minimized to the system tray/background.

## Expected behavior

- Monitor for an active Teams meeting.
- Do not record audio.
- Do not send data to OpenAI.
- Use minimal system resources.
- Do not display unnecessary notifications.

## User-visible state

System tray icon or equivalent background indicator.

---

# 4. State: TEAMS_MEETING_DETECTED

When the application detects that the user has entered an active Teams meeting, it should determine whether a note-taking session is already active.

If no session is active, continue to:

`AWAITING_USER_DECISION`

If a session is already active, do not create another session.

---

# 5. State: AWAITING_USER_DECISION

Display a small Windows notification or popup.

Recommended content:

> **Teams meeting detected**  
> Would you like to take notes?

Buttons:

**Start Taking Notes**

**Not Now**

The notification should be visually clear but unobtrusive.

---

# 6. User Selects "Not Now"

If the user selects:

**Not Now**

the application should:

1. Close the notification.
2. Do not capture audio.
3. Do not create a note-taking session.
4. Continue running in the background.
5. Avoid repeatedly asking about the same meeting.

The application should remember that the user dismissed the prompt for the current meeting/session.

When the Teams meeting ends, the dismissed state can be cleared.

---

# 7. User Selects "Start Taking Notes"

When the user selects:

**Start Taking Notes**

the application should:

1. Create a new meeting session.
2. Record meeting metadata available to the application.
3. Start Teams meeting audio capture.
4. Open/show the meeting workspace.
5. Begin transcription processing.
6. Enable the personal notes editor.
7. Clearly indicate that capture is active.

The application should not join or interact with the Teams meeting as a participant.

---

# 8. Meeting Workspace

The meeting workspace is the main screen while note taking is active.

Recommended layout:

```text
+-------------------------------------------------------+
| Meeting Notes Assistant                               |
| Project Review                         ● Recording    |
+-----------------------------+-------------------------+
|                             |                         |
| TRANSCRIPT                  | MY NOTES                |
|                             |                         |
| 10:01 John: ...             | • Discuss pricing      |
| 10:02 Sarah: ...            | • Follow up with...   |
| 10:03 You: ...              |                         |
|                             |                         |
|                             |                         |
+-----------------------------+-------------------------+
| [Pause]                 [End Meeting]                 |
+-------------------------------------------------------+
```

The exact visual design is defined in `UI_UX_SPECIFICATION.md`.

---

# 9. Transcript Behavior

While the session is active:

- Transcript content should appear as it becomes available if live transcription is supported.
- Otherwise, the application may process audio after capture.
- Transcript segments should preferably contain timestamps.
- The transcript should remain readable while new content is added.
- The user should not need to manually refresh the transcript.

If transcription is temporarily unavailable:

- Continue preserving captured audio locally.
- Continue allowing manual notes.
- Show a non-blocking status message.
- Attempt processing again when possible.

---

# 10. Personal Notes Behavior

The user can type notes at any time during the meeting.

Examples:

```text
Need to confirm pricing with Finance.

John will share the updated proposal.

Important: Germany rollout moved to November.
```

Notes should be saved continuously or at frequent intervals.

The application should not require the user to click "Save" after every note.

---

# 11. Relationship Between Transcript and Notes

Transcript and personal notes are separate data sources.

```text
                  MEETING
                     │
             ┌───────┴───────┐
             │               │
             ▼               ▼
        Transcript       My Notes
             │               │
             └───────┬───────┘
                     ▼
                 MOM Input
```

The user's notes should not be mixed into the transcript.

They should remain separately identifiable when passed to the MOM generator.

---

# 12. Meeting Continues

While the Teams meeting is active:

- Continue capturing meeting audio.
- Continue processing transcription.
- Continue saving notes.
- Keep the application responsive.
- Allow the user to stop the session manually.

The application should not require the user to interact with it continuously.

---

# 13. User Manually Ends Note Taking

The user may select:

**End Meeting**

or an equivalent action.

Before stopping, the application may display a confirmation:

> **End note-taking session?**

Options:

- **End & Generate MOM**
- **Continue Meeting**

The exact confirmation behavior can be simplified if usability testing shows it is unnecessary.

---

# 14. Teams Meeting Ends

The application should attempt to detect when the Teams meeting has ended.

If a note-taking session is active, the application should not silently discard it.

Recommended behavior:

> **Your Teams meeting appears to have ended.**  
> Would you like to finish your notes and generate the MOM?

Options:

- **Finish & Generate MOM**
- **Continue Session**

The application may automatically stop audio capture when it can reliably determine that the Teams meeting has ended.

If reliable meeting-end detection is not possible, the user must always have a manual **End Meeting** control.

---

# 15. Capture Stop

When capture ends:

1. Stop audio capture.
2. Finalize the current audio file/chunks.
3. Complete pending transcription where possible.
4. Save the transcript locally.
5. Save personal notes locally.
6. Transition to the processing/review stage.

The application should not delete the recorded audio before the transcript has been successfully processed unless the user has explicitly configured a no-recording-retention option.

---

# 16. Processing State

The application should display a clear progress state.

Example:

```text
Finishing meeting...

✓ Audio captured
✓ Transcript completed
● Preparing MOM...
```

The UI should remain usable.

The user should not mistake "processing" for application failure.

---

# 17. Generate MOM

After transcript processing is complete, the application should present:

**Generate MOM**

The user should be able to initiate MOM generation explicitly.

The input to the AI should include:

- Meeting metadata.
- Transcript.
- Personal notes.

The application should not include unrelated local files or unrelated meetings.

---

# 18. MOM Generation State

Example:

```text
Generating Minutes of Meeting...

Analyzing transcript...
Combining your notes...
Identifying decisions...
Identifying action items...
Preparing MOM...
```

The application should not expose technical API details to the normal user.

If the API fails, show a clear user-facing error and provide:

**Retry**

The transcript and notes must remain available.

---

# 19. MOM Ready

When generation completes, show the MOM in an editable view.

Example:

```text
-------------------------------------------------------
MINUTES OF MEETING

Meeting: Project Review
Date: 11 September 2026

1. Discussion Summary
...

2. Key Decisions
...

3. Action Items
...

4. Open Points
...

5. Next Steps
...
-------------------------------------------------------

[Save] [Copy] [Regenerate]
```

---

# 20. User Edits MOM

The MOM should be editable.

Changes made by the user should not automatically be overwritten.

If the user chooses to regenerate the MOM after editing, display a warning such as:

> Regenerating may replace your current edits.

Require confirmation before replacing the current MOM.

---

# 21. Save Meeting

When the user saves/completes the meeting, the application should preserve:

- Meeting metadata.
- Recording, if retained.
- Transcript.
- Personal notes.
- Final MOM.

The data should be stored locally.

---

# 22. Copy MOM

The user should have a simple:

**Copy MOM**

action.

This should place the complete current MOM into the Windows clipboard.

The user can then paste it into:

- Email.
- Teams chat.
- Word.
- Outlook.
- Any other application.

No integration is required.

---

# 23. Start New Meeting

After a meeting session is completed:

1. Close the completed meeting workspace.
2. Return the application to the background/idle state.
3. Wait for the next Teams meeting.
4. Treat the next meeting as a new session.

A new meeting must not accidentally append to an earlier meeting.

---

# 24. Meeting History

The application may provide a simple history screen.

Example:

```text
MEETING HISTORY

Today
──────────────────────────────
Project Review       10:00 AM
Team Discussion      2:00 PM

Yesterday
──────────────────────────────
Customer Meeting     4:00 PM
```

Selecting a meeting opens its saved information.

The history should not become a complex analytics dashboard.

---

# 25. Error Flow — Teams Detection

If Teams detection fails:

- Continue running.
- Do not crash.
- Do not record.
- Provide a useful diagnostic/status message where appropriate.

The user should still be able to manually start a meeting session if manual start is provided.

---

# 26. Error Flow — Audio Capture

If audio capture cannot start:

Display:

> **Unable to start meeting audio capture.**

Provide:

- Retry.
- Cancel.

Do not pretend that recording is active.

The application should preserve the user's notes if they have already started entering them.

---

# 27. Error Flow — Transcription

If transcription fails:

- Preserve the audio locally.
- Preserve personal notes.
- Inform the user.
- Provide a retry option.
- Do not delete the source audio.

If the API is unavailable, the user should still be able to access the recorded meeting session.

---

# 28. Error Flow — MOM Generation

If MOM generation fails:

- Preserve transcript.
- Preserve notes.
- Preserve audio.
- Show the error.
- Provide Retry.

The user should not have to record the meeting again.

---

# 29. Application Close During Active Meeting

If the user attempts to close the application during an active session:

Display:

> **A meeting note-taking session is active.**  
> Closing the application may stop audio capture.

Options:

- **Keep Running**
- **Stop & Save**
- **Cancel**

The application should make it difficult to accidentally lose the session.

---

# 30. System Shutdown / Sleep

The application should attempt to handle Windows shutdown, restart, and sleep events gracefully.

At minimum:

- Save current notes.
- Finalize accessible data.
- Avoid corrupting local files.
- Indicate if a session was interrupted.

Full automatic recovery is desirable but should not delay the basic V1.

---

# 31. Duplicate Meeting Detection

The application should prevent multiple active note-taking sessions for the same Teams meeting.

Example:

```text
Teams Meeting A
      ↓
Session A active
      ↓
Additional detection event
      ↓
Do NOT create Session B
```

---

# 32. Notification Rules

The application should follow these rules:

### Rule 1

Do not notify when no Teams meeting is active.

### Rule 2

Do not start recording without explicit user action.

### Rule 3

Do not repeatedly prompt for the same active meeting after the user selects **Not Now**.

### Rule 4

Do not create duplicate sessions.

### Rule 5

Clearly indicate active capture.

---

# 33. Recommended End-to-End Flow

The complete happy path is:

```text
                    APP START
                       │
                       ▼
                     IDLE
                       │
                       ▼
              Teams meeting detected
                       │
                       ▼
                Show notification
                       │
             ┌─────────┴─────────┐
             │                   │
             ▼                   ▼
           Not Now             Start
             │                   │
             ▼                   ▼
            IDLE          Create session
                                 │
                                 ▼
                         Capture Teams audio
                                 │
                    ┌────────────┴────────────┐
                    │                         │
                    ▼                         ▼
               Transcript                 My Notes
                    │                         │
                    └────────────┬────────────┘
                                 │
                                 ▼
                         End note-taking
                                 │
                                 ▼
                       Finalize transcript
                                 │
                                 ▼
                         Generate MOM
                                 │
                                 ▼
                         Review / Edit
                                 │
                                 ▼
                          Save / Copy
                                 │
                                 ▼
                                IDLE
```

---

# 34. Core UX Principle

At every point, the application should answer one question for the user:

> **"What do I need to do next?"**

The number of required user actions should be minimized.

The ideal interaction is:

**One click to start.**

**One click to end.**

**One click to generate MOM.**

Everything else should happen automatically where reliable.

---

# 35. V1 UX Boundary

Do not add user flows for:

- Team collaboration.
- Project management.
- Calendar management.
- Task synchronization.
- Cloud synchronization.
- Multiple meeting providers.
- Complex user accounts.
- Advanced AI workflows.

The V1 user flow ends with:

> **A useful, editable MOM saved locally on the user's computer.**
