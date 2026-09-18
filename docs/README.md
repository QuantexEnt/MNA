# Meeting Notes Assistant

## 1. Project Overview

Meeting Notes Assistant is a lightweight Windows desktop application designed to help a user capture meeting transcripts and personal notes during Microsoft Teams meetings and generate a clean Minutes of Meeting (MOM) at the end.

The application is intentionally simple.

It is **not** intended to be a meeting bot, Teams participant, meeting-management platform, collaboration suite, or cloud-based knowledge system.

The application runs locally on the user's Windows computer and operates outside the Teams meeting.

---

## 2. Core Objective

The application should make this workflow as simple as possible:

**Detect Teams meeting → Ask whether to take notes → Capture meeting audio → Transcribe → Add personal notes → Generate MOM → Review/Edit → Save**

The user should not have to add a bot, participant, extension, or application to the Teams meeting.

---

## 3. Target Platform

### Primary platform

- Windows 10/11 desktop
- Microsoft Teams desktop application

### Initial scope

The first version is intended for personal/local use on Windows.

Other operating systems and meeting platforms are outside the initial scope.

---

## 4. Core User Experience

The application runs quietly in the Windows background.

When it detects that the user has entered an active Microsoft Teams meeting, it presents a small notification/popup:

> **Teams meeting detected. Would you like to take notes?**

Options:

- **Start Taking Notes**
- **Not Now**

If the user chooses **Start Taking Notes**, the application begins capturing the meeting audio.

The application should capture the audio being transmitted/played by the Teams meeting rather than independently recording everything spoken into the user's microphone.

Therefore:

- Other participants' voices should be captured.
- The user's voice should be captured when the user is actually speaking in the Teams meeting.
- If the user is muted and speaks locally, that speech should not be captured as meeting audio.
- Headphones should be supported.
- Unrelated system audio should be avoided where technically possible.

During the meeting, the user can:

- See the transcript.
- Add their own notes.
- Stop/end note taking.

At the end of the meeting, the application uses:

**Transcript + User Notes**

to generate a structured MOM.

The user can then:

- Review the MOM.
- Edit the MOM.
- Copy the MOM.
- Save/export the MOM locally.

---

## 5. V1 Scope

The first usable version should contain only the following major capabilities:

1. Windows desktop application.
2. Background operation.
3. Microsoft Teams meeting detection.
4. Meeting-detected notification.
5. Start/stop note-taking session.
6. Teams meeting audio capture.
7. Audio-to-text transcription.
8. Live or near-live transcript display, subject to the selected transcription architecture.
9. Manual user notes.
10. Transcript and notes review.
11. AI-generated MOM.
12. Editable MOM.
13. Local saving of meeting information.
14. Basic meeting history.
15. Basic application settings.

The implementation should remain lightweight and focused.

---

## 6. Explicitly Out of Scope for V1

Do not add the following unless explicitly requested later:

- Teams bot
- Teams meeting participant/bot account
- Teams app/add-in
- Zoom integration
- Google Meet integration
- Webex integration
- Calendar integration
- Slack integration
- Jira integration
- Azure DevOps integration
- Cloud storage
- Cloud database
- User accounts
- Multi-user collaboration
- Subscription management
- Complex project management
- Advanced analytics
- Sentiment analysis
- Psychological analysis
- Enterprise administration
- Cross-device synchronization
- Mobile application
- Browser extension
- Large knowledge base
- Complex meeting dashboards
- Automatic task-management integrations

The product should remain a **simple personal meeting transcript and MOM assistant**.

---

## 7. High-Level Architecture

The initial architecture should be local-first:

```text
+-----------------------------+
|       Windows Laptop        |
|                             |
|  +-----------------------+  |
|  | Meeting Notes App     |  |
|  +-----------+-----------+  |
|              |              |
|      Teams Meeting         |
|              |              |
|      Audio Capture         |
|              |              |
|       Transcription        |
|              |              |
|       +------+-------+      |
|       |              |      |
|   Transcript     My Notes  |
|       |              |      |
|       +------+-------+      |
|              |              |
|         OpenAI API          |
|              |              |
|         MOM Generation      |
|              |              |
|       Local File Storage    |
+-----------------------------+
```

No application server or cloud database is required for V1.

---

## 8. Initial Technology Direction

The current preferred technology stack is:

- **Tauri** for the Windows desktop application shell.
- **React** for the user interface.
- **TypeScript** for frontend application logic.
- **Rust** for desktop/native functionality.
- **Windows APIs** for operating-system integration.
- **WASAPI / Windows Process Loopback Capture** for meeting audio capture where supported and appropriate.
- **Local files / JSON / text files** for initial persistence.
- **OpenAI API** for transcription and MOM generation.
- **Git/GitHub** for source control.

