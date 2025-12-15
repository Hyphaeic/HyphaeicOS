import { createMemo, Show } from "solid-js";
import { windowStore, WindowInstance } from "../../store";
import Controller from "../A_Controller/Controller";
import OSbar_IC from "../OSbar/OSbar_IC";
import Compositor_IC from "../WindowManager/Compositor/Compositor_IC";
import Window_IC from "../WindowManager/Window/Window_IC";
import TESTING_DUMMY from "../../TESTING_DUMMY/TESTING_DUMMY";
import TerminalRS from "../../subcomponents/rustTerm/terminal_rs";

// ============================================================================
// INTERFACE - The Visual Environment
// ============================================================================
// Pure layout file. NO logic, NO listeners, NO hooks.
// 
// Architecture:
// - Controller: Centralized input handling (first child, wraps all visuals)
// - OSbar_IC: Navigation bar
// - Compositor_IC: Window management layer
// ============================================================================

/**
 * Renders window content based on content_key
 */
function WindowContent(props: { win: WindowInstance }) {
  // Route content based on content_key
  if (props.win.content_key === "TERMINAL") {
    return <TerminalRS windowId={props.win.id} />;
  }

  if (props.win.content_key === "TESTING_DUMMY") {
    return <TESTING_DUMMY />;
  }

  // Default fallback for unknown content keys
  return (
    <div style={{ padding: "20px" }}>
      <h1>{props.win.title}</h1>
      <p>Content Key: {props.win.content_key}</p>
    </div>
  );
}

/**
 * Interface - Pure visual composition layer
 * 
 * Window Placement Rules:
 * - Each window has an assigned slot (Left or Right)
 * - Minimized: Window fills its slot (half the compositor)
 * - Maximized: Window spans entire compositor (full width)
 * - Only one window per slot
 */
export default function Interface() {
  // Get window in each slot (excluding hidden)
  const leftWindow = createMemo(() =>
    windowStore.windows.find((w) => w.slot === "Left" && w.state !== "Hidden")
  );

  const rightWindow = createMemo(() =>
    windowStore.windows.find((w) => w.slot === "Right" && w.state !== "Hidden")
  );

  // Check if any window is maximized (takes full compositor)
  const maximizedWindow = createMemo(() =>
    windowStore.windows.find((w) => w.state === "Maximized")
  );

  return (
    <Controller>
      <OSbar_IC />
      <Compositor_IC
        maximizedWindow={maximizedWindow()}
        leftContent={
          // Render window without keyed to prevent recreation
          // Window_IC will handle its own state preservation
          <Show when={leftWindow()}>
            {(win) => (
              <Window_IC
                id={win().id}
                title={win().title}
                windowState={win().state}
                contentKey={win().content_key}
              >
                <WindowContent win={win()} />
              </Window_IC>
            )}
          </Show>
        }
        rightContent={
          // Render window without keyed to prevent recreation
          <Show when={rightWindow()}>
            {(win) => (
              <Window_IC
                id={win().id}
                title={win().title}
                windowState={win().state}
                contentKey={win().content_key}
              >
                <WindowContent win={win()} />
              </Window_IC>
            )}
          </Show>
        }
      />
    </Controller>
  );
}
