# Notes and MOM Specification

## Meeting Notes Assistant — V1

**Document Status:** Implementation specification  
**Purpose:** Define manual notes, transcript review, AI-generated Minutes of Meeting (MOM), editing, and local persistence.

---

# 1. Purpose

Meeting Notes Assistant combines two sources of meeting information:

1. The **transcript** generated from captured Teams meeting audio.
2. The **personal notes** entered by the user during the meeting.

These are then used to generate a professional, editable:

**Minutes of Meeting (MOM)**

The core flow is:

```text
Teams Audio
    ↓
Transcript
    +
User Notes
    ↓
AI MOM Generation
    ↓
Editable MOM
    ↓
Save / Copy
```

---

# 2. Core Principle

The application must keep three artifacts separate:

```text
1. Original Audio
2. Raw Transcript
3. Personal Notes
```

The AI-generated MOM is a fourth artifact derived from them.

```text
Audio
  ↓
Transcript ──────┐
                 ├──→ MOM
Manual Notes ────┘
```

The AI must never overwrite the original transcript or the user's personal notes.

---

# 3. Personal Notes

The user should be able to type notes while the meeting is happening.

Example:

```text
Need architecture review next week.

Ask finance for updated numbers.

Michael to confirm timeline.

Customer expects decision by Friday.
```

Notes are intentionally simple.

V1 does not require:

- Rich text editing.
- Complex formatting.
- Tags.
- Categories.
- Templates.
- Voice notes.
- Collaborative editing.

A basic text editor is sufficient.

---

# 4. Notes Auto-Save

Manual notes should be saved locally while the meeting is active.

The application should not depend on the user pressing Save every time.

Suggested behavior:

```text
User types
   ↓
Short delay
   ↓
Auto-save locally
```

If the application crashes, already saved notes should remain available.

The exact debounce interval can be decided during implementation.

---

# 5. Notes Storage

Notes should belong to a specific meeting session.

Conceptually:

```text
Meeting
   └── notes.md
```

The note file should be readable independently of the application.

Example:

```markdown
# Meeting Notes

- Discussed service portfolio ownership.
- Need follow-up with architecture team.
- Confirm target date.
```

---

# 6. Transcript Review

After transcription, the user should be able to view the transcript.

The transcript is primarily a source/reference artifact.

The user may review it before generating the MOM.

V1 does not require sophisticated transcript editing.

If editing is supported later, preserve the original transcription separately.

---

# 7. MOM Generation

The user should have a clear action:

**Generate MOM**

The generation process uses:

```text
Transcript
+
Manual Notes
+
Basic meeting metadata
```

and sends the relevant content to the configured OpenAI generation model.

---

# 8. MOM Generation Timing

MOM generation should normally occur after:

1. Meeting ends.
2. Audio is finalized.
3. Transcription completes.
4. Transcript is saved.

Then:

```text
Transcript ready
      ↓
Generate MOM
```

The user may also trigger generation manually.

---

# 9. MOM Draft Status

The application should distinguish between:

```text
MOM NOT GENERATED
MOM GENERATING
MOM READY
MOM EDITED
MOM GENERATION FAILED
```

The user should always know whether they are looking at:

- AI-generated draft, or
- Their edited final version.

---

# 10. Recommended MOM Structure

The default MOM should use the following structure:

```text
Meeting Title

Date & Time

Participants

Purpose / Context

Key Discussion Points

Decisions Made

Action Items

Open Questions / Follow-ups

Next Steps
```

Not every section needs to contain content.

Empty sections may be omitted or marked as:

```text
Not specified
```

The final formatting should remain clean and professional.

---

# 11. Meeting Title

The title may come from:

- Teams meeting information, if reliably available.
- User-entered title.
- AI-generated title based on the discussion.

For V1, do not depend on deep Teams integration.

If the title is unavailable, use a simple fallback such as:

```text
Meeting — 11 Sep 2026
```

---

# 12. Date and Time

Use locally available application/session information.

Do not invent dates or times from the transcript.

If the meeting date/time is known:

```text
Date: 11 September 2026
Time: 10:00 AM
```

If unavailable, omit the field or mark it as not specified.

---

# 13. Participants

Participants should only be listed when reliably known.

Potential sources:

- Teams meeting metadata if available through supported Windows/application observation.
- User-provided information.
- Reliable speaker identification from transcription, if available.

Do not infer participants merely because their names appear in a conversation.

Example:

> "Let's ask John about this."

does not automatically prove John attended the meeting.

---

# 14. Purpose / Context

The AI should summarize the purpose of the meeting using the transcript and notes.

Example:

```text
Purpose:
Review the current service portfolio and agree on priorities
for improving subscription service delivery.
```

Keep this concise.

---

# 15. Key Discussion Points

Capture meaningful topics discussed during the meeting.

Example:

```text
- Current service portfolio structure.
- Gaps in end-to-end ownership.
- Customer impact of fragmented delivery.
- Proposed service portfolio management model.
```

Do not reproduce the entire conversation.

---

# 16. Decisions Made

This section must be conservative.

Only record decisions that were actually agreed.

Example:

```text
Decision:
Establish a cross-functional review of the service portfolio.
```

Do not convert:

```text
"We could establish..."
```

into:

```text
Decision:
Establish...
```

unless the meeting later confirms the decision.

---

# 17. Action Items

Action items should be represented clearly.

Recommended structure:

| Action | Owner | Due Date | Status |
|---|---|---|---|
| Confirm service ownership | John | Friday | Open |

For V1, the underlying data may be stored as structured JSON while the UI displays a table.

---

# 18. Action Item Rules

The AI should identify:

- What needs to be done.
- Who is responsible.
- When it is due.
- Whether it was explicitly committed.

Example:

Transcript:

> "I'll send the updated proposal by Thursday."

MOM:

```text
Action: Send updated proposal
Owner: Speaker/identified participant
Due: Thursday
```

If owner is not reliably identifiable:

```text
Owner: Not specified
```

Do not guess.

---

# 19. Suggestions vs Commitments

This distinction is critical.

### Suggestion

> "We should review this with Finance."

Do not automatically create an action item.

### Commitment

> "I'll review this with Finance tomorrow."

This can become an action item.

The AI prompt should explicitly enforce this distinction.

---

# 20. Open Questions

Capture unresolved questions.

Example:

```text
- Who will own the service portfolio?
- What is the target implementation date?
- Which teams need to participate?
```

These are useful when a meeting ends without resolution.

---

# 21. Next Steps

The next-steps section should summarize the immediate continuation of the work.

It may overlap with action items but should remain higher-level.

Example:

```text
1. Confirm ownership model.
2. Review proposal with architecture.
3. Schedule follow-up meeting.
```

---

# 22. AI Generation Rules

The MOM generation model must follow these principles:

1. Use only supplied transcript, notes, and reliable metadata.
2. Do not invent facts.
3. Do not invent decisions.
4. Do not invent action owners.
5. Do not invent deadlines.
6. Do not invent participants.
7. Distinguish discussion from decision.
8. Distinguish suggestions from commitments.
9. Preserve important terminology.
10. Prefer concise professional language.
11. Resolve obvious repetition.
12. Preserve meaningful context.
13. Clearly indicate uncertainty.
14. Never present assumptions as facts.

---

# 23. Hallucination Prevention

The AI must not create information simply because a MOM normally contains it.

For example, if there is no due date:

```text
Due Date: Not specified
```

not:

```text
Due Date: Next Friday
```

If no owner is known:

```text
Owner: Not specified
```

not:

```text
Owner: Project Manager
```

---

# 24. Transcript and Notes Weighting

The transcript represents the meeting conversation.

The manual notes represent the user's own observations.

Neither should automatically be considered more authoritative in every situation.

The model should use both and resolve obvious conflicts conservatively.

If there is a meaningful contradiction, the MOM should avoid pretending certainty.

---

# 25. Manual Notes as Context

Manual notes may contain shorthand.

Example:

```text
Arch review next week
Finance number
Vikram timeline?
```

The AI may interpret these in the context of the transcript.

However, it should not expand shorthand into unsupported commitments.

For example:

```text
Vikram timeline?
```

should remain an open question rather than:

```text
Action: Vikram to provide timeline.
```

unless the transcript confirms it.

---

# 26. MOM Prompt

The generation prompt should contain explicit instructions similar to:

