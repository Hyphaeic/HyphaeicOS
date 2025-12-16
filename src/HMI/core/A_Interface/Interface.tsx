import { createMemo, Show, Switch, Match } from "solid-js";
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
  // Use Switch for reactive content switching
  return (
    <Switch fallback={
      <div style={{ padding: "20px" }}>
        <h1>{props.win.title}</h1>
        <p>Content Key: {props.win.content_key}</p>
      </div>
    }>
      <Match when={props.win.content_key === "TERMINAL"}>
        <TerminalRS windowId={props.win.id} />
      </Match>
      <Match when={props.win.content_key === "TESTING_DUMMY"}>
        <TESTING_DUMMY />
      </Match>
    </Switch>
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
  // Get window ID in each slot (excluding hidden)
  // We strictly track the ID to prevent re-renders when other window properties change
  const leftWindowId = createMemo(() =>
    windowStore.windows.find((w) => w.slot === "Left" && w.state !== "Hidden")?.id
  );

  const rightWindowId = createMemo(() =>
    windowStore.windows.find((w) => w.slot === "Right" && w.state !== "Hidden")?.id
  );

  // Check if any window is maximized (takes full compositor)
  const maximizedWindow = createMemo(() =>
    windowStore.windows.find((w) => w.state === "Maximized")
  );

  // Helper to safely get window data by ID
  const getWindow = (id: string) => windowStore.windows.find((w) => w.id === id);

  return (
    <Controller>
      <OSbar_IC />
      <Compositor_IC
        maximizedWindow={maximizedWindow()}
        leftContent={
          <Show when={leftWindowId()}>
            {(id) => {
              const win = () => getWindow(id());
              return (
                <Window_IC
                  id={id()}
                  title={win()?.title}
                  windowState={win()?.state}
                  contentKey={win()?.content_key}
                >
                  {/* Create a memoized win object or pass specific props to WindowContent */}
                  <WindowContent win={win()!} />
                </Window_IC>
              );
            }}
          </Show>
        }
        rightContent={
          <Show when={rightWindowId()}>
            {(id) => {
              const win = () => getWindow(id());
              return (
                <Window_IC
                  id={id()}
                  title={win()?.title}
                  windowState={win()?.state}
                  contentKey={win()?.content_key}
                >
                  <WindowContent win={win()!} />
                </Window_IC>
              );
            }}
          </Show>
        }
      />
    </Controller>
  );
}
