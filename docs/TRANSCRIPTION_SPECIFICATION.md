# Transcription Specification

## Meeting Notes Assistant — V1

**Document Status:** Implementation specification  
**Purpose:** Define how captured Teams meeting audio becomes a reliable local transcript.

---

# 1. Purpose

The transcription component converts the locally captured Teams meeting audio into text that can be:

- Viewed by the user.
- Stored locally.
- Used as input for MOM generation.
- Reviewed when the AI-generated MOM needs correction.

The transcription system must preserve the original source audio and must not silently replace or destroy it.

---

# 2. Core Principle

The workflow is:

```text
Teams Meeting
     ↓
Audio Capture
     ↓
Local Audio File
     ↓
Transcription
     ↓
Local Transcript
     ↓
MOM Generation
```

The transcription component does **not** capture audio itself.

It receives finalized audio from the audio-capture subsystem.

---

# 3. V1 Transcription Mode

The default V1 implementation should use:

**Post-meeting transcription.**

Workflow:

```text
Meeting active
     ↓
Audio continuously recorded
     ↓
Meeting ends
     ↓
Audio finalized
     ↓
Transcription starts
     ↓
Transcript generated
```

This is preferred over near-live transcription for the first working version because reliability is more important than live display.

---

# 4. Near-Live Transcription

Near-live transcription is optional.

It may be implemented later if the underlying API and application architecture support it reliably.

Possible future flow:

```text
Audio chunk
    ↓
Transcription API
    ↓
Transcript segment
    ↓
UI
```

Potential problems include:

- Duplicate text.
- Missing words at chunk boundaries.
- Delayed responses.
- Out-of-order responses.
- Failed chunks.
- Network interruptions.
- Increased API usage.
- Increased implementation complexity.

Do not make near-live transcription a blocker for V1.

---

# 5. Input Requirements

The transcription service receives:

- A finalized audio file.
- The selected transcription model.
- Optional language information if known.

Example:

```text
Audio:
meeting_2026-09-11_10-00.wav

Model:
configured transcription model
```

The service should validate the file before making the API request.

---

# 6. Audio Validation

Before transcription, verify:

1. File exists.
2. File is readable.
3. File size is greater than zero.
4. File is finalized.
5. File format is supported.
6. Audio duration can be determined where practical.
7. The recording was created by the current meeting session.

If validation fails:

```text
Do not call OpenAI
        ↓
Show useful error
        ↓
Keep local audio
```

---

# 7. Transcription Service

Create a dedicated service:

```text
TranscriptionService
```

Responsibilities:

- Accept audio.
- Validate input.
- Select configured model.
- Send transcription request.
- Receive result.
- Normalize the result.
- Return structured transcript data.
- Report errors.
- Support retry where appropriate.

It should not:

- Manage the Teams meeting.
- Start/stop audio capture.
- Generate MOM.
- Modify manual notes.
- Control the UI directly.

---

# 8. Transcript Data Model

The application should store more than one plain text string where practical.

Suggested structure:

```json
{
  "id": "transcript-id",
  "meeting_id": "meeting-id",
  "text": "Full transcript...",
  "language": "en",
  "duration_seconds": 3600,
  "created_at": "2026-09-11T12:00:00Z",
  "model": "configured-model",
  "segments": []
}
```

The exact fields may evolve during implementation.

---

# 9. Transcript Segments

If the transcription API provides segment information, retain it.

Example:

```json
{
  "start": 125.4,
  "end": 138.2,
  "text": "Let's review the architecture."
}
```

Segments can later support:

- Timestamp display.
- Navigation.
- Better transcript review.
- More precise MOM verification.

If segments are not available, the application should still work with full transcript text.

---

# 10. Speaker Identification

Speaker identification is desirable but not a hard V1 requirement.

The system must never invent speaker identities.

Valid:

```text
Speaker information provided by transcription system
```

Invalid:

