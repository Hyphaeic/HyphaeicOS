import "./Compositor_IC.css";
import { JSXElement } from "solid-js";

interface CompositorProps {
  leftContent?: JSXElement;
  rightContent?: JSXElement;
}

/**
 * Compositor_IC - Window Manager Main Container
 * 
 * Architecture:
 * - Two side-by-side subcontainers (50/50 split)
 * - Left: Reserved for system UI (dock, sidebar, etc.)
 * - Right: Active window content area
 * - Aligns with OSbar positioning (responds to aspect ratio)
 */
export default function Compositor_IC(props: CompositorProps) {
  return (
    <div class="compositor">
      {/* Left subcontainer - future: dock, file browser, etc. */}
      <div class="compositor-subcontainer compositor-subcontainer-left">
        {props.leftContent}
      </div>
      
      {/* Right subcontainer - houses Window_IC instances */}
      <div class="compositor-subcontainer compositor-subcontainer-right">
        {props.rightContent}
      </div>
    </div>
  );
}

