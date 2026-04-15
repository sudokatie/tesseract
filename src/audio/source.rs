//! Audio source component for playing sounds in the world.

use crate::audio::spatial::SpatialSettings;

/// Handle to a playing sound, used to control playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub(crate) u64);

impl SoundHandle {
    /// Create a new sound handle.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID.
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Current playback state of an audio source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Sound is not playing.
    #[default]
    Stopped,
    /// Sound is currently playing.
    Playing,
    /// Sound is paused.
    Paused,
}

/// An audio source that can play sounds in 3D space.
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Path to the audio file to play.
    pub sound_path: String,
    /// Volume multiplier (0.0 to 1.0).
    pub volume: f32,
    /// Playback speed (1.0 = normal).
    pub pitch: f32,
    /// Whether the sound should loop.
    pub looping: bool,
    /// Whether the sound plays automatically when created.
    pub autoplay: bool,
    /// Current playback state.
    pub state: PlaybackState,
    /// Whether to use 3D spatial audio.
    pub spatial: bool,
    /// Spatial audio settings.
    pub spatial_settings: SpatialSettings,
    /// Internal handle to the playing sound.
    pub(crate) handle: Option<SoundHandle>,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            sound_path: String::new(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            autoplay: false,
            state: PlaybackState::Stopped,
            spatial: true,
            spatial_settings: SpatialSettings::default(),
            handle: None,
        }
    }
}

impl AudioSource {
    /// Create a new audio source for a sound file.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            sound_path: path.into(),
            ..Default::default()
        }
    }

    /// Create a sound effect (spatial, doesn't loop).
    pub fn effect(path: impl Into<String>) -> Self {
        Self {
            sound_path: path.into(),
            spatial: true,
            looping: false,
            ..Default::default()
        }
    }

    /// Create background music (non-spatial, loops).
    pub fn music(path: impl Into<String>) -> Self {
        Self {
            sound_path: path.into(),
            spatial: false,
            looping: true,
            volume: 0.7,
            ..Default::default()
        }
    }

    /// Create an ambient sound (spatial, loops).
    pub fn ambient(path: impl Into<String>) -> Self {
        Self {
            sound_path: path.into(),
            spatial: true,
            looping: true,
            spatial_settings: SpatialSettings::distant(),
            ..Default::default()
        }
    }

    /// Set the volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set the pitch/playback speed.
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.max(0.1);
        self
    }

    /// Enable or disable looping.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Enable autoplay.
    pub fn with_autoplay(mut self) -> Self {
        self.autoplay = true;
        self
    }

    /// Set spatial audio settings.
    pub fn with_spatial(mut self, settings: SpatialSettings) -> Self {
        self.spatial = true;
        self.spatial_settings = settings;
        self
    }

    /// Disable spatial audio (2D sound).
    pub fn without_spatial(mut self) -> Self {
        self.spatial = false;
        self
    }

    /// Check if the source is currently playing.
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Check if the source is paused.
    pub fn is_paused(&self) -> bool {
        self.state == PlaybackState::Paused
    }

    /// Check if the source is stopped.
    pub fn is_stopped(&self) -> bool {
        self.state == PlaybackState::Stopped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_source() {
        let source = AudioSource::default();
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.pitch, 1.0);
        assert!(!source.looping);
        assert!(!source.autoplay);
        assert!(source.spatial);
        assert!(source.is_stopped());
    }

    #[test]
    fn test_new_source() {
        let source = AudioSource::new("sounds/explosion.ogg");
        assert_eq!(source.sound_path, "sounds/explosion.ogg");
    }

    #[test]
    fn test_effect_preset() {
        let source = AudioSource::effect("sounds/hit.ogg");
        assert!(source.spatial);
        assert!(!source.looping);
    }

    #[test]
    fn test_music_preset() {
        let source = AudioSource::music("music/theme.ogg");
        assert!(!source.spatial);
        assert!(source.looping);
        assert_eq!(source.volume, 0.7);
    }

    #[test]
    fn test_ambient_preset() {
        let source = AudioSource::ambient("ambient/wind.ogg");
        assert!(source.spatial);
        assert!(source.looping);
    }

    #[test]
    fn test_builder_pattern() {
        let source = AudioSource::new("test.ogg")
            .with_volume(0.5)
            .with_pitch(1.2)
            .with_looping(true)
            .with_autoplay();

        assert_eq!(source.volume, 0.5);
        assert_eq!(source.pitch, 1.2);
        assert!(source.looping);
        assert!(source.autoplay);
    }

    #[test]
    fn test_volume_clamping() {
        let source = AudioSource::new("test.ogg").with_volume(2.0);
        assert_eq!(source.volume, 1.0);

        let source = AudioSource::new("test.ogg").with_volume(-0.5);
        assert_eq!(source.volume, 0.0);
    }

    #[test]
    fn test_pitch_minimum() {
        let source = AudioSource::new("test.ogg").with_pitch(0.0);
        assert_eq!(source.pitch, 0.1);
    }

    #[test]
    fn test_playback_state_checks() {
        let mut source = AudioSource::new("test.ogg");
        assert!(source.is_stopped());
        assert!(!source.is_playing());
        assert!(!source.is_paused());

        source.state = PlaybackState::Playing;
        assert!(!source.is_stopped());
        assert!(source.is_playing());
        assert!(!source.is_paused());

        source.state = PlaybackState::Paused;
        assert!(!source.is_stopped());
        assert!(!source.is_playing());
        assert!(source.is_paused());
    }

    #[test]
    fn test_sound_handle() {
        let handle = SoundHandle::new(42);
        assert_eq!(handle.id(), 42);
    }
}
