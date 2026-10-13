import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MeetingMetadata, StatusBadge, TranscriptViewer } from "./TranscriptViewer";

// History page: shows all recorded meetings with their transcription
// status. Clicking a row opens its transcript (view or transcribe).
// A separate "Open in Review" button opens the full Meeting Review
// screen (MOM / AI Notes / My Notes / Transcripts tabs) for that
// meeting - previously Review was only reachable via the end-of-meeting
// redirect, which made it hard to test MOM/AI Notes without recording a
// fresh meeting every time.

interface HistoryPageProps {
  onOpenReview: (meeting: MeetingMetadata) => void;
}

export function HistoryPage({ onOpenReview }: HistoryPageProps) {
  const [meetings, setMeetings] = useState<MeetingMetadata[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeMeeting, setActiveMeeting] = useState<MeetingMetadata | null>(null);

  const loadMeetings = () => {
    setLoading(true);
    invoke<MeetingMetadata[]>("list_meetings")
      .then((result) => {
        setMeetings(result);
        setLoading(false);
      })
      .catch((err) => {
        console.error("[history] Failed to load meetings:", err);
        setLoading(false);
      });
  };

  useEffect(() => {
    loadMeetings();
  }, []);

  if (loading) {
    return <div style={{ padding: "20px", color: "#6F7F98", fontSize: "13px" }}>Loading...</div>;
  }

  if (activeMeeting) {
    return (
      <TranscriptViewer
        meeting={activeMeeting}
        onBack={() => {
          setActiveMeeting(null);
          loadMeetings();
        }}
        onStatusChange={loadMeetings}
      />
    );
  }

  if (meetings.length === 0) {
    return (
      <div style={{ padding: "20px", color: "#6F7F98", fontSize: "13px" }}>
        <p>No meetings yet.</p>
        <p style={{ fontSize: "12px" }}>
          When you take notes during a Teams meeting, your meetings will
          appear here.
        </p>
      </div>
    );
  }

  return (
    <div style={{ padding: "20px" }}>
      {meetings.map((m) => (
        <div
          key={m.id}
          style={{
            padding: "12px 0",
            borderBottom: "1px solid #DCE3F0",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            fontSize: "13px",
          }}
        >
          <div style={{ cursor: "pointer" }} onClick={() => setActiveMeeting(m)}>
            <div style={{ fontWeight: 600, color: "#12244A" }}>{m.title}</div>
            <div style={{ color: "#6F7F98", fontSize: "12px" }}>
              {new Date(m.started_at).toLocaleString()}
            </div>
            <div style={{ marginTop: "4px", display: "flex", gap: "6px" }}>
              <StatusBadge status={m.transcript_status} />
              <span
                style={{
                  fontSize: "11px",
                  padding: "2px 8px",
                  borderRadius: "10px",
                  background: m.status === "COMPLETED" ? "#F1F3FF" : "#FDF2F2",
                  color: m.status === "COMPLETED" ? "#147DFF" : "#C73636",
                }}
              >
                {m.status}
              </span>
            </div>
          </div>
          <button onClick={() => onOpenReview(m)} style={reviewButtonStyle}>
            Open in Review
          </button>
        </div>
      ))}
    </div>
  );
}

const reviewButtonStyle: React.CSSProperties = {
  background: "#F1F3FF",
  color: "#12244A",
  border: "none",
  borderRadius: "4px",
  padding: "6px 14px",
  cursor: "pointer",
  fontSize: "12px",
  flexShrink: 0,
};