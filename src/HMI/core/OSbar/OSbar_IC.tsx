import { createSignal, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import hyphaeicLogo from "../../../assets/images/hyphaeicLogo.png";
import Domain from "../A_Domain/Domain";
import Button_IC from "../Button/Button_IC";
import "./OSBar_IS.css";

/**
 * Determines layout mode based on aspect ratio
 * Portrait/Square (≤1:1) = horizontal bar at bottom = list-horizontal (A/D nav)
 * Landscape (>1:1) = vertical bar on left = list-vertical (W/S nav)
 */
function getLayoutMode(): 'list-vertical' | 'list-horizontal' {
  const aspectRatio = window.innerWidth / window.innerHeight;
  return aspectRatio > 1 ? 'list-vertical' : 'list-horizontal';
}

export default function OSbar_IC() {
  const [layoutMode, setLayoutMode] = createSignal<'list-vertical' | 'list-horizontal'>(getLayoutMode());

  // Update layout mode on resize and notify Rust backend
  onMount(() => {
    const handleResize = async () => {
      const newMode = getLayoutMode();
      const oldMode = layoutMode();

      if (newMode !== oldMode) {
        setLayoutMode(newMode);
        console.log(`OSbar layout changed: ${oldMode} -> ${newMode}`);

        // Update Rust backend with new layout mode
        try {
          await invoke('update_domain_layout', {
            domainId: 'osbar-nav',
            layoutMode: newMode,
            gridColumns: null
          });
        } catch (error) {
          console.error('Failed to update domain layout:', error);
        }
      }
    };

    window.addEventListener('resize', handleResize);

    onCleanup(() => {
      window.removeEventListener('resize', handleResize);
    });
  });

  return (
    <div class="osbar">
      {/* Subcontainer 0: Logo */}
      <div class="osbar-subcontainer osbar-subcontainer-logo">
        <img
          src={hyphaeicLogo}
          alt="Hyphaeic Logo"
          class="osbar-logo"
        />
      </div>

      {/* Navigation Domain containing 3 buttons */}
      <Domain
        id="osbar-nav"
        layoutMode={layoutMode()}
        class="osbar-domain"
      >
        {/* Button 1 */}
        <div class="osbar-subcontainer osbar-subcontainer-1">
          <Button_IC
            id="osbar-btn-1"
            order={0}
            onClick={() => {
              console.log("Button 1 clicked");
              invoke('spawn_window', {
                contentKey: 'TESTING_DUMMY',
                sourceElementId: 'osbar-btn-1',
                sourceDomainId: 'osbar-nav'
              }).catch(console.error);
            }}
          >
            <span>1</span>
          </Button_IC>
        </div>

        {/* Button 2 */}
        <div class="osbar-subcontainer osbar-subcontainer-2">
          <Button_IC
            id="osbar-btn-2"
            order={1}
            onClick={() => {
              console.log("Button 2 clicked");
              invoke('spawn_window', {
                contentKey: 'EMPTY_WINDOW_2',
                sourceElementId: 'osbar-btn-2',
                sourceDomainId: 'osbar-nav'
              }).catch(console.error);
            }}
          >
            <span>2</span>
          </Button_IC>
        </div>

        {/* Button 3 */}
        <div class="osbar-subcontainer osbar-subcontainer-3">
          <Button_IC
            id="osbar-btn-3"
            order={2}
            onClick={() => {
              console.log("Button 3 clicked - spawning terminal");
              invoke('spawn_window', {
                contentKey: 'TERMINAL',
                sourceElementId: 'osbar-btn-3',
                sourceDomainId: 'osbar-nav'
              }).catch(console.error);
            }}
          >
            <span>3</span>
          </Button_IC>
        </div>
      </Domain>
    </div>
  );
}
