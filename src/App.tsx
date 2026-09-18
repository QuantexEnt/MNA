import { MeetingNotification } from "./components/MeetingNotification";

function App() {
  return (
    <div style={{ padding: "20px", fontFamily: "sans-serif" }}>
      <MeetingNotification />
      <h1 style={{ fontSize: "18px" }}>Meeting Notes Assistant</h1>
      <p style={{ color: "#666" }}>
        Running in the background. Waiting for a Teams meeting...
      </p>
    </div>
  );
}

export default App;