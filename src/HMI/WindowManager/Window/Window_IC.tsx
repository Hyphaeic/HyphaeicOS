import "./Window_IC.css";
import { JSXElement, Show } from "solid-js";
import Domain from "../../A_Domain/Domain";
import Button_IC from "../../Button/Button_IC";
import { windowActions } from "../../A_Controller/Controller";
import { WindowState } from "../../store";

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
 * - Hidden: Not rendered
 */
export default function Window_IC(props: WindowProps) {
  const isMaximized = () => props.windowState === "Maximized";

  // Action handlers - exposed for external triggering (keybindings)
  const handleMinimize = () => {
    windowActions.minimize(props.id);
  };

  // Toggle between maximized and minimized
  const handleToggleMaximize = () => {
    if (isMaximized()) {
      windowActions.minimize(props.id);
    } else {
      windowActions.maximize(props.id);
    }
  };

  const handleClose = () => {
    windowActions.close(props.id);
  };

  return (
    <Show when={props.windowState !== "Hidden"}>
      <div 
        class={`window ${isMaximized() ? 'window-maximized' : 'window-minimized'}`} 
        id={props.id}
      >
        {/* Window Header - Domain with navigation buttons */}
        <div class="window-header">
          {/* Title display */}
          <div class="window-title-bar">
            <span class="window-title">{props.title || props.contentKey || 'Window'}</span>
          </div>
          
          <Domain 
            id={`${props.id}-header-nav`} 
            layoutMode="list-horizontal"
            class="window-header-domain"
          >
            {/* Minimize Button */}
            <div class="window-header-subcontainer window-header-subcontainer-1">
              <Button_IC
                id={`${props.id}-btn-min`}
                order={0}
                onClick={handleMinimize}
              >
                <span class="window-btn-icon">−</span>
              </Button_IC>
            </div>
            
            {/* Maximize/Restore Toggle Button */}
            <div class="window-header-subcontainer window-header-subcontainer-2">
              <Button_IC
                id={`${props.id}-btn-max`}
                order={1}
                onClick={handleToggleMaximize}
              >
                <span class="window-btn-icon">{isMaximized() ? '❐' : '□'}</span>
              </Button_IC>
            </div>
            
            {/* Close Button */}
            <div class="window-header-subcontainer window-header-subcontainer-3">
              <Button_IC
                id={`${props.id}-btn-close`}
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