Technology choices should remain replaceable where practical. Do not introduce additional infrastructure unless a documented requirement requires it.

---

## 9. AI Usage

The application will use a single OpenAI API key for its API access.

Different OpenAI models/capabilities may be used for different tasks.

### Transcription

Meeting audio is sent to an appropriate OpenAI speech-to-text/transcription capability.

### MOM generation

The resulting transcript and the user's manually entered notes are provided to an appropriate OpenAI text-generation model to create the MOM.

The application should keep these two responsibilities separate in the code so that models can be changed later without redesigning the application.

---

## 10. Local-First Principle

The application should store meeting data locally by default.

The initial design should avoid:

- Mandatory cloud accounts.
- Mandatory cloud storage.
- Mandatory application backend.
- Mandatory database server.

The user should be able to find their meeting data on their own computer.

Cloud functionality may be considered in a future version, but it is not part of the initial product.

---

## 11. Privacy Principle

Meeting audio and transcripts can contain confidential business information.

Therefore:

- Recording should start only after the user explicitly chooses to take notes.
- The application should clearly indicate when recording/capture is active.
- The application should not silently record meetings.
- The application should not join the Teams meeting.
- The application should not request Teams credentials for the basic workflow.
- Temporary audio/transcription files should be handled carefully.
- API credentials must not be exposed unnecessarily.
- Data sent to external AI services should be clearly understood by the user.
- Local meeting data should remain under the user's control.

The detailed security and privacy requirements are defined separately in `docs/SECURITY_AND_PRIVACY.md`.

---

## 12. Product Philosophy

The application should follow these principles:

### Simple

A user should be able to start taking meeting notes with one click.

### Invisible

The application should stay out of the way until it detects a relevant meeting.

### Local

The application should not require cloud infrastructure for basic operation.

### Reliable

Audio capture and meeting detection are more important than visual complexity.

### Useful

The final output should be a practical MOM that the user can immediately review and share.

### Focused

Do not turn the application into a general-purpose meeting platform.

---

## 13. Definition of Success

The V1 application is successful when a user can:

1. Start the application.
2. Join a Microsoft Teams meeting.
3. Receive a meeting-detected notification.
4. Select **Start Taking Notes**.
5. Capture the Teams meeting audio.
6. Obtain a usable transcript.
7. Add personal notes during the meeting.
8. End the session.
9. Generate a useful MOM from the transcript and notes.
10. Edit the MOM.
11. Save the meeting information locally.
12. Complete the entire workflow without adding a bot or participant to the Teams meeting.

---

## 14. Documentation Structure

The project documentation is organized as follows:

```text
MeetingNotesAssistant/
│
├── README.md
│
├── CLAUDE_INSTRUCTIONS.md
│
└── docs/
    ├── PRODUCT_REQUIREMENTS.md
    ├── USER_FLOW.md
    ├── TECHNICAL_ARCHITECTURE.md
    ├── AUDIO_CAPTURE.md
    ├── OPENAI_INTEGRATION.md
    ├── TRANSCRIPTION_SPECIFICATION.md
    ├── NOTES_AND_MOM_SPECIFICATION.md
    ├── UI_UX_SPECIFICATION.md
    ├── LOCAL_STORAGE_SPECIFICATION.md
    ├── SECURITY_AND_PRIVACY.md
    ├── SETUP_AND_DEVELOPMENT.md
    ├── IMPLEMENTATION_PLAN.md
    ├── TESTING_AND_ACCEPTANCE.md
    └── DECISIONS.md
```

Each document has a specific purpose. Claude should read the relevant documents before implementing the corresponding functionality.

---

## 15. Important Development Rule

Do not expand the product beyond this specification merely because additional features appear technically possible.

When there is a choice between:

- a simple implementation that satisfies the requirement, and
- a more complicated implementation with additional capabilities,

prefer the simple implementation.

If a requirement cannot be implemented reliably, document the limitation and propose the smallest practical solution rather than silently changing the product objective.

---

## 16. Current Product Definition

The simplest accurate description of this application is:

> **A lightweight Windows desktop assistant that detects Microsoft Teams meetings, lets the user start personal note-taking, captures the meeting audio without joining the meeting, creates a transcript, combines the transcript with the user's notes, and generates an editable Minutes of Meeting locally.**

That definition should remain the foundation of the project unless the product scope is deliberately changed.
