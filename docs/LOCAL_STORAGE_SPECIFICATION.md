# Local Storage Specification

## Meeting Notes Assistant — V1

**Document Status:** Implementation specification  
**Platform:** Windows desktop  
**Storage principle:** Local-first, simple, human-readable

---

# 1. Purpose

Meeting Notes Assistant does not require a cloud database or backend for V1.

Meeting information should be stored locally on the user's Windows computer.

The application should preserve the complete meeting working set:

```text
Meeting
├── Audio
├── Transcript
├── Manual Notes
└── MOM
```

The primary objective is reliability and simplicity.

---

# 2. Storage Principle

V1 should use:

- Local Windows folders.
- JSON for structured application data.
- Markdown for human-readable documents.
- Audio files for original recordings.

Do not introduce:

- MySQL.
- PostgreSQL.
- Firebase.
- Supabase.
- Cloud databases.
- A hosted backend.

A database such as SQLite may be considered later if the application's search/history requirements justify it.

---

# 3. Recommended Root Folder

The default application data location should be user-specific.

Recommended concept:

```text
%LOCALAPPDATA%\MeetingNotesAssistant\
```

or a user-configurable equivalent.

The application should not require administrator privileges to write meeting data.

---

# 4. Meeting Storage Location

Users should be able to configure the meeting storage location.

Example:

```text
C:\Users\<User>\Documents\MeetingNotesAssistant\
```

The default should be a sensible user-accessible Windows folder.

The exact default can be finalized during implementation.

---

# 5. Meeting Folder Structure

Recommended structure:

```text
MeetingNotesAssistant/
│
├── meetings/
│   │
│   ├── 2026-09-11_1000_Service_Review/
│   │   ├── meeting.json
│   │   │
│   │   ├── audio/
│   │   │   └── meeting.wav
│   │   │
│   │   ├── transcript/
│   │   │   ├── transcript.json
│   │   │   └── transcript.md
│   │   │
│   │   ├── notes/
│   │   │   └── notes.md
│   │   │
│   │   └── mom/
│   │       ├── mom.json
│   │       ├── mom_generated.md
│   │       └── mom_final.md
│   │
│   └── ...
│
└── app/
    └── settings.json
```

The exact directory layout may change slightly during implementation, but the separation of artifacts should remain.

---

# 6. Meeting Identifier

Every meeting session should have a unique internal ID.

Example:

```text
meeting_20260911_100000_a8f31
```

Do not use the meeting title alone as the identifier.

Two meetings may have identical titles.

---

# 7. Folder Naming

A human-readable folder name is preferred.

Example:

```text
2026-09-11_1000_Service_Review
```

If the title contains invalid Windows filename characters, sanitize them.

For example:

```text
Customer: Q3 / Planning
```

may become:

```text
Customer_Q3_Planning
```

Do not allow user-controlled titles to create invalid or unsafe paths.

---

# 8. Meeting Metadata

Each meeting should have a `meeting.json`.

Suggested structure:

```json
{
  "id": "meeting_20260911_100000_a8f31",
  "title": "Service Review",
  "started_at": "2026-09-11T10:00:00",
  "ended_at": "2026-09-11T11:00:00",
  "status": "completed",
  "audio": {
    "path": "audio/meeting.wav",
    "duration_seconds": 3600
  },
  "transcript": {
    "status": "completed",
    "path": "transcript/transcript.json"
  },
  "notes": {
    "path": "notes/notes.md"
  },
  "mom": {
    "status": "edited",
    "path": "mom/mom_final.md"
  }
}
```

This is an example schema, not a rigid final contract.

---

# 9. Meeting Lifecycle Status

Suggested meeting states:

```text
DETECTED
ACTIVE
STOPPING
PROCESSING
COMPLETED
PARTIAL
FAILED
```

Examples:

```text
DETECTED
  ↓
ACTIVE
  ↓
PROCESSING
  ↓
COMPLETED
```

If audio capture fails but notes survive:

```text
ACTIVE
  ↓
PARTIAL
```

---

# 10. Audio Storage

The original recording should be stored locally.

Example:

```text
audio/meeting.wav
```

The audio file should not be embedded inside `meeting.json`.

Keep large binary files separate from metadata.

---

# 11. Audio Temporary Files

If capture requires temporary chunks:

```text
audio/tmp/
```

may be used.

Example:

```text
audio/
├── tmp/
│   ├── chunk_0001.wav
│   └── chunk_0002.wav
└── meeting.wav
```

Temporary files should be removed after successful finalization.

If finalization fails, preserve the chunks when possible so recovery remains possible.

---

# 12. Transcript Storage

Store both structured and human-readable transcript forms where practical.

```text
transcript/
├── transcript.json
└── transcript.md
```

`transcript.json` supports application processing.

`transcript.md` allows the user to open the transcript independently.

---

# 13. Notes Storage

Manual notes should be stored as:

```text
notes/notes.md
```

The notes file should be continuously updated during the meeting.

The application should not wait until the meeting ends to save notes.

---

# 14. MOM Storage

The MOM should have a clear distinction between generated and final versions.

Recommended:

```text
mom/
├── mom.json
├── mom_generated.md
└── mom_final.md
```

Meaning:

```text
mom_generated.md
    = original AI draft

mom_final.md
    = user's reviewed/edited version
```

---

# 15. MOM Editing

When the user edits the MOM:

```text
AI-generated draft
        ↓
User edits
        ↓
mom_final.md
```

Do not overwrite the generated draft unless explicitly intended.

This protects the user from losing the original AI output.

---

# 16. Auto-Save

Auto-save should be used for:

- Manual notes.
- MOM editing.

Recommended behavior:

```text
User changes content
       ↓
Short debounce
       ↓
Save locally
```

The application should provide a small status such as:

```text
Saved locally
```

---

# 17. Crash Recovery

If the application crashes during a meeting, the next launch should attempt to identify unfinished sessions.

Example:

```text
Meeting detected as incomplete.

An unfinished meeting session was found.

[ Recover ] [ Discard ]
```

The default should favor recovery.

Do not automatically delete incomplete meeting data.

---

# 18. Recovery Data

A recoverable session may contain:

```text
meeting.json
audio/
notes/
```

The application should determine whether transcription can continue after recovery.

For example:

```text
Audio finalized
    ↓
Transcription can resume
```

or:

```text
Audio incomplete
    ↓
Preserve for manual recovery
```

---

# 19. Application Settings

Application-level settings should be stored separately from meeting data.

Example:

```text
app/settings.json
```

Possible settings:

```json
{
  "storage_location": "C:\\Users\\User\\Documents\\MeetingNotesAssistant",
  "start_with_windows": true,
  "show_meeting_notification": true,
  "openai": {
    "transcription_model": "configured-model",
    "mom_model": "configured-model"
  }
}
```

The API key should preferably not be stored as plain text in this JSON file.

Use an appropriate Windows secure credential mechanism once the prototype is working.

---

# 20. API Key Storage

Do not store the OpenAI API key in:

```text
meeting.json
settings.json
logs
source code
Git repository
```

For V1, use a local secure storage mechanism where practical.

During early development, an environment variable can be used.

Example:

```text
OPENAI_API_KEY
```

---

# 21. No Secrets in Meeting Files

Meeting files should never contain:

- OpenAI API keys.
- Authentication tokens.
- Windows credentials.
- Application secrets.

Meeting files may contain sensitive meeting content, so they should be treated as private user data.

---

# 22. File Encoding

Text files should use:

**UTF-8**

This ensures compatibility with:

- English.
- Indian languages.
- Technical terminology.
- Special characters.
- Names.

Avoid proprietary encodings.

---

# 23. File Naming Rules

Use predictable lowercase or simple names.

Recommended:

```text
meeting.json
notes.md
transcript.json
transcript.md
mom.json
mom_generated.md
mom_final.md
```

Avoid filenames based entirely on AI-generated text.

---

# 24. Path Safety

All generated filenames and paths must be sanitized.

Handle:

- Invalid Windows characters.
- Very long titles.
- Reserved Windows filenames.
- Trailing spaces.
- Trailing periods.
- Duplicate folder names.

The application must never allow a meeting title to escape the configured meeting storage directory.

---

# 25. Duplicate Titles

Two meetings may have the same title.

Example:

```text
Service Review
Service Review
```

Use timestamp and unique ID to distinguish them.

Example:

```text
2026-09-11_1000_Service_Review
2026-09-12_1000_Service_Review
```

