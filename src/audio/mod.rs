//! Audio system for Tesseract game engine.
//!
//! Provides spatial audio, sound effects, and music playback with
//! 3D positioning based on listener and source locations.

mod listener;
mod manager;
mod source;
mod spatial;

pub use listener::AudioListener;
pub use manager::{AudioManager, AudioChannel, ChannelSettings};
pub use source::{AudioSource, PlaybackState, SoundHandle};
pub use spatial::{AttenuationModel, SpatialSettings};
