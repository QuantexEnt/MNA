# OpenAI Integration Specification

## Meeting Notes Assistant — V1

**Document Status:** Technical implementation specification  
**Purpose:** Define how Meeting Notes Assistant uses the OpenAI API for transcription and Minutes of Meeting (MOM) generation.

---

# 1. Purpose

OpenAI is used for two primary tasks:

1. Convert captured Teams meeting audio into text.
2. Convert the transcript and user's personal notes into an editable MOM.

The application does **not** need separate OpenAI API keys for these tasks.

## V1 principle

> **One OpenAI API key can be used for both transcription and MOM generation.**

Different API models can be selected for the two jobs.

---

# 2. OpenAI Responsibilities

The application should keep OpenAI responsibilities narrowly defined.

```text
Teams Audio
    ↓
Local Audio Capture
    ↓
OpenAI Transcription
    ↓
Transcript
    +
Manual Notes
    ↓
OpenAI MOM Generation
    ↓
Editable MOM
```

OpenAI should not be responsible for:

- Detecting Teams meetings.
- Capturing Windows audio.
- Storing application data.
- Managing the UI.
- Managing Windows processes.
- Starting/stopping recording.
- Maintaining application state.

---

# 3. API Key

The application requires one OpenAI API key.

Example configuration concept:

```text
OPENAI_API_KEY
```

The same key can authorize:

```text
Transcription API call
        +
MOM generation API call
```

No second key is required merely because two different models are used.

---

# 4. ChatGPT Subscription vs API

The application uses the **OpenAI API**, not the user's ChatGPT web session.

Important:

```text
ChatGPT subscription
        ≠
OpenAI API billing
```

Having access to a model in ChatGPT does not by itself mean the desktop application can call that model through the API.

The application must use a valid API key with API billing/access configured.

---

# 5. Model Selection

Model IDs should be configurable rather than hard-coded throughout the application.

Recommended configuration concept:

```json
{
  "transcription_model": "gpt-4o-mini-transcribe",
  "mom_model": "gpt-5.6-luna"
}
```

These are example defaults, not permanent product requirements.

The application must isolate model names in configuration so they can be changed without modifying application logic.

OpenAI currently lists specialized transcription models including **GPT-4o Mini Transcribe**, **GPT-4o Transcribe**, and **GPT-Transcribe**. Model availability and pricing can change, so the implementation must verify the current model documentation when development begins.

---

# 6. Transcription

## 6.1 Input

The transcription service receives the finalized local audio recording.

```text
audio.wav
    ↓
TranscriptionService
    ↓
Transcript
```

The capture module must complete or finalize the audio before a standard post-meeting transcription request is made.

---

# 7. Transcription Modes

V1 should support the simplest reliable mode first:

## Post-Meeting Transcription

```text
Meeting
  ↓
Audio captured locally
  ↓
Meeting ends
  ↓
Audio finalized
  ↓
Upload audio
  ↓
Transcription
```

This should be the baseline implementation.

## Near-Live Transcription

Near-live transcription may be added later.

It should not delay the first usable version of the application.

If near-live transcription introduces significant complexity, use post-meeting transcription for V1.

---

# 8. Transcription Service Boundary

Create a dedicated abstraction:

```text
TranscriptionService
```

The rest of the application should not directly construct OpenAI HTTP requests.

Conceptually:

```text
Meeting Session
      ↓
TranscriptionService
      ↓
OpenAI API
      ↓
TranscriptResult
```

This allows the transcription provider/model to change later without redesigning the meeting workflow.

---

# 9. Transcript Result

The transcription service should return structured information rather than only a raw string.

Suggested internal structure:

```json
{
  "text": "Full transcript text...",
  "language": "en",
  "duration_seconds": 3600,
  "segments": []
}
```

Segments are optional for the initial implementation.

If timestamps or speaker information are available, retain them where practical.

---

# 10. Speaker Identification

Do not make advanced speaker diarization a hard V1 dependency.

The system should first produce a reliable transcript.

If the selected transcription capability provides useful speaker/segment information, preserve it.

Otherwise:

```text
Speaker identification unavailable
        ↓
Use chronological transcript
```

Do not invent speaker identities.

Never generate names such as:

> Michael said...

unless the source data actually identifies Michael.

---

# 11. Transcript Accuracy

The application should not silently modify the raw transcript before saving it.

Maintain two conceptual forms:

```text
Raw Transcript
      ↓
AI Processing
      ↓
MOM
```

