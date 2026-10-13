import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MeetingMetadata, TranscriptViewer } from "./TranscriptViewer";
import { MomPanel } from "./MomPanel";
import { AiNotesPanel } from "./AiNotesPanel";

type ReviewTab = "mom" | "ai_notes" | "my_notes" | "transcripts";

interface MeetingReviewPageProps {
  meeting: MeetingMetadata;
  onClose: () => void;
}

const TABS: { id: ReviewTab; label: string }[] = [
  { id: "mom", label: "MOM" },
  { id: "ai_notes", label: "AI Notes" },
  { id: "my_notes", label: "My Notes" },
  { id: "transcripts", label: "Transcripts" },
];

export function MeetingReviewPage({ meeting, onClose }: MeetingReviewPageProps) {
  const [tab, setTab] = useState<ReviewTab>("mom");
  const [notes, setNotes] = useState<string | null>(null);
  const [notesLoading, setNotesLoading] = useState(false);
  const [notesError, setNotesError] = useState<string | null>(null);

  useEffect(() => {
    if (tab === "my_notes" && notes === null && !notesLoading) {
      setNotesLoading(true);
      invoke<string>("get_notes", { meetingId: meeting.id })
        .then((content) => setNotes(content))
        .catch((err) => {
          console.error("[meeting-review] Failed to load notes:", err);
          setNotesError(String(err));
        })
        .finally(() => setNotesLoading(false));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab]);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "14px 20px",
          borderBottom: "1px solid #DCE3F0",
          flexShrink: 0,
        }}
      >
        <div>
          <div style={{ fontWeight: 600, fontSize: "15px", color: "#12244A" }}>
            Meeting Review
          </div>
          <div style={{ fontSize: "12px", color: "#6F7F98", marginTop: "2px" }}>
            {meeting.title} · {new Date(meeting.started_at).toLocaleString()}
          </div>
        </div>
        <button onClick={onClose} style={closeButtonStyle}>
          Close
        </button>
      </div>

      <div style={{ display: "flex", borderBottom: "1px solid #DCE3F0", flexShrink: 0 }}>
        {TABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            style={{
              padding: "10px 18px",
              fontSize: "13px",
              fontWeight: tab === t.id ? 600 : 400,
              color: tab === t.id ? "#147DFF" : "#6F7F98",
              background: "none",
              border: "none",
              borderBottom: tab === t.id ? "2px solid #147DFF" : "2px solid transparent",
              cursor: "pointer",
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div style={{ flex: 1, overflow: "auto" }}>
        {tab === "mom" && <MomPanel meetingId={meeting.id} initialStatus={meeting.mom_status} />}
        {tab === "ai_notes" && (
          <AiNotesPanel meetingId={meeting.id} initialStatus={meeting.ai_notes_status} />
        )}
        {tab === "my_notes" && (
          <div style={{ padding: "20px" }}>
            {notesLoading && <p style={{ color: "#6F7F98", fontSize: "13px" }}>Loading notes...</p>}
            {notesError && <p style={{ color: "#C73636", fontSize: "12px" }}>{notesError}</p>}
            {notes !== null && notes.trim() === "" && (
              <p style={{ color: "#6F7F98", fontSize: "13px" }}>
                No notes were written during this meeting.
              </p>
            )}
            {notes !== null && notes.trim() !== "" && (
              <pre
                style={{
                  whiteSpace: "pre-wrap",
                  fontFamily: "inherit",
                  fontSize: "13px",
                  color: "#12244A",
                  lineHeight: 1.6,
                  margin: 0,
                }}
              >
                {notes}
              </pre>
            )}
          </div>
        )}
        {tab === "transcripts" && <TranscriptViewer meeting={meeting} />}
      </div>
    </div>
  );
}

const closeButtonStyle: React.CSSProperties = {
  background: "#F1F3FF",
  color: "#12244A",
  border: "none",
  borderRadius: "4px",
  padding: "6px 14px",
  fontSize: "12px",
  cursor: "pointer",
};