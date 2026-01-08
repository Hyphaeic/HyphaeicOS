import { createSignal, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import "./background_IS.css";

/**
 * Asset information returned from the Rust backend
 */
interface AssetInfo {
  path: string;
  cached: boolean;
  asset_type: string;
}

/**
 * Props for the Background component
 */
interface BackgroundProps {
  /** URL of the background image to load */
  url?: string;
  /** CSS class to add to the container */
  class?: string;
  /** Children to render on top of the background */
  children?: any;
}

/**
 * Background component that loads an image from a remote URL
 * using the Rust asset loader for caching and performance.
 */
export default function BackgroundIC(props: BackgroundProps) {
  const [imageSrc, setImageSrc] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [fromCache, setFromCache] = createSignal(false);

  // Default background URL
  const backgroundUrl = () => props.url ?? "https://src.hyphaeic.com/website/backgrounds/1.jpg";

  onMount(async () => {
    try {
      setLoading(true);
      setError(null);

      // Call the Rust asset loader to download/cache the image
      const assetInfo = await invoke<AssetInfo>("load_asset", {
        url: backgroundUrl(),
        assetType: "Image",
      });

      // Convert the local file path to a URL the webview can display
      const assetUrl = convertFileSrc(assetInfo.path);
      setImageSrc(assetUrl);
      setFromCache(assetInfo.cached);

      if (assetInfo.cached) {
        console.log("[BackgroundIC] Loaded from cache:", assetInfo.path);
      } else {
        console.log("[BackgroundIC] Downloaded and cached:", assetInfo.path);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error("[BackgroundIC] Failed to load background image:", err);
    } finally {
      setLoading(false);
    }
  });

  return (
    <div class={`background-container ${props.class ?? ""}`}>
      {/* Loading state */}
      <Show when={loading()}>
        <div class="background-loading">
          <div class="loading-spinner" />
          <span>Loading background...</span>
        </div>
      </Show>

      {/* Error state */}
      <Show when={error()}>
        <div class="background-error">
          <span>Failed to load background</span>
          <small>{error()}</small>
        </div>
      </Show>

      {/* Background image */}
      <Show when={imageSrc()}>
        <img
          src={imageSrc()!}
          alt="Background"
          class="background-image"
          data-cached={fromCache()}
        />
      </Show>

      {/* Children rendered on top of background */}
      <Show when={props.children}>
        <div class="background-content">{props.children}</div>
      </Show>
    </div>
  );
}

