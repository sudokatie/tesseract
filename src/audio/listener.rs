//! Audio listener component representing the player's ears.

use glam::{Quat, Vec3};

/// The audio listener - typically attached to the camera or player entity.
/// Only one listener should be active at a time.
#[derive(Debug, Clone)]
pub struct AudioListener {
    /// Master volume for all audio (0.0 to 1.0).
    pub volume: f32,
    /// Whether this listener is currently active.
    pub active: bool,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            volume: 1.0,
            active: true,
        }
    }
}

impl AudioListener {
    /// Create a new audio listener with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a listener with a specific volume.
    pub fn with_volume(volume: f32) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            active: true,
        }
    }

    /// Set the master volume.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Calculate the forward and right vectors from a rotation.
    pub fn vectors_from_rotation(rotation: Quat) -> (Vec3, Vec3) {
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        (forward, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_listener() {
        let listener = AudioListener::default();
        assert_eq!(listener.volume, 1.0);
        assert!(listener.active);
    }

    #[test]
    fn test_with_volume() {
        let listener = AudioListener::with_volume(0.5);
        assert_eq!(listener.volume, 0.5);
    }

    #[test]
    fn test_volume_clamping() {
        let listener = AudioListener::with_volume(2.0);
        assert_eq!(listener.volume, 1.0);

        let listener = AudioListener::with_volume(-0.5);
        assert_eq!(listener.volume, 0.0);
    }

    #[test]
    fn test_set_volume() {
        let mut listener = AudioListener::new();
        listener.set_volume(0.75);
        assert_eq!(listener.volume, 0.75);
    }

    #[test]
    fn test_vectors_from_identity_rotation() {
        let (forward, right) = AudioListener::vectors_from_rotation(Quat::IDENTITY);
        assert!((forward - Vec3::NEG_Z).length() < 0.001);
        assert!((right - Vec3::X).length() < 0.001);
    }

    #[test]
    fn test_vectors_from_90_degree_rotation() {
        // Rotate 90 degrees around Y axis
        let rotation = Quat::from_rotation_y(PI / 2.0);
        let (forward, right) = AudioListener::vectors_from_rotation(rotation);
        // Forward should now point along -X
        assert!((forward - Vec3::NEG_X).length() < 0.001);
        // Right should now point along -Z
        assert!((right - Vec3::NEG_Z).length() < 0.001);
    }
}