---

# 26. Duplicate Detection

Duplicate detection should be based on meeting/session identity and timing where reliable.

Do not treat identical titles as duplicate meetings.

---

# 27. Meeting History

The history screen should read local meeting metadata.

V1 can discover meetings by scanning:

```text
meetings/
```

and reading:

```text
meeting.json
```

This avoids requiring a database.

---

# 28. History Performance

For a small number of meetings, scanning folders is sufficient.

If the user eventually has thousands of meetings and history becomes slow, consider:

- Indexed metadata.
- SQLite.
- Search indexing.

Do not introduce these systems prematurely.

---

# 29. Search

If search is implemented in V1, search can initially use:

- Meeting titles.
- Dates.
- Stored metadata.

Full-text search across transcripts and MOMs can be added later.

Do not build a dedicated search database unless needed.

---

# 30. Storage Location Changes

If the user changes the storage location:

```text
Current:
C:\...\MeetingNotesAssistant

New:
D:\Meetings
```

The application should not silently move existing meetings.

A simple V1 behavior is:

1. Ask whether existing meetings should be moved.
2. If not, use the new location only for future meetings.
3. Preserve existing data.

---

# 31. Storage Validation

When the user selects a storage location, verify:

- Directory exists or can be created.
- Application can write to it.
- Application can read from it.
- Sufficient path permissions exist.

Show a clear error if validation fails.

---

# 32. Disk Space

Audio recordings can consume significant storage.

The application should check for obvious disk-space problems before recording where practical.

If storage becomes critically low:

```text
Not enough disk space to safely continue recording.
```

The application should prioritize preserving existing data.

---

# 33. Storage Failure

If writing audio fails:

```text
Unable to save meeting audio.

Your manual notes are still being saved if possible.
```

If writing notes fails:

```text
Unable to save notes.

Please check the meeting storage location.
```

Do not pretend the artifact was saved.

---

# 34. Atomic Writes

For important structured files such as:

```text
meeting.json
mom.json
```

prefer safe/atomic write behavior.

Conceptually:

```text
Write temporary file
      ↓
Flush
      ↓
Replace original
```

This reduces corruption risk if the application crashes during a write.

---

# 35. File Locking

The application should avoid unnecessary file locks.

Users should ideally be able to open Markdown files while the application is running.

If a file is actively being written, use appropriate safe-write behavior.

---

# 36. Local-Only Principle

Meeting data should remain local unless it is intentionally sent to an external AI service for processing.

The application should not silently synchronize meeting files to:

- OneDrive.
- Google Drive.
- Dropbox.
- Other cloud storage.

Cloud integrations are outside V1.

---

# 37. Backup

V1 does not need an internal backup system.

Users may back up the configured meeting folder using normal Windows backup tools.

The folder structure should therefore remain portable.

---

# 38. Delete Meeting

A simple delete option may be provided in history.

If implemented:

```text
Delete meeting?

This will delete:
- Audio
- Transcript
- Notes
- MOM

[ Delete ] [ Cancel ]
```

Because audio and notes may be valuable, deletion should require explicit confirmation.

---

# 39. Permanent Deletion

When the user confirms deletion, remove the complete meeting folder.

Do not leave orphaned audio chunks or temporary files.

If Windows prevents deletion of a file:

- Report the failure.
- Do not claim the meeting was fully deleted.

---

# 40. Export

V1 does not require a separate export subsystem.

Because the artifacts are stored as Markdown and JSON, users can already access the content locally.

Future versions may provide:

- DOCX.
- PDF.
- TXT.

These are outside the core V1 storage requirement.

---

# 41. Data Integrity

A completed meeting should normally contain:

```text
meeting.json
audio/meeting.wav
notes/notes.md
transcript/transcript.json
transcript/transcript.md
mom/mom.json
mom/mom_generated.md
mom/mom_final.md
```

Not every artifact is mandatory if the corresponding processing step was not completed.

For example:

```text
Transcription failed
```

may result in:

```text
meeting.json
audio/meeting.wav
notes/notes.md
```

with transcript status marked as failed/pending.

---

# 42. Partial Meetings

A meeting should not be considered lost because one processing step failed.

Example:

```text
Audio:      AVAILABLE
Notes:      AVAILABLE
Transcript: FAILED
MOM:        NOT GENERATED
```

