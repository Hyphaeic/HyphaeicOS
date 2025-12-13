import BackgroundIC from "./HMI/Background/background_IC";
import Interface from "./HMI/A_Interface/Interface";
import "./App.css";

// ============================================================================
// APP - Root Component
// ============================================================================
// Minimal root that composes Background + Interface.
// All input handling is inside Interface via Controller.
// ============================================================================

function App() {
  return (
    <BackgroundIC>
      <Interface />
    </BackgroundIC>
  );
}

export default App;
