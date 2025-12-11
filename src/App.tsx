import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import BackgroundIC from "./HMI/Background/background_IC";
import Interface from "./HMI/A_Interface/Interface";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  return (
    <BackgroundIC>
      <Interface />
    </BackgroundIC>
  );
}

export default App;
