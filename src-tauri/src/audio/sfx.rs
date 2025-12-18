use rodio::{Decoder, OutputStreamHandle, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

/// SfxEngine handles low-latency sound effects.
/// It pre-loads samples into memory to ensure instant playback.
pub struct SfxEngine {
    stream_handle: OutputStreamHandle,
    samples: HashMap<String, Vec<u8>>,
}

impl SfxEngine {
    pub fn new(stream_handle: OutputStreamHandle) -> Self {
        let mut engine = Self {
            stream_handle,
            samples: HashMap::new(),
        };

        // We load assets here. In a real app we might want to do this async or lazy,
        // but for "fastest execution" and known small set, pre-loading is best.
        engine.preload_assets();

        engine
    }

    fn preload_assets(&mut self) {
        println!("[Audio] Preloading SFX assets...");

        // Map logical IDs to filenames
        let assets = [
            ("nav", "cursorMove.wav"),
            ("domain_switch", "cursorDomainSwitch.wav"),
            ("click", "cursorClick.wav"),
            ("resize", "windowSizeChange.mp3"),
        ];

        // Base path: src/assets/audio/UI
        // Note: In production bundle this needs resource handling, but for dev we use direct path
        let base_path = "../src/assets/audio/UI";

        for (id, filename) in assets.iter() {
            let path = format!("{}/{}", base_path, filename);
            match std::fs::read(&path) {
                Ok(data) => {
                    self.samples.insert(id.to_string(), data);
                    println!("[Audio] Loaded: {}", id);
                }
                Err(e) => {
                    eprintln!("[Audio] Failed to load {}: {}", path, e);
                }
            }
        }
    }

    pub fn play(&self, id: &str) {
        if let Some(data) = self.samples.get(id) {
            let cursor = std::io::Cursor::new(data.clone());

            // Decode on the fly (fast from memory) or cached decoded ??
            // Rodio's Decoder is fast enough for wav usually, but for ultimate speed
            // we could cache the decoded samples if they are raw.
            // For now, re-decoding from memory buffer is good balance.
            match Decoder::new(cursor) {
                Ok(source) => {
                    // Play event - this clones the source effectively
                    let _ = self.stream_handle.play_raw(source.convert_samples());
                }
                Err(e) => eprintln!("[Audio] Decode error for {}: {}", id, e),
            }
        } else {
            eprintln!("[Audio] Sound not found: {}", id);
        }
    }
}