```text
Speaker 1 = Michael
```

unless the application has reliable evidence that Speaker 1 is Michael.

If speaker labels are unavailable, store the transcript chronologically without fabricated names.

---

# 11. Language

The initial target should be English.

The application may allow automatic language detection if supported.

Future versions may support additional languages.

The system should not force English interpretation onto clearly non-English audio.

---

# 12. Technical Vocabulary

Meeting conversations may contain:

- Product names.
- Technology names.
- Acronyms.
- Internal terminology.
- Customer names.
- Project names.
- Architecture terminology.

The transcript should preserve terminology as accurately as possible.

The application should not aggressively rewrite the raw transcript.

For example:

```text
Raw transcript:
"Let's review the SHS service portfolio."

MOM:
"Reviewed the SHS service portfolio."
```

The second is an AI-generated summary, not a replacement for the raw transcript.

---

# 13. Raw vs Processed Transcript

Maintain a distinction:

```text
RAW TRANSCRIPT
     ↓
Optional display formatting
     ↓
MOM generation
```

The original transcription result should remain recoverable.

Do not overwrite the raw transcript with AI-cleaned text.

---

# 14. Transcript Display

The meeting workspace should eventually provide a transcript area.

Example:

```text
-----------------------------------------
 TRANSCRIPT
-----------------------------------------

10:02
Let's start with the service review.

10:05
The current portfolio has three major gaps.

10:08
We need to agree on ownership.
```

Timestamp display is optional for the first UI implementation if timestamps are not available.

---

# 15. Transcript Status

The meeting/session should expose a clear transcription state.

Recommended states:

```text
NOT_STARTED
QUEUED
TRANSCRIBING
COMPLETED
FAILED
RETRYING
```

Example:

```text
Meeting ended
     ↓
TRANSCRIBING
     ↓
COMPLETED
```

---

# 16. User Experience

When transcription begins, show:

> **Transcribing your meeting...**

The UI should remain usable.

The user should be able to see that processing is underway.

When complete:

> **Transcript ready**

If it fails:

> **Transcription could not be completed. Your meeting audio is still saved locally.**

---

# 17. Progress

Exact percentage progress may not be available from the transcription API.

Do not display a fake percentage.

Use status-based progress instead:

```text
Preparing audio...
Uploading audio...
Transcribing...
Saving transcript...
Transcript ready
```

If reliable progress information becomes available later, a percentage can be added.

---

# 18. Large Audio Files

Long meetings may create large recordings.

The transcription service should support segmentation when required.

Possible process:

```text
Full audio
    ↓
Check size/duration
    ↓
Within supported limit?
    ├── Yes → Transcribe directly
    │
    └── No → Split into chunks
                  ↓
             Transcribe each
                  ↓
             Reassemble
```

The exact limits must be taken from the current OpenAI API documentation during implementation.

Do not hard-code outdated limits into the architecture.

---

# 19. Chunking

If audio must be split:

- Preserve chronological order.
- Use consistent overlap only when technically necessary.
- Avoid duplicated sentences.
- Avoid missing words at boundaries.
- Keep chunk metadata.
- Preserve the relationship to the original recording.

Example:

```text
Chunk 1: 00:00–20:00
Chunk 2: 19:55–40:00
Chunk 3: 39:55–60:00
```

If overlap is used, the final assembly logic must remove duplicated text.

Chunking should only be introduced when needed.

---

# 20. Transcript Assembly

For multi-chunk transcription:

```text
Chunk 1 transcript
        +
Chunk 2 transcript
        +
Chunk 3 transcript
        ↓
Ordered transcript
```

The final transcript should read naturally without obvious chunk boundaries.

The original chunk results should remain available during processing if needed for troubleshooting.

---

# 21. API Failure Handling

Common failures include:

- Invalid API key.
- Missing API access/billing.
- Network unavailable.
- Request timeout.
- Rate limit.
- Temporary service failure.
- Invalid model.
- Unsupported audio.
- File-size limitation.

