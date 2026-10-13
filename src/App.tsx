import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MeetingNotification } from "./components/MeetingNotification";
import { SettingsPage } from "./components/SettingsPage";
import { HistoryPage } from "./components/HistoryPage";
import { TranscriptsPage } from "./components/TranscriptsPage";
import { MeetingReviewPage } from "./components/MeetingReviewPage";
import { MeetingMetadata } from "./components/TranscriptViewer";
import logo from "./assets/Horizontal-_MNA.png";

type Page = "meeting" | "history" | "transcripts" | "settings" | "review";

function NavItem({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      style={{
        padding: "8px 16px",
        fontSize: "13px",
        cursor: "pointer",
        color: active ? "#147DFF" : "#6F7F98",
        fontWeight: active ? 600 : 400,
        borderLeft: active ? "3px solid #147DFF" : "3px solid transparent",
      }}
    >
      {label}
    </div>
  );
}

function GearIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

function LogoutIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
      <polyline points="16 17 21 12 16 7" />
      <line x1="21" y1="12" x2="9" y2="12" />
    </svg>
  );
}

function App() {
  const [page, setPage] = useState<Page>("meeting");
  const [reviewMeeting, setReviewMeeting] = useState<MeetingMetadata | null>(null);

  const openReview = (meeting: MeetingMetadata) => {
    setReviewMeeting(meeting);
    setPage("review");
  };

  return (
    <div
      style={{
        fontFamily: "Inter, -apple-system, sans-serif",
        display: "flex",
        flexDirection: "column",
        height: "100vh",
      }}
    >
      <header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "10px 16px",
          borderBottom: "1px solid #DCE3F0",
          flexShrink: 0,
        }}
      >
        <img src={logo} alt="Meeting Note Assistant" style={{ height: "24px" }} />
        <button
          onClick={() => setPage("settings")}
          title="Settings"
          aria-label="Settings"
          style={{
            background: "transparent",
            color: page === "settings" ? "#147DFF" : "#6F7F98",
            border: "none",
            borderRadius: "4px",
            padding: "6px",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <GearIcon />
        </button>
      </header>

      {/* Always mounted regardless of active tab - fixes the bug where
          meeting detection stopped working while on History/Settings,
          and keeps recording controls/notes accessible from any tab. */}
      <MeetingNotification onMeetingEnded={openReview} />

      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <nav
          style={{
            width: "140px",
            background: "#F8FAFF",
            borderRight: "1px solid #DCE3F0",
            padding: "12px 0",
            flexShrink: 0,
            display: "flex",
            flexDirection: "column",
            justifyContent: "space-between",
          }}
        >
          <div>
            <NavItem label="Meeting" active={page === "meeting"} onClick={() => setPage("meeting")} />
            <NavItem label="History" active={page === "history"} onClick={() => setPage("history")} />
            <NavItem label="Transcripts" active={page === "transcripts"} onClick={() => setPage("transcripts")} />
          </div>

          <button
            onClick={() => invoke("exit_app")}
            title="Quit the app completely (finishes and saves any active recording first)"
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              background: "transparent",
              color: "#C73636",
              border: "none",
              borderTop: "1px solid #DCE3F0",
              padding: "12px 16px",
              fontSize: "13px",
              fontWeight: 600,
              cursor: "pointer",
              textAlign: "left",
            }}
          >
            <LogoutIcon />
            Logout
          </button>
        </nav>

        <main style={{ flex: 1, overflow: "auto" }}>
          {page === "meeting" && (
            <div style={{ padding: "20px" }}>
              <p style={{ color: "#6F7F98" }}>
                Running in the background. Waiting for a Teams meeting...
              </p>
            </div>
          )}
          {page === "history" && <HistoryPage onOpenReview={openReview} />}
          {page === "transcripts" && <TranscriptsPage />}
          {page === "settings" && <SettingsPage />}
          {page === "review" &&
            (reviewMeeting ? (
              <MeetingReviewPage
                meeting={reviewMeeting}
                onClose={() => {
                  setReviewMeeting(null);
                  setPage("meeting");
                }}
              />
            ) : (
              <div style={{ padding: "20px", color: "#6F7F98" }}>No meeting selected.</div>
            ))}
        </main>
      </div>
    </div>
  );
}

export default App;