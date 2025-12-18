pub mod ambience;
pub mod sfx;

use self::ambience::AmbienceEngine;
use self::sfx::SfxEngine;
use rodio::{OutputStream, OutputStreamHandle};
use std::sync::{Arc, Mutex};

/// Central controller for the audio system.
pub struct AudioSystem {
    // Only keep the handle, which is Send + Sync (internally Arc)
    stream_handle: OutputStreamHandle,

    sfx: SfxEngine,
    ambience: AmbienceEngine,
}

impl AudioSystem {
    /// Returns (AudioSystem, OutputStream).
    /// IMPORTANT: The caller MUST keep the OutputStream alive, but it cannot be shared across threads.
    pub fn new() -> (Self, OutputStream) {
        // Initialize audio device
        let (stream, stream_handle) =
            OutputStream::try_default().expect("Failed to get default audio output");

        let sfx = SfxEngine::new(stream_handle.clone());
        let ambience = AmbienceEngine::new(stream_handle.clone());

        println!("[Audio] System initialized");

        (
            Self {
                stream_handle,
                sfx,
                ambience,
            },
            stream,
        )
    }

    pub fn play_sfx(&self, id: &str) {
        self.sfx.play(id);
    }

    pub fn on_domain_change(&mut self, domain_id: &str) {
        // SFX feedback for the switch itself
        self.play_sfx("domain_switch");

        // Update ambience context
        self.ambience.update_context(domain_id);
    }
}

// Global state wrapper
pub struct AudioState(pub Arc<Mutex<AudioSystem>>);