The original transcript remains available for verification.

The MOM is an interpreted summary and should not replace the source transcript.

---

# 12. Audio Upload

Audio should be uploaded only when required for transcription.

Before upload:

1. Confirm the recording exists.
2. Confirm the recording is finalized.
3. Confirm file size is valid.
4. Confirm the audio format is supported.
5. Start transcription.
6. Record progress/status.
7. Save the resulting transcript locally.

---

# 13. Large Meetings

Long meetings may produce large audio files.

The implementation must not assume that every meeting fits comfortably into one API request.

If file size or API limits require segmentation:

```text
Full recording
     ↓
Chunk 1
Chunk 2
Chunk 3
...
     ↓
Individual transcription
     ↓
Ordered transcript
```

Chunk boundaries should be handled carefully so words are not unnecessarily lost or duplicated.

The exact maximum file/request limits should be read from the current OpenAI API documentation during implementation rather than permanently hard-coded in this specification.

---

# 14. Transcription Errors

Possible errors:

- Invalid API key.
- Insufficient API balance.
- Network failure.
- Unsupported audio format.
- File too large.
- Rate limit.
- Temporary OpenAI service failure.
- Invalid model.
- Request timeout.

The application should display a useful status.

Example:

> **Transcription could not be completed. Your meeting audio is still saved locally.**

The original audio must remain available.

---

# 15. Retry Strategy

Safe transient failures should be retried.

Examples:

- Temporary network failure.
- Temporary service error.
- Rate limiting where a retry is appropriate.

Do not retry indefinitely.

Suggested behavior:

```text
Attempt 1
   ↓
temporary failure
   ↓
wait
   ↓
Attempt 2
   ↓
temporary failure
   ↓
wait
   ↓
Attempt 3
   ↓
failure
   ↓
Show error
```

The exact retry/backoff values should be configurable.

---

# 16. MOM Generation

After transcription:

```text
Transcript
     +
Manual Notes
     ↓
MOM Generation
     ↓
Structured MOM
```

The MOM model should receive enough context to understand the meeting.

Input should normally include:

- Transcript.
- User's manual notes.
- Optional meeting title.
- Optional meeting date/time.
- Optional known participants if the application has reliable data.

Do not send unrelated application data.

---

# 17. MOM Output

The AI should produce a structured meeting summary.

Recommended V1 structure:

```text
Meeting Title

Date

Participants

Purpose / Context

Key Discussion Points

Decisions Made

Action Items
- Action
- Owner
- Due Date

Open Questions / Follow-ups

Next Steps
```

Not every meeting will contain every section.

The AI should use:

> Not mentioned / Not specified

when appropriate rather than inventing information.

---

# 18. Action Item Accuracy

Action items are one of the most important MOM outputs.

The AI must distinguish between:

### Explicit action

> "John will send the report by Friday."

This can become:

```text
Action: Send the report
Owner: John
Due: Friday
```

### Discussion only

> "We should consider sending the report."

This should **not automatically become** a committed action.

The prompt must instruct the model to avoid turning suggestions into commitments.

---

# 19. No Hallucinated Decisions

The AI must not invent:

- Decisions.
- Owners.
- Deadlines.
- Participants.
- Commitments.
- Facts not supported by the transcript/notes.

If something is ambiguous, the MOM should reflect the uncertainty.

---

# 20. Manual Notes

Manual notes are first-class meeting input.

The user may type notes while the meeting is happening.

Example:

```text
Manual Notes:

- Need architecture review next week
- Ask finance for updated number
- Michael to confirm timeline
```

These notes should be included in MOM generation.

However, the model should distinguish personal notes from confirmed statements in the meeting.

---

# 21. Prompt Design

The MOM generation prompt should be stored in source control as a versioned application prompt.

Suggested structure:

```text
SYSTEM INSTRUCTIONS

You are generating a professional Minutes of Meeting.

Rules:
1. Use only information supplied in the transcript and notes.
2. Do not invent decisions, people, owners, or deadlines.
3. Clearly distinguish decisions from discussion.
4. Clearly distinguish confirmed actions from suggestions.
5. Preserve important technical/business terminology.
6. Prefer concise, professional language.
7. If information is unavailable, say so rather than guessing.
8. Produce structured output suitable for editing by the user.
```

The actual prompt should be refined during testing.

---

# 22. Structured Output

The preferred implementation should request structured output where supported by the selected OpenAI API/model.

Conceptually:

```json
{
  "title": "",
  "date": "",
  "participants": [],
  "purpose": "",
  "discussion_points": [],
  "decisions": [],
  "action_items": [],
  "open_questions": [],
  "next_steps": []
}
```

The UI can then render this into an editable MOM.

Do not rely on fragile markdown parsing if structured output is available and reliable.

---

# 23. Editable MOM

The AI-generated MOM is a draft.

The user must be able to:

- Edit text.
- Correct mistakes.
- Add information.
- Remove information.
- Copy the MOM.
- Save the final version locally.

The AI must never overwrite the user's final edited content without explicit action.

---

# 24. Regeneration

V1 may provide:

```text
Generate MOM
Regenerate MOM
```

If regeneration is implemented:

- Preserve the original transcript.
- Preserve manual notes.
- Replace only the generated draft.
- Warn if the user has already edited the current MOM.

Do not silently destroy user edits.

---

# 25. API Architecture

Recommended internal structure:

```text
src/
  services/
    openai/
      OpenAIClient
      TranscriptionService
      MomGenerationService
```

Conceptually:

```text
OpenAIClient
      │
      ├── TranscriptionService
      │
      └── MomGenerationService
```

Both services share the same configured API key.

---

# 26. API Key Storage

For the personal V1 desktop application, the key should be entered/configured locally.

Do not put the key into:

- Git.
- GitHub.
- Source code.
- README files.
- Logs.
- Screenshots.
- Error reports.

Use local configuration or an appropriate Windows secure storage mechanism.

---

# 27. Environment Variables

During development, an environment variable may be used:

```text
OPENAI_API_KEY=...
```

The real value must never be committed.

Example:

```text
.env
```

must be excluded from Git.

The repository should contain only:

```text
.env.example
```

with a placeholder:

```text
OPENAI_API_KEY=your_api_key_here
```

---

# 28. Desktop Security Consideration

A desktop application containing a user's API key cannot provide the same secret protection as a server-side application.

For the personal/local V1 this is acceptable if the application is used only by the owner.

If the application is later distributed to many users commercially, do not embed one shared API key into the application.

A future commercial architecture may require a backend or another secure API-access model.

---

# 29. Privacy

The application should clearly explain:

```text
Meeting audio → OpenAI API for transcription
Transcript + notes → OpenAI API for MOM generation
```

Only the information required for these operations should be transmitted.

Local recordings and transcripts should remain on the user's machine unless the user explicitly chooses another workflow.

---

# 30. Network Dependency

The application should continue to work locally when the internet is unavailable for:

- Meeting detection.
- Audio capture.
- Manual notes.
- Local storage.

OpenAI-dependent functions require internet connectivity.

Therefore:

```text
Internet unavailable
       ↓
Capture still possible
       ↓
Notes still possible
       ↓
Transcription/MOM delayed
```

The application should not discard the meeting because OpenAI is temporarily unavailable.

---

# 31. Offline Recovery

If OpenAI is unavailable after a meeting:

```text
Audio saved locally
Transcript not yet generated
MOM not yet generated
```

The application should allow the user to retry later.

The meeting session should remain in a state such as:

```text
TRANSCRIPTION_PENDING
```

rather than being marked as failed permanently.

---

# 32. Cost Control

The application should avoid unnecessary API calls.

Do not:

- Re-transcribe unchanged audio automatically.
- Regenerate MOM repeatedly without user action.
- Send the same transcript multiple times unnecessarily.
- Upload audio when transcription is not requested.

Store completed results locally.

Example:

```text
Audio
  ↓
Transcribe once
  ↓
Save transcript
  ↓
Generate MOM
  ↓
Save MOM
```

---

# 33. API Usage Logging

Log operational metadata such as:

- Request started.
- Request completed.
- Model used.
- Duration.
- Success/failure.
- Error category.

Do **not** log:

- API key.
- Raw audio.
- Full transcript.
- Full MOM content unless explicitly required for local debugging.

---

# 34. Timeouts

Network operations must have explicit timeout handling.

The UI must never appear frozen because an OpenAI request is waiting.

Use asynchronous operations:

```text
UI
 │
 ├── Start request
 │
 └── Continue responding
          ↓
     OpenAI request
          ↓
     Result/event
```

---

# 35. User Interface Status

The application should expose clear AI processing states.

Example:

```text
Transcribing meeting...
Generating MOM...
MOM ready
Transcription failed
MOM generation failed
Retry available
```

Avoid vague messages such as:

> Something went wrong.

