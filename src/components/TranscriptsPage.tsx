import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MeetingMetadata, StatusBadge, TranscriptViewer } from "./TranscriptViewer";

export function TranscriptsPage() {
  const [meetings, setMeetings] = useState<MeetingMetadata[]>([]);
  const [loadingList, setLoadingList] = useState(true);
  const [activeMeeting, setActiveMeeting] = useState<MeetingMetadata | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadMeetings = () => {
    setLoadingList(true);
    invoke<MeetingMetadata[]>("list_meetings")
      .then((result) => {
        setMeetings(result);
        setLoadingList(false);
      })
      .catch((err) => {
        console.error("[transcripts] Failed to load meetings:", err);
        setError(String(err));
        setLoadingList(false);
      });
  };

  useEffect(() => {
    loadMeetings();
  }, []);

  if (loadingList) {
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

  const latest = meetings.length > 0 ? meetings[0] : null;
  const past = meetings.length > 1 ? meetings.slice(1) : [];

  return (
    <div style={{ padding: "20px", overflow: "auto", height: "100%" }}>
      {error && (
        <div style={{ color: "#C73636", fontSize: "12px", marginBottom: "12px" }}>{error}</div>
      )}

      {!latest && (
        <p style={{ color: "#6F7F98", fontSize: "13px" }}>
          No meetings yet. Once you record a meeting, it will appear here.
        </p>
      )}

      {latest && (
        <div style={{ marginBottom: "28px" }}>
          <h3 style={{ fontSize: "14px", color: "#12244A", marginBottom: "10px" }}>
            Latest Meeting
          </h3>
          <div
            style={{
              padding: "14px",
              border: "1px solid #DCE3F0",
              borderRadius: "8px",
              background: "#F8FAFF",
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              cursor: "pointer",
            }}
            onClick={() => setActiveMeeting(latest)}
          >
            <div>
              <div style={{ fontWeight: 600, fontSize: "13px", color: "#12244A" }}>
                {latest.title}
              </div>
              <div style={{ fontSize: "12px", color: "#6F7F98", marginTop: "2px" }}>
                {new Date(latest.started_at).toLocaleString()}
              </div>
              <div style={{ marginTop: "6px" }}>
                <StatusBadge status={latest.transcript_status} />
              </div>
            </div>
            <button
              style={primaryButtonStyle}
              onClick={(e) => {
                e.stopPropagation();
                setActiveMeeting(latest);
              }}
            >
              {latest.transcript_status === "TRANSCRIBED"
                ? "View Transcript"
                : "Transcribe Latest Meeting"}
            </button>
          </div>
        </div>
      )}

      {past.length > 0 && (
        <div>
          <h3 style={{ fontSize: "14px", color: "#12244A", marginBottom: "10px" }}>
            Past Meetings
          </h3>
          {past.map((m) => (
            <div
              key={m.id}
              style={{
                padding: "10px 0",
                borderBottom: "1px solid #DCE3F0",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                fontSize: "13px",
                cursor: "pointer",
              }}
              onClick={() => setActiveMeeting(m)}
            >
              <div>
                <div style={{ fontWeight: 600, color: "#12244A" }}>{m.title}</div>
                <div style={{ color: "#6F7F98", fontSize: "12px" }}>
                  {new Date(m.started_at).toLocaleString()}
                </div>
                <div style={{ marginTop: "4px" }}>
                  <StatusBadge status={m.transcript_status} />
                </div>
              </div>
              <button
                style={secondaryButtonStyle}
                onClick={(e) => {
                  e.stopPropagation();
                  setActiveMeeting(m);
                }}
              >
                {m.transcript_status === "TRANSCRIBED" ? "View Transcript" : "Transcribe"}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

const primaryButtonStyle: React.CSSProperties = {
  background: "#147DFF",
  color: "white",
  border: "none",
  borderRadius: "4px",
  padding: "6px 14px",
  cursor: "pointer",
  fontSize: "12px",
};

const secondaryButtonStyle: React.CSSProperties = {
  background: "#F1F3FF",
  color: "#12244A",
  border: "none",
  borderRadius: "4px",
  padding: "6px 14px",
  cursor: "pointer",
  fontSize: "12px",
};