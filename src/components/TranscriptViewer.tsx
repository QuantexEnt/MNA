import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface MeetingMetadata {
  id: string;
  title: string;
  started_at: string;
  ended_at: string | null;
  status: string;
  audio: { path: string; duration_seconds: number | null } | null;
  transcript_status: "NOT_TRANSCRIBED" | "PROCESSING" | "TRANSCRIBED" | "FAILED";
  mom_status: "NOT_GENERATED" | "GENERATING" | "GENERATED" | "EDITED" | "FAILED";
  ai_notes_status: "NOT_GENERATED" | "GENERATING" | "GENERATED" | "EDITED" | "FAILED";
}

interface TranscriptSegment {
  speaker: string;
  text: string;
  start: number | null;
  end: number | null;
}

interface Transcript {
  meeting_id: string;
  text: string;
  language: string;
  duration_seconds: number | null;
  model: string;
  segments: TranscriptSegment[];
  created_at: string;
}

export function StatusBadge({ status }: { status: MeetingMetadata["transcript_status"] }) {
  const config: Record<string, { label: string; bg: string; color: string }> = {
    NOT_TRANSCRIBED: { label: "Not Transcribed", bg: "#F1F3FF", color: "#6F7F98" },
    PROCESSING: { label: "Processing...", bg: "#FFF6E5", color: "#B86A00" },
    TRANSCRIBED: { label: "Transcribed", bg: "#E8F7EF", color: "#1F8A5B" },
    FAILED: { label: "Failed", bg: "#FDF2F2", color: "#C73636" },
  };
  const c = config[status] ?? config.NOT_TRANSCRIBED;
  return (
    <span
      style={{
        fontSize: "11px",
        fontWeight: 600,
        padding: "2px 8px",
        borderRadius: "10px",
        background: c.bg,
        color: c.color,
      }}
    >
      {c.label}
    </span>
  );
}

interface TranscriptViewerProps {
  meeting: MeetingMetadata;
  onBack?: () => void;
  onStatusChange?: () => void;
}

