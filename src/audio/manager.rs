//! Audio manager for sound playback and mixing.

use std::collections::HashMap;
use std::path::Path;

use glam::Vec3;
use kira::manager::{AudioManager as KiraManager, AudioManagerSettings};
use kira::manager::backend::DefaultBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::Tween;
use kira::Volume;

use crate::audio::listener::AudioListener;
use crate::audio::source::{AudioSource, PlaybackState, SoundHandle};
use crate::audio::spatial::{calculate_attenuation, calculate_panning};

/// Audio channel for organizing sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannel {
    /// Master channel (affects all sounds).
    Master,
    /// Sound effects channel.
    Effects,
    /// Music channel.
    Music,
    /// Ambient/environmental sounds.
    Ambient,
    /// UI sounds.
    Ui,
}

/// Settings for an audio channel.
#[derive(Debug, Clone)]
pub struct ChannelSettings {
    /// Volume for this channel (0.0 to 1.0).
    pub volume: f32,
    /// Whether the channel is muted.
    pub muted: bool,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            muted: false,
        }
    }
}

/// Manages all audio playback for the engine.
pub struct AudioManager {
    manager: KiraManager,
    sounds: HashMap<u64, StaticSoundHandle>,
    next_handle_id: u64,
    channels: HashMap<AudioChannel, ChannelSettings>,
    listener_position: Vec3,
    listener_forward: Vec3,
    listener_right: Vec3,
    listener_volume: f32,
}

impl AudioManager {
    /// Create a new audio manager.
    pub fn new() -> Result<Self, AudioError> {
        let manager = KiraManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|e| AudioError::InitFailed(e.to_string()))?;

        let mut channels = HashMap::new();
        channels.insert(AudioChannel::Master, ChannelSettings::default());
        channels.insert(AudioChannel::Effects, ChannelSettings::default());
        channels.insert(AudioChannel::Music, ChannelSettings::default());
        channels.insert(AudioChannel::Ambient, ChannelSettings::default());
        channels.insert(AudioChannel::Ui, ChannelSettings::default());

