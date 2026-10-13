import { MeetingNotification } from "./components/MeetingNotification";
import logo from "./assets/Horizontal-_MNA.png";

function App() {
  return (
    <div style={{ padding: "20px", fontFamily: "sans-serif" }}>
      <MeetingNotification />
      <h1>Meeting Notes Assistant</h1> line with: <img src={logo} alt="Meeting Notes Assistant" style={{ height: "32px", marginBottom: "16px" }} />
        <p style={{ color: "#666" }}>
        Running in the background. Waiting for a Teams meeting...
      </p>
    </div>
  );
}

export default App;