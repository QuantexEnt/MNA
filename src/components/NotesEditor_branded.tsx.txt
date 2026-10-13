import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

// Phase 8: Manual Notes. Auto-saves to notes/notes.md inside the current
// meeting session's folder. Notes are separate from any transcript and
// must never be silently rewritten by anything else
// (NOTES_AND_MOM_SPECIFICATION.md §2/§3). Plain text only, per
// UI_UX_SPECIFICATION.md §14 - no rich formatting for V1.

const AUTOSAVE_DELAY_MS = 1000;

type SaveStatus = "idle" | "saving" | "saved" | "error";

export function NotesEditor() {
  const [content, setContent] = useState("");
  const [status, setStatus] = useState<SaveStatus>("idle");
  const debounceRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (debounceRef.current !== null) {
        window.clearTimeout(debounceRef.current);
      }
    };
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setContent(value);
    setStatus("saving");

    if (debounceRef.current !== null) {
      window.clearTimeout(debounceRef.current);
    }
    debounceRef.current = window.setTimeout(async () => {
      try {
        await invoke("save_notes", { content: value });
        setStatus("saved");
      } catch (err) {
        console.error("[notes] Failed to save:", err);
        setStatus("error");
      }
    }, AUTOSAVE_DELAY_MS);
  };

  const statusText: Record<SaveStatus, string> = {
    idle: "",
    saving: "Saving...",
    saved: "Saved locally",
    error: "Could not save - check console",
  };

  return (
    <div style={{ padding: "12px 16px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
        <label style={{ fontSize: "12px", fontWeight: 600, color: "#12244A" }}>My Notes</label>
        <span style={{ fontSize: "11px", color: status === "error" ? "#C73636" : "#6F7F98" }}>
          {statusText[status]}
        </span>
      </div>
      <textarea
        value={content}
        onChange={handleChange}
        placeholder="Type your notes here..."
        style={{
          width: "100%",
          minHeight: "120px",
          padding: "8px",
          fontSize: "13px",
          fontFamily: "inherit",
          border: "1px solid #ddd",
          borderRadius: "4px",
          resize: "vertical",
          boxSizing: "border-box",
        }}
      />
    </div>
  );
}
