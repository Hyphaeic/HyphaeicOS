import "./Window_IC.css";
import { JSXElement } from "solid-js";
import Domain from "../../B_Domain/Domain";
import Button_IC from "../../Button/Button_IC";

interface WindowProps {
  children?: JSXElement;
}

/**
 * Window_IC - Individual Window Component
 * 
 * Structure:
 * - Header: Height = 1/27th screen width (matches OSbar width)
 *   - Contains a Domain with 3 navigable buttons (4, 5, 6)
 * - Body: Fills remaining vertical space
 * - Fills parent container (SubContainer_Right)
 */
export default function Window_IC(props: WindowProps) {
  return (
    <div class="window">
      {/* Window Header - Domain with navigation buttons */}
      <div class="window-header">
        <Domain 
          id="window-header-nav" 
          layoutMode="list-horizontal"
          class="window-header-domain"
        >
          {/* Button 4 */}
          <div class="window-header-subcontainer window-header-subcontainer-1">
            <Button_IC
              id="window-btn-4"
              order={0}
              onClick={() => console.log("Button 4 clicked")}
            >
              <span>4</span>
            </Button_IC>
          </div>
          
          {/* Button 5 */}
          <div class="window-header-subcontainer window-header-subcontainer-2">
            <Button_IC
              id="window-btn-5"
              order={1}
              onClick={() => console.log("Button 5 clicked")}
            >
              <span>5</span>
            </Button_IC>
          </div>
          
          {/* Button 6 */}
          <div class="window-header-subcontainer window-header-subcontainer-3">
            <Button_IC
              id="window-btn-6"
              order={2}
              onClick={() => console.log("Button 6 clicked")}
            >
              <span>6</span>
            </Button_IC>
          </div>
        </Domain>
      </div>
      
      {/* Window Body - flexible content area */}
      <div class="window-body">
        {props.children}
      </div>
    </div>
  );
}

