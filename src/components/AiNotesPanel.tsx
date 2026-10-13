import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AiNotes {
  meeting_id: string;
  bullets: string[];
  model: string;
  created_at: string;
}

type GenStatus = "NOT_GENERATED" | "GENERATING" | "GENERATED" | "EDITED" | "FAILED";

interface AiNotesPanelProps {
  meetingId: string;
  initialStatus: GenStatus;
}

function linesToArray(text: string): string[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
}

export function AiNotesPanel({ meetingId, initialStatus }: AiNotesPanelProps) {
  const [status, setStatus] = useState<GenStatus>(initialStatus);
  const [notes, setNotes] = useState<AiNotes | null>(null);
  const [bulletsText, setBulletsText] = useState("");
  const [loading, setLoading] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copyFeedback, setCopyFeedback] = useState<string | null>(null);

  useEffect(() => {
    if (initialStatus === "GENERATED" || initialStatus === "EDITED") {
      setLoading(true);
      invoke<AiNotes>("get_ai_notes", { meetingId })
        .then((n) => {
          setNotes(n);
          setBulletsText(n.bullets.join("\n"));
        })
        .catch((err) => {
          console.error("[ai-notes] Failed to load:", err);
          setError(String(err));
        })
        .finally(() => setLoading(false));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [meetingId]);

  const generate = async () => {
    setError(null);
    setGenerating(true);
    try {
      const n = await invoke<AiNotes>("generate_ai_notes", { meetingId });
      setNotes(n);
      setBulletsText(n.bullets.join("\n"));
      setStatus("GENERATED");
    } catch (err) {
      console.error("[ai-notes] Generation failed:", err);
      setError(String(err));
      setStatus("FAILED");
    } finally {
      setGenerating(false);
    }
  };

  const regenerate = async () => {
    if (status === "EDITED") {
      const confirmed = window.confirm(
        "Your AI Notes have been edited. Regenerating will replace your changes with a fresh AI draft. Continue?"
      );
      if (!confirmed) return;
    }
    setError(null);
    setGenerating(true);
    try {
      const n = await invoke<AiNotes>("regenerate_ai_notes", { meetingId });
      setNotes(n);
      setBulletsText(n.bullets.join("\n"));
      setStatus("GENERATED");
    } catch (err) {
      console.error("[ai-notes] Regeneration failed:", err);
      setError(String(err));
    } finally {
      setGenerating(false);
    }
  };

  const save = async () => {
    if (!notes) return;
    setSaving(true);
    setError(null);
    const updated: AiNotes = { ...notes, bullets: linesToArray(bulletsText) };
    try {
      await invoke("save_ai_notes", { meetingId, notes: updated });
      setNotes(updated);
      setStatus("EDITED");
    } catch (err) {
      console.error("[ai-notes] Save failed:", err);
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const copy = async () => {
    try {
      const markdown = await invoke<string>("get_ai_notes_markdown", { meetingId });
      await navigator.clipboard.writeText(markdown);
      setCopyFeedback("Copied");
    } catch (err) {
      console.error("[ai-notes] Copy failed:", err);
      setCopyFeedback("Copy failed");
    } finally {
      setTimeout(() => setCopyFeedback(null), 2000);
    }
  };

  if (loading) {
    return <div style={{ padding: "20px", color: "#6F7F98", fontSize: "13px" }}>Loading...</div>;
  }

  if (!notes && status !== "GENERATING") {
    return (
      <div style={{ padding: "20px" }}>
        <p style={{ color: "#6F7F98", fontSize: "13px", marginBottom: "10px" }}>
          {status === "FAILED"
            ? "The last AI Notes generation attempt failed. You can try again."
            : "No AI Notes have been generated for this meeting yet."}
        </p>
        {error && (
          <p style={{ color: "#C73636", fontSize: "12px", marginBottom: "10px" }}>{error}</p>
        )}
        <button onClick={generate} disabled={generating} style={primaryButtonStyle}>
          {generating ? "Generating..." : status === "FAILED" ? "Retry" : "Generate AI Notes"}
        </button>
      </div>
    );
  }

  return (
    <div style={{ padding: "20px", maxWidth: "700px" }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "16px",
        }}
      >
        <StatusLabel status={status} />
        <div style={{ display: "flex", gap: "8px" }}>
          <button onClick={copy} style={secondaryButtonStyle}>
            Copy
          </button>
          <button onClick={regenerate} disabled={generating} style={secondaryButtonStyle}>
            {generating ? "Regenerating..." : "Regenerate"}
          </button>
          <button onClick={save} disabled={saving} style={primaryButtonStyle}>
            {saving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>

      {copyFeedback && (
        <div style={{ fontSize: "12px", color: "#1F8A5B", marginBottom: "10px" }}>
          {copyFeedback}
        </div>
      )}
      {error && (
        <div style={{ fontSize: "12px", color: "#C73636", marginBottom: "10px" }}>{error}</div>
      )}

      <label
        style={{
          display: "block",
          fontSize: "12px",
          fontWeight: 600,
          color: "#12244A",
          marginBottom: "4px",
        }}
      >
        Key Points (one per line)
      </label>
      <textarea
        style={textareaStyle}
        rows={12}
        value={bulletsText}
        onChange={(e) => setBulletsText(e.target.value)}
      />
    </div>
  );
}

function StatusLabel({ status }: { status: GenStatus }) {
  const config: Record<GenStatus, { label: string; color: string }> = {
    NOT_GENERATED: { label: "Not generated", color: "#6F7F98" },
    GENERATING: { label: "Generating...", color: "#B86A00" },
    GENERATED: { label: "AI draft (unedited)", color: "#1F8A5B" },
    EDITED: { label: "Edited by you", color: "#147DFF" },
    FAILED: { label: "Generation failed", color: "#C73636" },
  };
  const c = config[status];
  return <span style={{ fontSize: "12px", fontWeight: 600, color: c.color }}>{c.label}</span>;
}

const inputStyle: React.CSSProperties = {
  width: "100%",
  padding: "6px 8px",
  border: "1px solid #DCE3F0",
  borderRadius: "4px",
  fontSize: "13px",
  boxSizing: "border-box",
};

const textareaStyle: React.CSSProperties = {
  ...inputStyle,
  fontFamily: "inherit",
  resize: "vertical",
};

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