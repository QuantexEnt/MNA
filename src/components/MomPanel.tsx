import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ActionItem {
  action: string;
  owner: string;
  due_date: string;
  status: string;
}

interface Mom {
  meeting_id: string;
  title: string;
  date: string;
  participants: string[];
  purpose: string;
  discussion_points: string[];
  decisions: string[];
  action_items: ActionItem[];
  open_questions: string[];
  next_steps: string[];
  model: string;
  created_at: string;
}

type GenStatus = "NOT_GENERATED" | "GENERATING" | "GENERATED" | "EDITED" | "FAILED";

interface MomPanelProps {
  meetingId: string;
  initialStatus: GenStatus;
}

function linesToArray(text: string): string[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
}

export function MomPanel({ meetingId, initialStatus }: MomPanelProps) {
  const [status, setStatus] = useState<GenStatus>(initialStatus);
  const [mom, setMom] = useState<Mom | null>(null);
  const [loading, setLoading] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copyFeedback, setCopyFeedback] = useState<string | null>(null);

  const [title, setTitle] = useState("");
  const [date, setDate] = useState("");
  const [participantsText, setParticipantsText] = useState("");
  const [purpose, setPurpose] = useState("");
  const [discussionText, setDiscussionText] = useState("");
  const [decisionsText, setDecisionsText] = useState("");
  const [actionItems, setActionItems] = useState<ActionItem[]>([]);
  const [openQuestionsText, setOpenQuestionsText] = useState("");
  const [nextStepsText, setNextStepsText] = useState("");

  const loadIntoForm = (m: Mom) => {
    setMom(m);
    setTitle(m.title);
    setDate(m.date);
    setParticipantsText(m.participants.join("\n"));
    setPurpose(m.purpose);
    setDiscussionText(m.discussion_points.join("\n"));
    setDecisionsText(m.decisions.join("\n"));
    setActionItems(m.action_items);
    setOpenQuestionsText(m.open_questions.join("\n"));
    setNextStepsText(m.next_steps.join("\n"));
  };

  useEffect(() => {
    if (initialStatus === "GENERATED" || initialStatus === "EDITED") {
      setLoading(true);
      invoke<Mom>("get_mom", { meetingId })
        .then(loadIntoForm)
        .catch((err) => {
          console.error("[mom] Failed to load:", err);
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
      const m = await invoke<Mom>("generate_mom", { meetingId });
      loadIntoForm(m);
      setStatus("GENERATED");
    } catch (err) {
      console.error("[mom] Generation failed:", err);
      setError(String(err));
      setStatus("FAILED");
    } finally {
      setGenerating(false);
    }
  };

  const regenerate = async () => {
    if (status === "EDITED") {
      const confirmed = window.confirm(
        "Your MOM has been edited. Regenerating will replace your changes with a fresh AI draft. Continue?"
      );
      if (!confirmed) return;
    }
    setError(null);
    setGenerating(true);
    try {
      const m = await invoke<Mom>("regenerate_mom", { meetingId });
      loadIntoForm(m);
      setStatus("GENERATED");
    } catch (err) {
      console.error("[mom] Regeneration failed:", err);
      setError(String(err));
    } finally {
      setGenerating(false);
    }
  };

  const save = async () => {
    if (!mom) return;
    setSaving(true);
    setError(null);
    const updated: Mom = {
      ...mom,
      title,
      date,
      participants: linesToArray(participantsText),
      purpose,
      discussion_points: linesToArray(discussionText),
      decisions: linesToArray(decisionsText),
      action_items: actionItems,
      open_questions: linesToArray(openQuestionsText),
      next_steps: linesToArray(nextStepsText),
    };
    try {
      await invoke("save_mom", { meetingId, mom: updated });
      setMom(updated);
      setStatus("EDITED");
    } catch (err) {
      console.error("[mom] Save failed:", err);
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const copy = async () => {
    try {
      const markdown = await invoke<string>("get_mom_markdown", { meetingId });
      await navigator.clipboard.writeText(markdown);
      setCopyFeedback("Copied");
    } catch (err) {
      console.error("[mom] Copy failed:", err);
      setCopyFeedback("Copy failed");
    } finally {
      setTimeout(() => setCopyFeedback(null), 2000);
    }
  };

  const updateActionItem = (index: number, field: keyof ActionItem, value: string) => {
    setActionItems((prev) =>
      prev.map((item, i) => (i === index ? { ...item, [field]: value } : item))
    );
  };

  const addActionItem = () => {
    setActionItems((prev) => [
      ...prev,
      { action: "", owner: "Not specified", due_date: "Not specified", status: "Open" },
    ]);
  };

  const removeActionItem = (index: number) => {
    setActionItems((prev) => prev.filter((_, i) => i !== index));
  };

  if (loading) {
    return <div style={{ padding: "20px", color: "#6F7F98", fontSize: "13px" }}>Loading...</div>;
  }

  if (!mom && status !== "GENERATING") {
    return (
      <div style={{ padding: "20px" }}>
        <p style={{ color: "#6F7F98", fontSize: "13px", marginBottom: "10px" }}>
          {status === "FAILED"
            ? "The last MOM generation attempt failed. You can try again."
            : "No MOM has been generated for this meeting yet."}
        </p>
        {error && (
          <p style={{ color: "#C73636", fontSize: "12px", marginBottom: "10px" }}>{error}</p>
        )}
        <button onClick={generate} disabled={generating} style={primaryButtonStyle}>
          {generating ? "Generating..." : status === "FAILED" ? "Retry" : "Generate MOM"}
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

      <Field label="Meeting Title">
        <input style={inputStyle} value={title} onChange={(e) => setTitle(e.target.value)} />
      </Field>
      <Field label="Date">
        <input style={inputStyle} value={date} onChange={(e) => setDate(e.target.value)} />
      </Field>
      <Field label="Participants" hint="one per line">
        <textarea
          style={textareaStyle}
          rows={3}
          value={participantsText}
          onChange={(e) => setParticipantsText(e.target.value)}
        />
      </Field>
      <Field label="Purpose / Context">
        <textarea
          style={textareaStyle}
          rows={3}
          value={purpose}
          onChange={(e) => setPurpose(e.target.value)}
        />
      </Field>
      <Field label="Key Discussion Points" hint="one per line">
        <textarea
          style={textareaStyle}
          rows={4}
          value={discussionText}
          onChange={(e) => setDiscussionText(e.target.value)}
        />
      </Field>
      <Field label="Decisions Made" hint="one per line">
        <textarea
          style={textareaStyle}
          rows={3}
          value={decisionsText}
          onChange={(e) => setDecisionsText(e.target.value)}
        />
      </Field>

      <Field label="Action Items">
        <div style={{ border: "1px solid #DCE3F0", borderRadius: "6px", overflow: "hidden" }}>
          {actionItems.length === 0 && (
            <div style={{ padding: "10px", fontSize: "12px", color: "#6F7F98" }}>
              No action items.
            </div>
          )}
          {actionItems.map((item, i) => (
            <div
              key={i}
              style={{
                display: "flex",
                gap: "6px",
                padding: "8px",
                borderBottom: "1px solid #DCE3F0",
                alignItems: "center",
              }}
            >
              <input
                style={{ ...inputStyle, flex: 2 }}
                placeholder="Action"
                value={item.action}
                onChange={(e) => updateActionItem(i, "action", e.target.value)}
              />
              <input
                style={{ ...inputStyle, flex: 1 }}
                placeholder="Owner"
                value={item.owner}
                onChange={(e) => updateActionItem(i, "owner", e.target.value)}
              />
              <input
                style={{ ...inputStyle, flex: 1 }}
                placeholder="Due Date"
                value={item.due_date}
                onChange={(e) => updateActionItem(i, "due_date", e.target.value)}
              />
              <select
                style={{ ...inputStyle, flex: 1 }}
                value={item.status}
                onChange={(e) => updateActionItem(i, "status", e.target.value)}
              >
                <option value="Open">Open</option>
                <option value="Done">Done</option>
              </select>
              <button
                onClick={() => removeActionItem(i)}
                style={removeButtonStyle}
                title="Remove"
              >
                ×
              </button>
            </div>
          ))}
          <button onClick={addActionItem} style={{ ...secondaryButtonStyle, margin: "8px" }}>
            + Add Action Item
          </button>
        </div>
      </Field>

      <Field label="Open Questions" hint="one per line">
        <textarea
          style={textareaStyle}
          rows={3}
          value={openQuestionsText}
          onChange={(e) => setOpenQuestionsText(e.target.value)}
        />
      </Field>
      <Field label="Next Steps" hint="one per line">
        <textarea
          style={textareaStyle}
          rows={3}
          value={nextStepsText}
          onChange={(e) => setNextStepsText(e.target.value)}
        />
      </Field>
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

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ marginBottom: "14px" }}>
      <label
        style={{
          display: "block",
          fontSize: "12px",
          fontWeight: 600,
          color: "#12244A",
          marginBottom: "4px",
        }}
      >
        {label}
        {hint && <span style={{ fontWeight: 400, color: "#6F7F98" }}> ({hint})</span>}
      </label>
      {children}
    </div>
  );
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

const removeButtonStyle: React.CSSProperties = {
  background: "none",
  border: "none",
  color: "#C73636",
  fontSize: "16px",
  cursor: "pointer",
  padding: "0 6px",
};