```text
You are generating professional Minutes of Meeting.

Use only the supplied meeting transcript, manual notes,
and reliable meeting metadata.

Do not invent:
- facts
- decisions
- participants
- owners
- deadlines
- commitments

Distinguish:
- discussion
- decision
- suggestion
- confirmed action
- open question

If information is unavailable, mark it as not specified.

Produce a concise, professional MOM suitable for
human review and editing.
```

The final production prompt should be version-controlled.

---

# 27. Structured MOM Data

Where supported by the selected OpenAI model/API, use structured output.

Suggested internal representation:

```json
{
  "title": "",
  "date": "",
  "time": "",
  "participants": [],
  "purpose": "",
  "discussion_points": [],
  "decisions": [],
  "action_items": [
    {
      "action": "",
      "owner": "",
      "due_date": "",
      "status": "Open"
    }
  ],
  "open_questions": [],
  "next_steps": []
}
```

The exact schema can evolve during implementation.

---

# 28. MOM Markdown

The application should also produce a readable Markdown representation.

Example:

```markdown
# Minutes of Meeting

**Date:** 11 September 2026

## Purpose

Review the current service portfolio.

## Key Discussion Points

- Current portfolio structure.
- Ownership gaps.
- Customer impact.

## Decisions Made

- Establish a cross-functional service portfolio review.

## Action Items

| Action | Owner | Due Date |
|---|---|---|
| Confirm service ownership | John | Friday |

## Open Questions

- What is the target implementation date?

## Next Steps

- Schedule architecture review.
```

---

# 29. Editable MOM

The generated MOM is always a **draft**.

The user must be able to modify it.

Required capabilities:

- Edit headings.
- Edit paragraphs.
- Edit bullet points.
- Correct names.
- Correct dates.
- Add missing actions.
- Remove incorrect information.
- Copy the final MOM.
- Save the final MOM.

---

# 30. Protecting User Edits

Once the user edits the MOM:

```text
AI Draft
   ↓
User edits
   ↓
Final Draft
```

The application must not automatically regenerate the MOM and erase those changes.

If regeneration is requested:

> **Your MOM has been edited. Regenerating may replace your changes. Continue?**

A simpler implementation may instead create a new generated version while preserving the previous one.

---

# 31. MOM Versioning

V1 does not need sophisticated version history.

At minimum, preserve:

```text
mom_generated
mom_final
```

This allows the original AI draft to remain separate from the user's final version.

---

# 32. Copy Function

Provide a simple:

**Copy MOM**

button.

The copied result should be clean text/Markdown suitable for pasting into:

- Email.
- Teams chat.
- Word.
- OneNote.
- Other documentation tools.

No external integrations are required.

---

# 33. Save Function

Provide:

**Save MOM**

The final version should be stored locally.

Suggested files:

```text
mom.json
mom.md
```

The Markdown version is human-readable.

The JSON version supports structured editing and future features.

---

# 34. Regenerate Function

Optional V1 feature:

**Regenerate MOM**

Use cases:

- User corrected notes.
- User wants a cleaner summary.
- Previous generation failed.

Regeneration must use the latest saved transcript and notes.

---

# 35. Generation Failure

If MOM generation fails:

```text
Transcript
   ↓
Saved successfully
   ↓
MOM generation fails
```

The transcript must remain available.

Show:

> **MOM generation failed. Your transcript and notes are safe.**

Provide:

```text
[Retry]
```

---

# 36. Empty Transcript

If transcription produced no meaningful content:

```text
No meaningful transcript available.
```

The user may still have manual notes.

In that case:

```text
Manual Notes
    ↓
MOM Generation
```

may be allowed.

If both transcript and notes are empty, do not call the model unnecessarily.

---

# 37. Notes-Only MOM

If the user took detailed manual notes but transcription failed, the application may still generate a MOM from the notes.

The UI should clearly communicate:

> **MOM generated from your manual notes because a transcript was unavailable.**

This prevents the user from assuming the MOM represents the complete spoken meeting.

---

# 38. Transcript-Only MOM

If there are no manual notes:

```text
Transcript
   ↓
MOM
```

This is fully supported.

Manual notes are an enhancement, not a requirement.

---

# 39. Content Length

Long meetings can produce large transcripts.