The application should map technical errors into understandable user messages.

Example:

```text
Technical:
HTTP 429

User:
"Transcription is temporarily busy. Please try again."
```

Do not expose raw stack traces to normal users.

---

# 22. Retry Policy

Retry only errors that are reasonably transient.

Good candidates:

- Temporary network errors.
- Temporary service errors.
- Rate limits where retry-after behavior permits.

Do not automatically retry:

- Invalid API key.
- Unsupported file.
- Invalid configuration.
- Permanently invalid request.

Suggested maximum:

```text
3 attempts
```

with increasing delay.

Exact values may be adjusted during implementation.

---

# 23. Offline Behavior

If the computer has no internet connection:

```text
Audio recording
      ↓
Saved locally
      ↓
Transcription unavailable
```

The application should preserve the meeting session.

When connectivity is restored, the user can retry transcription.

Do not delete audio because transcription could not start.

---

# 24. Re-Transcription

The user may need to retry transcription.

The application should avoid duplicate unnecessary API calls.

Example:

```text
Audio
 ↓
Transcript completed
```

If the transcript already exists, do not automatically transcribe again.

Provide an explicit retry/re-transcribe action where appropriate.

---

# 25. Transcript Storage

The transcript should be saved locally with the meeting.

Suggested structure:

```text
Meetings/
  2026-09-11_1000_Service_Review/
    audio/
      meeting.wav
    transcript/
      transcript.json
      transcript.md
    notes/
      notes.md
    mom/
      mom.json
      mom.md
```

The exact folder structure is defined in the local-storage specification.

---

# 26. Markdown Transcript

A human-readable Markdown version is useful.

Example:

```markdown
# Meeting Transcript

**Date:** 2026-09-11

## Transcript

Let's start with the service review.

The current portfolio has three major gaps.

We need to agree on ownership.
```

This is in addition to structured storage where required.

---

# 27. Transcript and Manual Notes

Manual notes are separate from the transcript.

Do not merge them into the raw transcript.

Use:

```text
Transcript
    +
Manual Notes
    ↓
MOM Generation
```

This preserves the distinction between:

- What the transcription system heard.
- What the user personally wrote.

---

# 28. Transcript Quality Rules

The system should favor:

- Accuracy.
- Completeness.
- Chronological order.
- Preservation of terminology.
- Preservation of meaningful statements.

It should not favor aggressive summarization.

Summarization belongs to MOM generation.

---

# 29. Empty or Poor Audio

Possible situation:

```text
Audio file exists
but contains little/no speech
```

The application should not fabricate a transcript.

Possible result:

```text
No meaningful speech detected.
```

The original audio should remain available.

---

# 30. Capture Quality Dependency

Transcription quality depends on audio quality.

The transcription service must not attempt to compensate for a fundamentally broken audio-capture implementation.

If the recording contains no Teams speech, transcription cannot recover speech that was never captured.

Therefore:

```text
Audio Capture Quality
        ↓
Transcription Quality
        ↓
MOM Quality
```

This dependency must remain visible during debugging.

---

# 31. Debugging Workflow

When transcript quality is poor, investigate in this order:

### 1. Was Teams audio actually captured?

Play the recording.

### 2. Is the recording complete?

Check duration and file integrity.

### 3. Is the audio understandable?

Listen for clipping, gaps, or distortion.

### 4. Is the correct transcription model configured?

Verify configuration.

### 5. Is the transcript accurate?

Compare selected portions with the recording.

### 6. Only then investigate prompt/MOM issues.

Do not try to solve an audio-capture problem by changing the MOM prompt.

---

# 32. Performance

Transcription should run asynchronously.

The UI must not freeze while transcription is running.

Conceptually:

```text
UI
 │
 ├── Start transcription
 │
 └── Remains responsive
          ↓
     Background request
          ↓
       Result
```