        Ok(Self {
            manager,
            sounds: HashMap::new(),
            next_handle_id: 1,
            channels,
            listener_position: Vec3::ZERO,
            listener_forward: Vec3::NEG_Z,
            listener_right: Vec3::X,
            listener_volume: 1.0,
        })
    }

    /// Update the listener position and orientation.
    pub fn update_listener(
        &mut self,
        position: Vec3,
        listener: &AudioListener,
        rotation: glam::Quat,
    ) {
        self.listener_position = position;
        self.listener_volume = listener.volume;
        let (forward, right) = AudioListener::vectors_from_rotation(rotation);
        self.listener_forward = forward;
        self.listener_right = right;
    }

    /// Play a sound from an AudioSource component.
    pub fn play(
        &mut self,
        source: &mut AudioSource,
        position: Vec3,
    ) -> Result<SoundHandle, AudioError> {
        let path = Path::new(&source.sound_path);
        let sound_data = StaticSoundData::from_file(path)
            .map_err(|e| AudioError::LoadFailed(source.sound_path.clone(), e.to_string()))?;

        // Calculate spatial volume and panning
        let (volume, panning) = if source.spatial {
            let distance = (position - self.listener_position).length();
            let attenuation = calculate_attenuation(distance, &source.spatial_settings);
            let pan = calculate_panning(
                self.listener_position,
                self.listener_forward,
                self.listener_right,
                position,
            );
            (attenuation * source.volume, pan)
        } else {
            (source.volume, 0.0)
        };

        // Apply master and listener volume
        let final_volume = volume * self.listener_volume * self.get_master_volume();

        let mut settings = StaticSoundSettings::default()
            .volume(Volume::Amplitude(final_volume as f64))
            .panning(panning as f64);
        
        if source.looping {
            settings = settings.loop_region(..);
        }

        let handle = self.manager
            .play(sound_data.with_settings(settings))
            .map_err(|e| AudioError::PlayFailed(e.to_string()))?;

        let handle_id = self.next_handle_id;
        self.next_handle_id += 1;
        let sound_handle = SoundHandle::new(handle_id);
        
        self.sounds.insert(handle_id, handle);
        source.handle = Some(sound_handle);
        source.state = PlaybackState::Playing;

        Ok(sound_handle)
    }

    /// Stop a playing sound.
    pub fn stop(&mut self, source: &mut AudioSource) {
        if let Some(handle) = source.handle.take() {
            if let Some(mut sound) = self.sounds.remove(&handle.id()) {
                let _ = sound.stop(Tween::default());
            }
        }
        source.state = PlaybackState::Stopped;
    }

    /// Pause a playing sound.
    pub fn pause(&mut self, source: &mut AudioSource) {
        if let Some(handle) = &source.handle {
            if let Some(sound) = self.sounds.get_mut(&handle.id()) {
                let _ = sound.pause(Tween::default());
                source.state = PlaybackState::Paused;
            }
        }
    }

    /// Resume a paused sound.
    pub fn resume(&mut self, source: &mut AudioSource) {
        if let Some(handle) = &source.handle {
            if let Some(sound) = self.sounds.get_mut(&handle.id()) {
                let _ = sound.resume(Tween::default());
                source.state = PlaybackState::Playing;
            }
        }
    }

    /// Update spatial parameters for a playing sound.
    pub fn update_spatial(
        &mut self,
        source: &AudioSource,
        position: Vec3,
    ) {
        if !source.spatial {
            return;
        }

        // Pre-compute values before borrowing sounds mutably
        let distance = (position - self.listener_position).length();
        let attenuation = calculate_attenuation(distance, &source.spatial_settings);
        let pan = calculate_panning(
            self.listener_position,
            self.listener_forward,
            self.listener_right,
            position,
        );
        let master_volume = self.get_master_volume();
        let final_volume = attenuation * source.volume * self.listener_volume * master_volume;

        if let Some(handle) = &source.handle {
            if let Some(sound) = self.sounds.get_mut(&handle.id()) {
                sound.set_volume(Volume::Amplitude(final_volume as f64), Tween::default());
                sound.set_panning(pan as f64, Tween::default());
            }
        }
    }

    /// Set volume for a channel.
    pub fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) {
        if let Some(settings) = self.channels.get_mut(&channel) {
            settings.volume = volume.clamp(0.0, 1.0);
        }
    }

    /// Get volume for a channel.
    pub fn get_channel_volume(&self, channel: AudioChannel) -> f32 {
        self.channels.get(&channel).map(|s| s.volume).unwrap_or(1.0)
    }

    /// Mute a channel.
    pub fn mute_channel(&mut self, channel: AudioChannel, muted: bool) {
        if let Some(settings) = self.channels.get_mut(&channel) {
            settings.muted = muted;
        }
    }

    /// Check if a channel is muted.
    pub fn is_channel_muted(&self, channel: AudioChannel) -> bool {
        self.channels.get(&channel).map(|s| s.muted).unwrap_or(false)
    }

    /// Get the master volume (considering mute state).
    fn get_master_volume(&self) -> f32 {
        let settings = self.channels.get(&AudioChannel::Master).unwrap();
        if settings.muted { 0.0 } else { settings.volume }
    }

    /// Clean up finished sounds.
    pub fn cleanup(&mut self) {
        self.sounds.retain(|_, handle| {
            // Keep sounds that are still playing
            handle.state() != kira::sound::PlaybackState::Stopped
        });
    }
}

/// Audio system errors.
#[derive(Debug, Clone)]
pub enum AudioError {
    /// Failed to initialize audio system.
    InitFailed(String),
    /// Failed to load sound file.
    LoadFailed(String, String),
    /// Failed to play sound.
    PlayFailed(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitFailed(e) => write!(f, "Audio initialization failed: {}", e),
            Self::LoadFailed(path, e) => write!(f, "Failed to load '{}': {}", path, e),
            Self::PlayFailed(e) => write!(f, "Failed to play sound: {}", e),
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_settings_default() {
        let settings = ChannelSettings::default();
        assert_eq!(settings.volume, 1.0);
        assert!(!settings.muted);
    }

    #[test]
    fn test_audio_channel_variants() {
        let channels = [
            AudioChannel::Master,
            AudioChannel::Effects,
            AudioChannel::Music,
            AudioChannel::Ambient,
            AudioChannel::Ui,
        ];
        // Just verify they're all distinct
        for (i, c1) in channels.iter().enumerate() {
            for (j, c2) in channels.iter().enumerate() {
                if i != j {
                    assert_ne!(c1, c2);
                }
            }
        }
    }

    #[test]
    fn test_audio_error_display() {
        let err = AudioError::InitFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = AudioError::LoadFailed("test.ogg".to_string(), "not found".to_string());
        assert!(err.to_string().contains("test.ogg"));
        assert!(err.to_string().contains("not found"));

        let err = AudioError::PlayFailed("buffer full".to_string());
        assert!(err.to_string().contains("buffer full"));
    }
}
