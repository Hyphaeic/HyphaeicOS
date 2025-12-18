use crate::asset_loader::load_local_audio;
use rodio::{buffer::SamplesBuffer, Decoder, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug, Eq, Hash, Copy)]
enum AmbientTrack {
    None,
    Home,
    WindowHeader,
    Terminal,
}

pub struct AmbienceEngine {
    // We utilize a sender one-way channel to communicate with the fade thread
    fade_tx: Sender<AmbientTrack>,
    current_track: AmbientTrack,
}

impl AmbienceEngine {
    pub fn new(stream_handle: OutputStreamHandle) -> Self {
        let (tx, rx) = channel();

        // Load assets and initialize Sinks in the main thread (or we could move this to thread)
        // But creating Sinks usually requires the stream handle.
        let sinks = Self::initialize_sinks(&stream_handle);

        // Spawn the Fade Manager thread
        // This thread owns the Sink handles and manages their volume.
        thread::spawn(move || {
            let mut target_track = AmbientTrack::None;
            // Map of Track -> Sink
            // Note: Sink is not Clone, but it is Send. We move Sinks into this thread.
            let sink_map = sinks;

            let mut last_tick = std::time::Instant::now();
            // 2.0 seconds fade for very smooth transition (user complained of stuttering)
            // Stuttering might be due to step size, so delta-time will help.
            let fade_duration = 1.5;

            loop {
                // Calculation delta time
                let now = std::time::Instant::now();
                let dt = now.duration_since(last_tick).as_secs_f32();
                last_tick = now;

                // 1. Process pending command
                if let Ok(new_target) = rx.try_recv() {
                    println!("[Audio] Fader received target: {:?}", new_target);
                    target_track = new_target;
                }

                // 2. Adjust volumes (Crossfade logic with delta time)
                // We want to move volume from 0 -> 1 over fade_duration
                let vol_change = (1.0 / fade_duration) * dt;

                for (track_id, sink) in &sink_map {
                    let current_vol = sink.volume();
                    let target_vol = if *track_id == target_track { 1.0 } else { 0.0 };

                    if (current_vol - target_vol).abs() > 0.001 {
                        let new_vol = if current_vol < target_vol {
                            (current_vol + vol_change).min(target_vol)
                        } else {
                            (current_vol - vol_change).max(target_vol)
                        };
                        sink.set_volume(new_vol);
                    } else if current_vol != target_vol {
                        sink.set_volume(target_vol);
                    }
                }

                // 3. Sleep
                // 10ms = 100 updates/second for smoothness
                thread::sleep(Duration::from_millis(10));
            }
        });

        let mut engine = Self {
            fade_tx: tx,
            current_track: AmbientTrack::None,
        };

        // Start default
        engine.update_context("osbar-nav");
        engine
    }

    fn initialize_sinks(stream_handle: &OutputStreamHandle) -> HashMap<AmbientTrack, Sink> {
        println!("[Audio] Initializing Virtual Timeline Sinks...");
        let mut sink_map = HashMap::new();

        // 1. Load Assets
        let assets = [
            (AmbientTrack::Home, "home.mp3"),
            (AmbientTrack::WindowHeader, "windowHeader.mp3"),
            (AmbientTrack::Terminal, "terminal.mp3"),
        ];

        for (track_id, filename) in assets.iter() {
            match load_local_audio(filename) {
                Ok(data) => {
                    let cursor = Cursor::new(data);
                    match Decoder::new(cursor) {
                        Ok(decoder) => {
                            // Decode to PCM - MUST EXTRACT METADATA BEFORE CONSUMING DECODER
                            let channels = decoder.channels();
                            let sample_rate = decoder.sample_rate();
                            let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();
                            let buffer = SamplesBuffer::new(channels, sample_rate, samples);

                            // Create Sink
                            match Sink::try_new(stream_handle) {
                                Ok(sink) => {
                                    // Start playing silently
                                    sink.append(buffer.repeat_infinite());
                                    sink.set_volume(0.0);
                                    sink.play();
                                    sink_map.insert(*track_id, sink);
                                    println!("[Audio] Sink ready (silent): {:?}", track_id);
                                }
                                Err(e) => eprintln!("[Audio] Sink creation failed: {}", e),
                            }
                        }
                        Err(e) => eprintln!("[Audio] Decode failed for {}: {}", filename, e),
                    }
                }
                Err(e) => eprintln!("[Audio] Asset load failed for {}: {}", filename, e),
            }
        }

        sink_map
    }

    /// Called when the active domain changes.
    pub fn update_context(&mut self, domain_id: &str) {
        // Determine the target track based on domain string patterns.
        let target_track = if domain_id.contains("osbar") {
            AmbientTrack::Home
        } else if domain_id.contains("header") {
            AmbientTrack::WindowHeader
        } else if domain_id.contains("terminal") {
            AmbientTrack::Terminal
        } else {
            AmbientTrack::Terminal
        };

        // Only switch if the track actually changes
        if target_track != self.current_track {
            println!(
                "[Audio] Switching ambience: {:?} -> {:?}",
                self.current_track, target_track
            );
            self.current_track = target_track;
            // Send command to fade thread
            let _ = self.fade_tx.send(target_track);
        }
    }
}