The application should not blindly send unlimited content into a generation request.

If the transcript exceeds the selected model's supported context:

```text
Long transcript
      ↓
Pre-process / chunk / summarize
      ↓
Final context
      ↓
MOM generation
```

The implementation should use current model context limits rather than hard-coded assumptions.

For V1, a straightforward approach is preferred until real meeting sizes demonstrate the need for additional processing.

---

# 40. Long-Meeting MOM Strategy

If a very long transcript cannot fit in one request, use a staged approach:

```text
Transcript
   ↓
Section summaries
   ↓
Combined meeting summary
   ↓
MOM generation
```

However, do not implement this complexity until required.

---

# 41. Content Preservation

The MOM should preserve important:

- Product names.
- Project names.
- Acronyms.
- Dates.
- Numbers.
- Technical terminology.
- Decisions.
- Action commitments.

The AI may simplify wording but must not change the underlying meaning.

---

# 42. Tone

Default MOM tone:

**Professional, concise, neutral.**

Avoid:

- Marketing language.
- Excessive enthusiasm.
- Emotional interpretation.
- Unnecessary verbosity.
- Psychological analysis.
- Sentiment analysis.

The tool is a meeting-notes assistant, not a meeting-analytics system.

---

# 43. No Psychological Inference

The MOM generator must not infer:

- Personality.
- Emotions.
- Intentions.
- Motivation.
- Conflict.
- Leadership quality.

unless the user explicitly adds a future feature requiring such analysis.

V1 remains focused on factual meeting documentation.

---

# 44. Local Persistence

The following should remain locally available:

```text
Audio
Transcript
Manual Notes
MOM Draft
Final MOM
```

The OpenAI API is used as a processing service, not as the application's permanent data store.

---

# 45. Complete Workflow

The intended end-to-end flow is:

```text
1. Teams meeting detected
          ↓
2. User selects Start Taking Notes
          ↓
3. Audio capture starts
          ↓
4. User types notes
          ↓
5. Meeting ends
          ↓
6. Audio finalized
          ↓
7. Transcript generated
          ↓
8. Transcript saved
          ↓
9. User reviews transcript/notes
          ↓
10. Generate MOM
          ↓
11. AI creates MOM draft
          ↓
12. User edits MOM
          ↓
13. User saves/copies final MOM
```

---

# 46. Acceptance Criteria

The Notes and MOM feature is complete for V1 when:

1. Users can enter manual notes during a meeting.
2. Notes are automatically saved locally.
3. Transcript and notes remain separate.
4. Users can review the transcript.
5. Users can generate an MOM from transcript + notes.
6. MOM output has a consistent professional structure.
7. Decisions are not fabricated.
8. Action items are not fabricated.
9. Owners and deadlines are not guessed.
10. The MOM is editable.
11. User edits are protected from accidental regeneration.
12. Final MOM can be saved locally.
13. Final MOM can be copied.
14. Transcript remains available if MOM generation fails.
15. Notes remain available if transcription fails.
16. Notes-only MOM generation can work when appropriate.
17. Empty input does not trigger unnecessary AI calls.

---

# 47. Implementation Priority

## P0

- Manual notes editor.
- Notes auto-save.
- Transcript display.
- Generate MOM action.
- MOM generation service integration.
- Structured MOM data.
- Editable MOM.
- Save MOM.
- Copy MOM.

## P1

- MOM regeneration.
- Draft/final separation.
- Action-item table.
- Timestamp-linked transcript.
- Better long-transcript handling.

## P2

- Multiple MOM templates.
- Custom prompt styles.
- Advanced formatting.
- Export formats.
- Speaker-linked action items.

---

# 48. Definition of Done

A meeting is successfully processed when:

```text
Audio captured
      ↓
Transcript available
      ↓
Manual notes available
      ↓
MOM generated
      ↓
User reviews
      ↓
User edits
      ↓
Final MOM saved locally
```

Every source artifact remains recoverable.

---

# 49. Final Product Principle

The AI should do the difficult work of turning a long meeting into a useful first draft.

The user remains the final authority.

Therefore:

> **AI generates. User reviews. User decides.**

The application should make the path from meeting conversation to trustworthy, editable MOM as simple as possible.