---

# 36. OpenAI Integration Sequence

## Transcription

```text
Meeting ends
     ↓
Finalize audio
     ↓
Validate file
     ↓
Call transcription service
     ↓
OpenAI transcription model
     ↓
Receive transcript
     ↓
Save transcript locally
```

## MOM

```text
Transcript saved
     +
Manual notes
     ↓
Build MOM request
     ↓
OpenAI generation model
     ↓
Receive structured MOM
     ↓
Save draft locally
     ↓
Display editable MOM
```

---

# 37. Separation of Concerns

The following modules should remain independent:

```text
TeamsDetection
AudioCapture
LocalStorage
TranscriptionService
MomGenerationService
UI
```

Do not place OpenAI API calls inside React components.

Do not place audio capture code inside OpenAI services.

Do not place MOM formatting logic inside the audio recorder.

---

# 38. Configuration

Suggested local configuration:

```json
{
  "openai": {
    "api_key": "",
    "transcription_model": "gpt-4o-mini-transcribe",
    "mom_model": "gpt-5.6-luna"
  }
}
```

The API key should preferably be stored using a secure local mechanism rather than plain JSON once the basic prototype is working.

Model names should remain configurable.

---

# 39. Model Upgrade Strategy

Do not scatter model IDs throughout source code.

Bad:

```text
call("gpt-4o-mini-transcribe")
```

in many files.

Preferred:

```text
config.openai.transcription_model
```

This allows model upgrades without broad code changes.

---

# 40. Testing Strategy

## Unit Tests

Test:

- Request construction.
- Prompt construction.
- JSON parsing.
- Error mapping.
- Retry logic.
- Configuration validation.

## Integration Tests

Test:

- Real transcription request.
- Real MOM generation request.
- Invalid API key.
- Network failure.
- Large transcript.
- Long meeting.
- Empty transcript.
- Manual notes only.
- Transcript + notes.

## UI Tests

Verify:

- Progress state.
- Failure state.
- Retry.
- MOM display.
- Editing.
- Save.

---

# 41. Mock Mode

Development should support a mock OpenAI mode.

Example:

```text
OPENAI_MOCK=true
```

In mock mode:

```text
Audio
  ↓
Fake transcript
  ↓
Fake MOM
```

This allows UI development without consuming API credits.

---

# 42. Acceptance Criteria

OpenAI integration is complete for V1 when:

1. One API key can be configured locally.
2. Transcription can be requested successfully.
3. The resulting transcript is saved locally.
4. MOM generation can use transcript + manual notes.
5. MOM output is structured and editable.
6. API failures are clearly reported.
7. Audio remains available when transcription fails.
8. Transcript remains available when MOM generation fails.
9. API keys are never committed to source control.
10. API requests do not block the UI.
11. Model IDs are configurable.
12. Unnecessary repeat API calls are avoided.
13. The application can retry appropriate transient failures.

---

# 43. Implementation Priority

Implement in this order:

### P0

- API key configuration.
- OpenAI client.
- Transcription service.
- Local transcript storage.
- MOM generation service.
- Structured MOM output.
- Error handling.

### P1

- Retry.
- Progress reporting.
- Mock mode.
- Configurable models.
- Better transcript segments/timestamps.

### P2

- Near-live transcription.
- Advanced speaker identification.
- More sophisticated prompt controls.

---

# 44. Important Product Boundary

OpenAI is an AI processing service, not the application's backend.

The application remains:

```text
Local Windows Application
        │
        ├── Local audio
        ├── Local notes
        ├── Local transcript
        ├── Local MOM
        │
        └── OpenAI API
              ├── Transcription
              └── MOM generation
```

No cloud database is required for V1.

---

# 45. Current OpenAI Model Reference

Model names, capabilities, pricing, limits, and API behavior can change.

At implementation time, consult the current official OpenAI API documentation before finalizing the model configuration.

The current OpenAI model catalog lists dedicated transcription models, including GPT-4o Mini Transcribe, GPT-4o Transcribe, and GPT-Transcribe. It also lists current general-purpose models that can be used for text generation. The application should therefore treat model selection as configuration rather than a permanent architectural dependency.

Reference:

https://platform.openai.com/docs/models

---

# 46. Final Rule

The OpenAI integration should remain:

**Simple, replaceable, observable, and local-first.**

The application should capture and preserve the meeting locally first, then use OpenAI to transform that source material into useful text.

Never make an OpenAI API failure equivalent to losing the meeting.
