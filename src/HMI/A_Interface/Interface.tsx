import Controller from "../A_Controller/Controller";
import OSbar_IC from "../OSbar/OSbar_IC";
import Compositor_IC from "../WindowManager/Compositor/Compositor_IC";
import Window_IC from "../WindowManager/Window/Window_IC";
import TESTING_DUMMY from "../TESTING_DUMMY/TESTING_DUMMY";

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
 * Interface - Pure visual composition layer
 * 
 * Strict Rules:
 * - No logic
 * - No listeners  
 * - No hooks (except pure UI state if needed)
 * - Only layout and component composition
 */
export default function Interface() {
  return (
    <Controller>
      <OSbar_IC />
      <Compositor_IC 
        leftContent={
          <Window_IC>
            <TESTING_DUMMY />
          </Window_IC>
        }
      />
    </Controller>
  );
}
