import "./Window_IC.css";
import { JSXElement, Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Domain from "../../A_Domain/Domain";
import Button_IC from "../../Button/Button_IC";
import { windowActions } from "../../A_Controller/Controller";
import { WindowState } from "../../../store";

interface WindowProps {
  id: string;
  title?: string;
  windowState?: WindowState;
  contentKey?: string;
  children?: JSXElement;
}

/**
 * Window_IC - Individual Window Component
 * 
 * Structure:
 * - Header: Height = 1/27th screen width (matches OSbar width)
 *   - Contains a Domain with 3 navigable buttons: Minimize, Maximize, Close
 * - Body: Fills remaining vertical space
 * - Fills parent container
 * 
 * States:
 * - Minimized: Half-size window in assigned slot
 * - Maximized: Full-size window spanning entire compositor
 * - Closing: Window is animating out (will be removed after animation)
 * - Hidden: Not rendered
 */
export default function Window_IC(props: WindowProps) {
  const isMaximized = () => props.windowState === "Maximized";

  // Local closing state that persists across prop changes
  // Once set to true, it stays true until component unmounts
  const [localIsClosing, setLocalIsClosing] = createSignal(false);

  // Create a safe ID accessor that handles potential stale access during cleanup
  // We use a signal to cache the LAST VALID ID. If props.id throws (stale), we keep the old ID.
  // This prevents the ID from becoming "" during cleanup, which causes ghost domains.
  const [safeId, setSafeId] = createSignal("");

  // Initialize safely
  try {
    setSafeId(props.id);
  } catch (e) { /* ignore */ }

  createEffect(() => {
    try {
      const newId = props.id;
      setSafeId(newId);
    } catch (e) {
      // If usage of props.id throws (stale), we simply don't update safeId.
      // It retains the last valid UUID.
    }
  });

  // Track if window has started closing
  createEffect(() => {
    if (props.windowState === "Closing" && !localIsClosing()) {
      setLocalIsClosing(true);
    }
  });

  const isClosing = () => localIsClosing();

  let windowRef: HTMLDivElement | undefined;
  let animationEndHandler: ((e: AnimationEvent) => void) | undefined;

  // Set up animation end listener
  onMount(() => {
    animationEndHandler = (e: AnimationEvent) => {
      // Only handle the fadeOutBlur animation (not fadeInBlur)
      if (e.animationName === 'fadeOutBlur' && isClosing()) {
        // Animation completed, now remove the window
        invoke('remove_window', { id: safeId() }).catch(console.error);
      }
    };

    if (windowRef) {
      windowRef.addEventListener('animationend', animationEndHandler as EventListener);
    }

    onCleanup(() => {
      if (windowRef && animationEndHandler) {
        windowRef.removeEventListener('animationend', animationEndHandler as EventListener);
      }
    });
  });

  // Fallback timeout in case animationend doesn't fire
  createEffect(() => {
    if (isClosing()) {
      const fallbackTimeout = setTimeout(() => {
        console.warn('[Window_IC] Animation fallback triggered for', safeId());
        invoke('remove_window', { id: safeId() }).catch(console.error);
      }, 500); // Slightly longer than animation duration as fallback

      onCleanup(() => clearTimeout(fallbackTimeout));
    }
  });

  // Action handlers - exposed for external triggering (keybindings)
  const handleMinimize = () => {
    windowActions.minimize(safeId());
  };

  // Toggle between maximized and minimized
  const handleToggleMaximize = () => {
    if (isMaximized()) {
      windowActions.minimize(safeId());
    } else {
      windowActions.maximize(safeId());
    }
  };

  const handleClose = () => {
    // Trigger closing state (Rust will emit window-state-changed event)
    windowActions.close(safeId());
  };

  return (
    <Show when={props.windowState !== "Hidden"}>
      <div
        ref={windowRef}
        class={`window ${isMaximized() ? 'window-maximized' : 'window-minimized'} ${isClosing() ? 'window-exiting' : ''}`}
        id={safeId()}
      >
        {/* Window Header - Domain with navigation buttons */}
        <div class="window-header">
          {/* Title display */}
          <div class="window-title-bar">
            <span class="window-title">{props.title || props.contentKey || 'Window'}</span>
          </div>

          <Domain
            id={`${safeId()}-header-nav`}
            layoutMode="list-horizontal"
            class="window-header-domain"
          >
            {/* Minimize Button */}
            <div class="window-header-subcontainer window-header-subcontainer-1">
              <Button_IC
                id={`${safeId()}-btn-min`}
                order={0}
                onClick={handleMinimize}
              >
                <span class="window-btn-icon">−</span>
              </Button_IC>
            </div>

            {/* Maximize/Restore Toggle Button */}
            <div class="window-header-subcontainer window-header-subcontainer-2">
              <Button_IC
                id={`${safeId()}-btn-max`}
                order={1}
                onClick={handleToggleMaximize}
              >
                <span class="window-btn-icon">{isMaximized() ? '❐' : '□'}</span>
              </Button_IC>
            </div>

            {/* Close Button */}
            <div class="window-header-subcontainer window-header-subcontainer-3">
              <Button_IC
                id={`${safeId()}-btn-close`}
                order={2}
                onClick={handleClose}
              >
                <span class="window-btn-icon">×</span>
              </Button_IC>
            </div>
          </Domain>
        </div>

        {/* Window Body */}
        <div class="window-body">
          {props.children}
        </div>
      </div>
    </Show>
  );
}

// Export action handlers for external use (keybindings, etc.)
export { windowActions };
