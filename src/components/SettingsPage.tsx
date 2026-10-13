import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

// Real Settings page. API key/models/toggles persist to settings.json
// via Rust commands. License section is UI-only placeholder per the
// planned future licensing system - disabled until a real backend
// exists. "Start with Windows" is visually present but not yet
// functionally wired (Windows startup registration) - the preference
// still saves correctly, the OS registration itself just isn't built
// yet. Test Connection is now live (Phase 9 real transcription mode).

interface AppSettings {
  openai_api_key: string;
  transcription_model: string;
  mom_model: string;
  start_with_windows: boolean;
  meeting_notifications: boolean;
  license_key: string;
}

const DEFAULT_SETTINGS: AppSettings = {
  openai_api_key: "",
  // Matches settings.rs's AppSettings::default() - diarization
  // requires this specific model; a plain transcribe model will fail
  // with a 400 error on response_format=diarized_json.
  transcription_model: "gpt-4o-transcribe-diarize",
  mom_model: "gpt-4o-mini",
  start_with_windows: false,
  meeting_notifications: true,
  license_key: "",
};

const sectionStyle: React.CSSProperties = { marginBottom: "24px" };
const headingStyle: React.CSSProperties = {
  fontSize: "12px",
  textTransform: "uppercase",
  letterSpacing: "0.05em",
  color: "#6F7F98",
  marginBottom: "10px",
  borderBottom: "1px solid #DCE3F0",
  paddingBottom: "6px",
};
const rowStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: "12px",
  marginBottom: "10px",
};
const labelStyle: React.CSSProperties = { width: "160px", fontSize: "13px", flexShrink: 0 };
const inputStyle: React.CSSProperties = {
  flex: 1,
  padding: "6px 8px",
  border: "1px solid #DCE3F0",
  borderRadius: "4px",
  fontSize: "13px",
};
const disabledButtonStyle: React.CSSProperties = {
  background: "#F1F3FF",
  color: "#6F7F98",
  border: "none",
  borderRadius: "4px",
  padding: "6px 12px",
  fontSize: "12px",
  cursor: "not-allowed",
};
const activeButtonStyle: React.CSSProperties = {
  background: "white",
  color: "#147DFF",
  border: "1px solid #147DFF",
  borderRadius: "4px",
  padding: "6px 12px",
  fontSize: "12px",
  cursor: "pointer",
};

export function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [status, setStatus] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string } | null>(null);

  useEffect(() => {
    invoke<AppSettings>("load_settings")
      .then(setSettings)
      .catch((err) => console.error("[settings] Failed to load:", err));
  }, []);

  const handleSave = async () => {
    setStatus("Saving...");
    try {
      await invoke("save_settings", { settings });
      setStatus("Saved");
      setTimeout(() => setStatus(""), 2000);
    } catch (err) {
      console.error("[settings] Failed to save:", err);
      setStatus("Failed to save - check console");
    }
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const message = await invoke<string>("test_openai_connection", {
        apiKey: settings.openai_api_key,
      });
      setTestResult({ ok: true, message });
    } catch (err) {
      setTestResult({ ok: false, message: String(err) });
    } finally {
      setTesting(false);
    }
  };

  return (
    <div style={{ padding: "20px", maxWidth: "500px" }}>
      <div style={sectionStyle}>
        <div style={headingStyle}>OpenAI</div>
        <div style={rowStyle}>
          <label style={labelStyle}>API Key</label>
          <input
            type="password"
            style={inputStyle}
            value={settings.openai_api_key}
            onChange={(e) => setSettings({ ...settings, openai_api_key: e.target.value })}
            placeholder="sk-..."
          />
        </div>
        <div style={rowStyle}>
          <label style={labelStyle}>Transcription Model</label>
          <input
            type="text"
            style={inputStyle}
            value={settings.transcription_model}
            onChange={(e) => setSettings({ ...settings, transcription_model: e.target.value })}
          />
        </div>
        <div style={rowStyle}>
          <label style={labelStyle}>MOM Model</label>
          <input
            type="text"
            style={inputStyle}
            value={settings.mom_model}
            onChange={(e) => setSettings({ ...settings, mom_model: e.target.value })}
          />
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <button
            onClick={handleTestConnection}
            disabled={testing || settings.openai_api_key.trim() === ""}
            style={
              testing || settings.openai_api_key.trim() === ""
                ? disabledButtonStyle
                : activeButtonStyle
            }
          >
            {testing ? "Testing..." : "Test Connection"}
          </button>
          {testResult && (
            <span
              style={{
                fontSize: "12px",
                color: testResult.ok ? "#1F8A5B" : "#C73636",
              }}
            >
              {testResult.message}
            </span>
          )}
        </div>
        <p style={{ fontSize: "11px", color: "#6F7F98", marginTop: "6px" }}>
          Tests that the key authenticates with OpenAI. It does not
          confirm the specific model above is accessible on your
          account or that diarization will succeed - that's only
          confirmed by actually transcribing a meeting.
        </p>
      </div>

      <div style={sectionStyle}>
        <div style={headingStyle}>
          License{" "}
          <span
            style={{
              fontSize: "10px",
              background: "#F1F3FF",
              color: "#5137E8",
              padding: "2px 6px",
              borderRadius: "4px",
              marginLeft: "6px",
              fontWeight: 600,
            }}
          >
            Coming soon
          </span>
        </div>
        <div style={rowStyle}>
          <label style={labelStyle}>License Key</label>
          <input type="text" style={inputStyle} placeholder="Enter your license key" disabled />
        </div>
        <div style={rowStyle}>
          <label style={labelStyle}>Status</label>
          <span
            style={{
              display: "inline-block",
              padding: "2px 8px",
              borderRadius: "10px",
              fontSize: "11px",
              fontWeight: 600,
              background: "#F1F3FF",
              color: "#6F7F98",
            }}
          >
            Not Activated
          </span>
        </div>
        <button disabled style={disabledButtonStyle}>
          Activate License
        </button>
      </div>

      <div style={sectionStyle}>
        <div style={headingStyle}>General</div>
        <div style={rowStyle}>
          <label style={labelStyle}>Start with Windows</label>
          <input
            type="checkbox"
            checked={settings.start_with_windows}
            onChange={(e) => setSettings({ ...settings, start_with_windows: e.target.checked })}
            title="Saves the preference; actual Windows startup registration not yet implemented"
          />
        </div>
        <div style={rowStyle}>
          <label style={labelStyle}>Meeting notifications</label>
          <input
            type="checkbox"
            checked={settings.meeting_notifications}
            onChange={(e) => setSettings({ ...settings, meeting_notifications: e.target.checked })}
          />
        </div>
      </div>

      <div style={sectionStyle}>
        <div style={headingStyle}>Storage</div>
        <p style={{ fontSize: "12px", color: "#6F7F98" }}>
          Meetings currently save to a "meetings" folder inside the
          project (development mode). A configurable location will be
          added in a later phase.
        </p>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <button
          onClick={handleSave}
          style={{
            background: "#147DFF",
            color: "white",
            border: "none",
            borderRadius: "4px",
            padding: "8px 16px",
            fontSize: "13px",
            cursor: "pointer",
          }}
        >
          Save Settings
        </button>
        <span style={{ fontSize: "12px", color: "#6F7F98" }}>{status}</span>
      </div>
    </div>
  );
}