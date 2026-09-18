import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

// Phase 3 scope only (IMPLEMENTATION_PLAN.md):
//   - show a Start/Not Now popup when a meeting is detected
//   - track whether the user dismissed THIS meeting instance (no repeat
//     prompts per DECISIONS.md §9 / USER_FLOW.md §6)
//   - reset when the meeting ends, ready for the next one
//   - NO actual audio capture starts here - "Start Taking Notes" only
//     transitions local UI state for now. Audio capture is a later phase,
//     wired in only once the real recorder exists (DECISIONS.md §013-A).
//
// DECISIONS.md #009 requirement this satisfies: no recording begins
// merely because a meeting was detected - only on explicit user action.

type SessionState = "idle" | "awaiting_decision" | "dismissed" | "active";

export function MeetingNotification() {
  const [state, setState] = useState<SessionState>("idle");

  useEffect(() => {
    const unlistenDetected = listen("teams-meeting-detected", () => {
      // Only show the prompt if we're not already mid-decision or
      // already in an active/dismissed state for this same meeting -
      // prevents duplicate prompts (CLAUDE_INSTRUCTIONS.md §26).
      setState((prev) => (prev === "idle" ? "awaiting_decision" : prev));
    });

    const unlistenEnded = listen("teams-meeting-ended", () => {
      // Reset fully - ready for the next distinct meeting.
      setState("idle");
    });

    return () => {
      unlistenDetected.then((f) => f());
      unlistenEnded.then((f) => f());
    };
  }, []);

  const handleStart = () => {
    // Placeholder only - real audio capture wiring happens in a later
    // phase, once the dual-source recorder (DECISIONS.md #013-A) exists.
    console.log("[meeting] Start Taking Notes selected (audio capture not yet wired - future phase)");
    setState("active");
  };

  const handleNotNow = () => {
    console.log("[meeting] Not Now selected - no capture will start for this meeting");
    setState("dismissed");
  };

  if (state !== "awaiting_decision") {
    return null;
  }

  return (
    <div style={overlayStyle}>
      <div style={cardStyle}>
        <h2 style={{ margin: "0 0 8px 0", fontSize: "16px" }}>
          Teams meeting detected
        </h2>
        <p style={{ margin: "0 0 20px 0", color: "#555", fontSize: "14px" }}>
          Would you like to take notes?
        </p>
        <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
          <button onClick={handleNotNow} style={secondaryButtonStyle}>
            Not Now
          </button>
          <button onClick={handleStart} style={primaryButtonStyle}>
            Start Taking Notes
          </button>
        </div>
      </div>
    </div>
  );
}

const overlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "center",
  paddingTop: "40px",
  pointerEvents: "none",
};

const cardStyle: React.CSSProperties = {
  pointerEvents: "auto",
  background: "white",
  border: "1px solid #ddd",
  borderRadius: "8px",
  boxShadow: "0 4px 16px rgba(0,0,0,0.15)",
  padding: "20px",
  width: "320px",
};

const primaryButtonStyle: React.CSSProperties = {
  background: "#4f46e5",
  color: "white",
  border: "none",
  borderRadius: "4px",
  padding: "8px 14px",
  cursor: "pointer",
  fontSize: "13px",
};

const secondaryButtonStyle: React.CSSProperties = {
  background: "#f0f0f0",
  color: "#333",
  border: "none",
  borderRadius: "4px",
  padding: "8px 14px",
  cursor: "pointer",
  fontSize: "13px",
};
