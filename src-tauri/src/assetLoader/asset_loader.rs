use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tauri::Manager;

/// Information about a loaded asset
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetInfo {
    /// Absolute path to the cached asset file
    pub path: String,
    /// Whether the asset was loaded from cache (true) or freshly downloaded (false)
    pub cached: bool,
    /// The type of asset (e.g., "Image", "Video")
    pub asset_type: String,
}

/// Supported asset types for the loader
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AssetType {
    Image,
    Video,
    Audio,
    Document,
    Other(String),
}

impl AssetType {
    /// Get the default file extension for this asset type
    fn extension(&self) -> &str {
        match self {
            AssetType::Image => "jpg",
            AssetType::Video => "mp4",
            AssetType::Audio => "mp3",
            AssetType::Document => "pdf",
            AssetType::Other(ext) => ext,
        }
    }

    /// Get the subdirectory name for this asset type
    fn subdirectory(&self) -> &str {
        match self {
            AssetType::Image => "images",
            AssetType::Video => "videos",
            AssetType::Audio => "audio",
            AssetType::Document => "documents",
            AssetType::Other(_) => "other",
        }
    }

    /// Get a display name for this asset type
    fn display_name(&self) -> String {
        match self {
            AssetType::Image => "Image".to_string(),
            AssetType::Video => "Video".to_string(),
            AssetType::Audio => "Audio".to_string(),
            AssetType::Document => "Document".to_string(),
            AssetType::Other(ext) => format!("Other({})", ext),
        }
    }
}

/// Generate a cache key from URL (hash-based filename)
fn url_to_filename(url: &str, asset_type: &AssetType) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:x}.{}", hash, asset_type.extension())
}

/// Get or create the assets directory in app data
async fn get_assets_dir(app: &tauri::AppHandle, asset_type: &AssetType) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let assets_dir = app_data_dir.join("assets").join(asset_type.subdirectory());

    // Create directory if it doesn't exist
    tokio::fs::create_dir_all(&assets_dir)
        .await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;

    Ok(assets_dir)
}

/// Load an asset from a URL, caching it locally
///
/// # Arguments
/// * `url` - The URL to download the asset from
/// * `asset_type` - The type of asset (Image, Video, Audio, Document, or Other)
/// * `app` - The Tauri app handle (injected automatically)
///
/// # Returns
/// * `AssetInfo` containing the local path and cache status
#[tauri::command]
pub async fn load_asset(
    url: String,
    asset_type: AssetType,
    app: tauri::AppHandle,
) -> Result<AssetInfo, String> {
    // Get assets directory
    let assets_dir = get_assets_dir(&app, &asset_type).await?;

    // Generate cache filename
    let filename = url_to_filename(&url, &asset_type);
    let file_path = assets_dir.join(&filename);

    // Check if already cached
    if file_path.exists() {
        return Ok(AssetInfo {
            path: file_path.to_string_lossy().to_string(),
            cached: true,
            asset_type: asset_type.display_name(),
        });
    }

    // Download the asset
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to download asset: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Save to disk
    tokio::fs::write(&file_path, &bytes)
        .await
        .map_err(|e| format!("Failed to save asset: {}", e))?;

    Ok(AssetInfo {
        path: file_path.to_string_lossy().to_string(),
        cached: false,
        asset_type: asset_type.display_name(),
    })
}

/// Clear the asset cache
///
/// # Arguments
/// * `asset_type` - Optional. If provided, clears only that asset type's cache.
///                  If None, clears all cached assets.
/// * `app` - The Tauri app handle (injected automatically)
///
/// # Returns
/// * A message indicating what was cleared
#[tauri::command]
pub async fn clear_asset_cache(
    asset_type: Option<AssetType>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let assets_dir = app_data_dir.join("assets");

    if let Some(asset_type) = asset_type {
        let type_dir = assets_dir.join(asset_type.subdirectory());
        if type_dir.exists() {
            tokio::fs::remove_dir_all(&type_dir)
                .await
                .map_err(|e| format!("Failed to clear cache: {}", e))?;
        }
        Ok(format!("Cleared cache for {}", asset_type.display_name()))
    } else {
        if assets_dir.exists() {
            tokio::fs::remove_dir_all(&assets_dir)
                .await
                .map_err(|e| format!("Failed to clear cache: {}", e))?;
        }
        Ok("Cleared all asset cache".to_string())
    }
}

/// Check if an asset is already cached
///
/// # Arguments
/// * `url` - The URL of the asset to check
/// * `asset_type` - The type of asset
/// * `app` - The Tauri app handle (injected automatically)
///
/// # Returns
/// * `true` if the asset is cached, `false` otherwise
#[tauri::command]
pub async fn is_asset_cached(
    url: String,
    asset_type: AssetType,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let assets_dir = get_assets_dir(&app, &asset_type).await?;
    let filename = url_to_filename(&url, &asset_type);
    let file_path = assets_dir.join(&filename);

    Ok(file_path.exists())
}

/// Get the cache path for an asset without downloading
///
/// # Arguments
/// * `url` - The URL of the asset
/// * `asset_type` - The type of asset
/// * `app` - The Tauri app handle (injected automatically)
///
/// # Returns
/// * The path where the asset would be cached (may or may not exist)
#[tauri::command]
pub async fn get_asset_cache_path(
    url: String,
    asset_type: AssetType,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let assets_dir = get_assets_dir(&app, &asset_type).await?;
    let filename = url_to_filename(&url, &asset_type);
    let file_path = assets_dir.join(&filename);

    Ok(file_path.to_string_lossy().to_string())
}

/// Load a local audio asset (e.g. from source/assets/audio/ambient)
/// This simulates a centralized asset loader for static content.
pub fn load_local_audio(filename: &str) -> std::io::Result<Vec<u8>> {
    // In dev mode, we look in the src directory relative to execution
    let base_path = "../src/assets/audio/ambient";
    let path = std::path::Path::new(base_path).join(filename);
    std::fs::read(path)
}