The application should release temporary resources after processing.

---

# 33. Security and Privacy

The transcription service must:

- Use HTTPS/API-secure communication.
- Never log the API key.
- Never expose the API key in the UI.
- Avoid writing raw audio into logs.
- Preserve local audio independently of API success/failure.

Only the selected meeting recording should be sent for transcription.

---

# 34. API Key Validation

When the user configures the API key, the application may provide:

**Test Connection**

The test should confirm that the configured key can make an appropriate API request.

Do not expose the actual key after entry.

Display:

```text
OpenAI connection successful
```

or:

```text
OpenAI connection failed
```

---

# 35. Model Configuration

The transcription model should be configurable.

Example:

```json
{
  "transcription_model": "gpt-4o-mini-transcribe"
}
```

This is a configuration example only.

Model names and availability can change.

The implementation must verify current OpenAI model/API documentation before finalizing production defaults.

---

# 36. Cost Awareness

Each transcription request can consume API usage.

The application should:

- Transcribe only when the user starts a session.
- Avoid duplicate transcription.
- Avoid unnecessary retries.
- Store completed transcripts locally.
- Allow explicit re-transcription.

For long meetings, users should understand that longer audio can result in higher API usage.

---

# 37. Testing Matrix

| Test | Expected Result |
|---|---|
| 5-minute meeting | Transcript generated |
| 30-minute meeting | Transcript generated |
| 60-minute meeting | Transcript generated |
| 2-hour meeting | Transcript generated or correctly chunked |
| English meeting | Accurate English transcript |
| Technical terminology | Terms reasonably preserved |
| Manual notes present | Notes remain separate |
| No manual notes | Transcript still works |
| API unavailable | Audio preserved |
| Invalid API key | Clear configuration error |
| Network interruption | Retry/preserve audio |
| Empty audio | No invented transcript |
| Re-transcription | Explicit user action required |
| Existing transcript | No unnecessary automatic duplicate call |

---

# 38. Acceptance Criteria

The transcription implementation is complete for V1 when:

1. A finalized Teams meeting recording can be submitted.
2. A transcript can be generated through the configured OpenAI transcription service.
3. The transcript is saved locally.
4. The raw transcript remains recoverable.
5. Long recordings are handled safely.
6. Transcription does not freeze the UI.
7. API failures are understandable to the user.
8. Meeting audio remains available after transcription failure.
9. Retry works for appropriate transient failures.
10. Manual notes remain separate from the raw transcript.
11. Speaker names are never invented.
12. The system does not fabricate speech when audio is empty or unclear.
13. The transcript can be passed reliably to MOM generation.

---

# 39. Implementation Order

Implement in this order:

### P0 — Basic transcription

- Create TranscriptionService.
- Configure OpenAI API key.
- Configure transcription model.
- Submit finalized WAV/audio file.
- Receive transcript.
- Save transcript locally.
- Display transcript.

### P1 — Reliability

- Validation.
- Error handling.
- Retry.
- Large-file handling.
- Better status reporting.
- Transcript segments where available.

### P2 — Advanced

- Near-live transcription.
- Speaker diarization.
- Timestamp navigation.
- Advanced transcript search.

---

# 40. Definition of Done

The transcription feature is considered done when a real Teams meeting can follow this complete path:

```text
Teams meeting
     ↓
User starts note-taking
     ↓
Audio captured
     ↓
Meeting ends
     ↓
Audio finalized
     ↓
Transcription starts
     ↓
Transcript generated
     ↓
Transcript saved locally
     ↓
Transcript displayed
     ↓
Transcript available for MOM generation
```

The most important rule is:

> **Never lose the original meeting audio simply because transcription failed.**

---

# 41. Final Principle

The transcript is a source artifact.

The MOM is an interpretation of that source.

Therefore:

```text
Audio
  ↓
Transcript
  ↓
MOM
```

Each layer should remain independently recoverable in V1.