This is a valid recoverable state.

---

# 43. Data Relationships

The relationship should be:

```text
Meeting ID
   │
   ├── Audio
   ├── Transcript
   ├── Notes
   └── MOM
```

Every artifact should be traceable to the same meeting ID.

---

# 44. Local Data Privacy

Because meeting audio and transcripts can contain confidential information, the application should:

- Store data in user-controlled locations.
- Avoid unnecessary copies.
- Avoid logging content.
- Avoid cloud synchronization.
- Clearly explain what is sent to OpenAI.
- Keep local artifacts independently accessible.

---

# 45. Logging

Logs should be separate from meeting content.

Recommended:

```text
%LOCALAPPDATA%\MeetingNotesAssistant\logs\
```

Logs may contain:

- Application startup.
- Meeting detection.
- Capture start/stop.
- Processing status.
- API request status.
- Error categories.

Logs should not contain:

- Raw audio.
- Full transcripts.
- Full MOMs.
- API keys.
- Sensitive meeting content.

---

# 46. Temporary Storage

Temporary processing files should be placed in a dedicated temporary directory.

Example:

```text
%LOCALAPPDATA%\MeetingNotesAssistant\tmp\
```

Temporary data should be cleaned up after successful processing.

If cleanup fails, the application should retry later rather than deleting data aggressively.

---

# 47. Storage Migration

V1 does not require complex schema migration.

If `meeting.json` changes in future versions, include a version field.

Example:

```json
{
  "schema_version": 1
}
```

This allows future versions to recognize older meeting data.

---

# 48. Data Versioning

Suggested:

```text
meeting.json
{
  "schema_version": 1,
  ...
}
```

Future versions can support:

```text
schema_version: 2
```

Do not change file structures without considering existing meetings.

---

# 49. Application Uninstall

Uninstalling the application should not automatically delete user meeting data without explicit confirmation.

The installer/uninstaller should clearly distinguish:

```text
Application files
```

from:

```text
User meeting data
```

Meeting data should be preserved by default.

---

# 50. Portable Human-Readable Data

A major benefit of the local-first design is that users can open their content without the application.

For example:

```text
transcript.md
notes.md
mom_final.md
```

should be readable in any normal text/Markdown editor.

This prevents unnecessary lock-in.

---

# 51. Acceptance Criteria

The local storage implementation is complete for V1 when:

1. Meetings can be stored entirely on the Windows machine.
2. No cloud database is required.
3. Every meeting has a unique ID.
4. Audio is stored separately from metadata.
5. Notes are saved continuously.
6. Transcript is saved locally.
7. MOM draft and final version can be distinguished.
8. Application settings are stored separately.
9. API secrets are not stored in meeting files.
10. Files use UTF-8.
11. Windows-invalid filenames are sanitized.
12. Duplicate meeting titles do not overwrite each other.
13. Interrupted meetings can be recovered where possible.
14. Processing failures do not delete source artifacts.
15. Users can access the Markdown files directly.
16. Storage location can be configured.
17. Storage errors are reported honestly.
18. Important metadata writes are protected against partial corruption.

---

# 52. Recommended Implementation Priority

## P0

- Meeting folder creation.
- `meeting.json`.
- Audio storage.
- Notes storage.
- Transcript storage.
- MOM storage.
- Settings storage.
- Safe writes.
- Recovery state.

## P1

- Storage location setting.
- Meeting history scanning.
- Delete meeting.
- Temporary-file cleanup.
- Disk-space checks.

## P2

- Full-text search.
- Database/indexing.
- Export.
- Backup management.
- Storage migration tools.

---

# 53. Definition of Done

A complete meeting should be recoverable from one local folder.

Example:

```text
2026-09-11_1000_Service_Review/
│
├── meeting.json
├── audio/
│   └── meeting.wav
├── transcript/
│   ├── transcript.json
│   └── transcript.md
├── notes/
│   └── notes.md
└── mom/
    ├── mom.json
    ├── mom_generated.md
    └── mom_final.md
```

The user owns these files.

The application is simply the tool that creates and manages them.

---

# 54. Final Principle

For V1:

> **Local files first. Database later, only if needed.**

The storage design should remain simple enough that a developer can understand the entire data model by looking at one meeting folder.