export function TranscriptViewer({ meeting, onBack, onStatusChange }: TranscriptViewerProps) {
  const [currentMeeting, setCurrentMeeting] = useState(meeting);
  const [transcript, setTranscript] = useState<Transcript | null>(null);
  const [loadingTranscript, setLoadingTranscript] = useState(false);
  const [transcribing, setTranscribing] = useState(false);
  const [speakerNames, setSpeakerNames] = useState<Record<string, string>>({});
  const [savingNames, setSavingNames] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setCurrentMeeting(meeting);
    setTranscript(null);
    setError(null);
    if (meeting.transcript_status === "TRANSCRIBED") {
      loadExisting(meeting.id);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [meeting.id]);

  const loadExisting = async (meetingId: string) => {
    setLoadingTranscript(true);
    setError(null);
    try {
      const t = await invoke<Transcript>("get_transcript", { meetingId });
      const names = await invoke<Record<string, string>>("get_speaker_names", { meetingId });
      setTranscript(t);
      setSpeakerNames(names);
    } catch (err) {
      console.error("[transcript-viewer] Failed to load transcript:", err);
      setError(String(err));
    } finally {
      setLoadingTranscript(false);
    }
  };

  const transcribe = async () => {
    setError(null);
    setTranscribing(true);
    try {
      const t = await invoke<Transcript>("transcribe_meeting", { meetingId: currentMeeting.id });
      const names = await invoke<Record<string, string>>("get_speaker_names", {
        meetingId: currentMeeting.id,
      });
      setTranscript(t);
      setSpeakerNames(names);
      setCurrentMeeting((m) => ({ ...m, transcript_status: "TRANSCRIBED" }));
      onStatusChange?.();
    } catch (err) {
      console.error("[transcript-viewer] Transcription failed:", err);
      setError(String(err));
      setCurrentMeeting((m) => ({ ...m, transcript_status: "FAILED" }));
      onStatusChange?.();
    } finally {
      setTranscribing(false);
    }
  };

  const saveSpeakerNames = async () => {
    setSavingNames(true);
    try {
      await invoke("save_speaker_names", { meetingId: currentMeeting.id, names: speakerNames });
    } catch (err) {
      console.error("[transcript-viewer] Failed to save speaker names:", err);
      setError(String(err));
    } finally {
      setSavingNames(false);
    }
  };

  const displaySpeaker = (raw: string) => {
    if (raw === "You") return "You";
    return speakerNames[raw] && speakerNames[raw].trim() !== "" ? speakerNames[raw] : raw;
  };

  const uniqueRenameableSpeakers = transcript
    ? Array.from(new Set(transcript.segments.map((s) => s.speaker))).filter((s) => s !== "You")
    : [];

  return (
    <div style={{ display: "flex", height: "100%" }}>
      <div style={{ flex: 1, overflow: "auto", padding: "20px" }}>
        {onBack && (
          <button onClick={onBack} style={backLinkStyle}>
            ← Back
          </button>
        )}

        <div style={{ marginTop: onBack ? "12px" : 0, marginBottom: "16px" }}>
          <div style={{ fontWeight: 600, fontSize: "15px", color: "#12244A" }}>
            {currentMeeting.title}
          </div>
          <div style={{ fontSize: "12px", color: "#6F7F98", marginTop: "2px" }}>
            {new Date(currentMeeting.started_at).toLocaleString()}
          </div>
          <div style={{ marginTop: "6px" }}>
            <StatusBadge status={currentMeeting.transcript_status} />
          </div>
        </div>

        {error && (
          <div style={{ color: "#C73636", fontSize: "12px", marginBottom: "12px" }}>{error}</div>
        )}

        {!transcript &&
          !loadingTranscript &&
          currentMeeting.transcript_status !== "PROCESSING" && (
            <div style={{ marginTop: "8px" }}>
              <p style={{ color: "#6F7F98", fontSize: "13px", marginBottom: "10px" }}>
                {currentMeeting.transcript_status === "FAILED"
                  ? "The last transcription attempt failed. You can try again."
                  : "This meeting has not been transcribed yet."}
              </p>
              <button onClick={transcribe} disabled={transcribing} style={primaryButtonStyle}>
                {transcribing
                  ? "Transcribing..."
                  : currentMeeting.transcript_status === "FAILED"
                  ? "Retry"
                  : "Transcribe"}
              </button>
            </div>
          )}

        {(loadingTranscript || currentMeeting.transcript_status === "PROCESSING" || transcribing) &&
          !transcript && <p style={{ color: "#6F7F98", fontSize: "13px" }}>Loading...</p>}

        {transcript && (
          <div
            style={{
              border: "1px solid #DCE3F0",
              borderRadius: "8px",
              padding: "16px",
              background: "white",
            }}
          >
            {transcript.segments.length === 0 && (
              <p style={{ color: "#6F7F98", fontSize: "13px" }}>
                No transcript content available.
              </p>
            )}
            {transcript.segments.map((seg, i) => (
              <div key={i} style={{ marginBottom: "14px" }}>
                <div style={{ fontWeight: 600, fontSize: "12px", color: "#147DFF" }}>
                  {displaySpeaker(seg.speaker)}
                </div>
                <div style={{ fontSize: "13px", color: "#12244A", marginTop: "2px" }}>
                  {seg.text}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {transcript && uniqueRenameableSpeakers.length > 0 && (
        <div
          style={{
            width: "240px",
            borderLeft: "1px solid #DCE3F0",
            padding: "20px 16px",
            background: "#F8FAFF",
            flexShrink: 0,
            overflow: "auto",
          }}
        >
          <h4 style={{ fontSize: "13px", color: "#12244A", marginBottom: "8px" }}>
            Name Speakers
          </h4>
          <p style={{ fontSize: "11px", color: "#6F7F98", marginBottom: "14px", lineHeight: 1.5 }}>
            Optional. Assigning real names helps make the AI-generated
            MOM and AI Notes more relevant and easier to read.
          </p>
          {uniqueRenameableSpeakers.map((speaker) => (
            <div key={speaker} style={{ marginBottom: "10px" }}>
              <label style={{ fontSize: "11px", color: "#6F7F98" }}>{speaker}</label>
              <input
                type="text"
                placeholder="e.g. Priya"
                value={speakerNames[speaker] ?? ""}
                onChange={(e) =>
                  setSpeakerNames((prev) => ({ ...prev, [speaker]: e.target.value }))
                }
                style={{
                  width: "100%",
                  padding: "6px 8px",
                  fontSize: "12px",
                  border: "1px solid #DCE3F0",
                  borderRadius: "4px",
                  marginTop: "4px",
                }}
              />
            </div>
          ))}
          <button
            onClick={saveSpeakerNames}
            disabled={savingNames}
            style={{ ...primaryButtonStyle, width: "100%", marginTop: "8px" }}
          >
            {savingNames ? "Saving..." : "Save Names"}
          </button>
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

const backLinkStyle: React.CSSProperties = {
  background: "none",
  border: "none",
  color: "#147DFF",
  fontSize: "13px",
  cursor: "pointer",
  padding: 0,
};