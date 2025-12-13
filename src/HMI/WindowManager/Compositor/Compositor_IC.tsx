import "./Compositor_IC.css";
import { JSXElement } from "solid-js";
import { WindowInstance } from "../../store";

interface CompositorProps {
  /** Content for the left subcontainer */
  leftContent?: JSXElement;
  /** Content for the right subcontainer */
  rightContent?: JSXElement;
  /** If a window is maximized, it takes over the entire compositor */
  maximizedWindow?: WindowInstance;
}

/**
 * Compositor_IC - Window Manager Main Container
 * 
 * Architecture:
 * - Two side-by-side subcontainers (50/50 split)
 * - Left: First window slot
 * - Right: Second window slot
 * - When a window is Maximized, CSS handles making it span the entire compositor
 *   (the Window_IC component stays in its slot to prevent unmounting)
 * - Aligns with OSbar positioning (responds to aspect ratio)
 */
export default function Compositor_IC(props: CompositorProps) {
  // Determine which slot has the maximized window
  const isLeftMaximized = () => props.maximizedWindow?.slot === "Left";
  const isRightMaximized = () => props.maximizedWindow?.slot === "Right";

  return (
    <div class="compositor">
      {/* Left subcontainer - first window slot */}
      <div class={`compositor-subcontainer compositor-subcontainer-left ${
        isLeftMaximized() ? 'compositor-subcontainer-maximized' : ''
      } ${isRightMaximized() ? 'compositor-subcontainer-hidden' : ''}`}>
        {props.leftContent}
      </div>
      
      {/* Right subcontainer - second window slot */}
      <div class={`compositor-subcontainer compositor-subcontainer-right ${
        isRightMaximized() ? 'compositor-subcontainer-maximized' : ''
      } ${isLeftMaximized() ? 'compositor-subcontainer-hidden' : ''}`}>
        {props.rightContent}
      </div>
    </div>
  );
}
