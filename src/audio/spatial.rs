//! Spatial audio calculations for 3D sound positioning.

use glam::Vec3;

/// Distance attenuation model for spatial audio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttenuationModel {
    /// No distance attenuation (constant volume).
    None,
    /// Linear falloff between ref_distance and max_distance.
    Linear,
    /// Inverse distance attenuation (realistic).
    Inverse,
    /// Inverse distance squared (more aggressive falloff).
    InverseSquared,
}

impl Default for AttenuationModel {
    fn default() -> Self {
        Self::Inverse
    }
}

/// Settings for spatial audio behavior.
#[derive(Debug, Clone)]
pub struct SpatialSettings {
    /// Distance at which volume is 100%.
    pub ref_distance: f32,
    /// Maximum distance for attenuation calculations.
    pub max_distance: f32,
    /// Rolloff factor (higher = faster falloff).
    pub rolloff: f32,
    /// Attenuation model to use.
    pub attenuation: AttenuationModel,
    /// Minimum volume (0.0 to 1.0).
    pub min_volume: f32,
}

impl Default for SpatialSettings {
    fn default() -> Self {
        Self {
            ref_distance: 1.0,
            max_distance: 100.0,
            rolloff: 1.0,
            attenuation: AttenuationModel::Inverse,
            min_volume: 0.0,
        }
    }
}

impl SpatialSettings {
    /// Create settings for a small/close sound (footsteps, UI).
    pub fn close() -> Self {
        Self {
            ref_distance: 0.5,
            max_distance: 20.0,
            rolloff: 1.5,
            ..Default::default()
        }
    }

    /// Create settings for a medium-range sound (voices, impacts).
    pub fn medium() -> Self {
        Self::default()
    }

    /// Create settings for a distant/ambient sound (thunder, explosions).
    pub fn distant() -> Self {
        Self {
            ref_distance: 5.0,
            max_distance: 500.0,
            rolloff: 0.5,
            ..Default::default()
        }
    }
}

/// Calculate volume attenuation based on distance.
pub fn calculate_attenuation(
    distance: f32,
    settings: &SpatialSettings,
) -> f32 {
    let clamped_distance = distance.clamp(settings.ref_distance, settings.max_distance);
    
    let gain = match settings.attenuation {
        AttenuationModel::None => 1.0,
        AttenuationModel::Linear => {
            let range = settings.max_distance - settings.ref_distance;
            if range <= 0.0 {
                1.0
            } else {
                1.0 - settings.rolloff * (clamped_distance - settings.ref_distance) / range
            }
        }
        AttenuationModel::Inverse => {
            settings.ref_distance
                / (settings.ref_distance
                    + settings.rolloff * (clamped_distance - settings.ref_distance))
        }
        AttenuationModel::InverseSquared => {
            let ratio = settings.ref_distance / clamped_distance;
            ratio * ratio
        }
    };

    gain.clamp(settings.min_volume, 1.0)
}

/// Calculate stereo panning based on listener and source positions.
/// Returns a value from -1.0 (full left) to 1.0 (full right).
pub fn calculate_panning(
    listener_pos: Vec3,
    _listener_forward: Vec3,
    listener_right: Vec3,
    source_pos: Vec3,
) -> f32 {
    let to_source = (source_pos - listener_pos).normalize_or_zero();
    
    // Project onto listener's horizontal plane
    let right_component = to_source.dot(listener_right);
    
    // Clamp to valid range
    right_component.clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attenuation_none() {
        let settings = SpatialSettings {
            attenuation: AttenuationModel::None,
            ..Default::default()
        };
        assert_eq!(calculate_attenuation(50.0, &settings), 1.0);
        assert_eq!(calculate_attenuation(100.0, &settings), 1.0);
    }

    #[test]
    fn test_attenuation_linear() {
        let settings = SpatialSettings {
            attenuation: AttenuationModel::Linear,
            ref_distance: 1.0,
            max_distance: 11.0,
            rolloff: 1.0,
            min_volume: 0.0,
        };
        // At ref_distance, should be 1.0
        assert!((calculate_attenuation(1.0, &settings) - 1.0).abs() < 0.001);
        // At max_distance, should be 0.0
        assert!((calculate_attenuation(11.0, &settings) - 0.0).abs() < 0.001);
        // Midpoint should be 0.5
        assert!((calculate_attenuation(6.0, &settings) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_attenuation_inverse() {
        let settings = SpatialSettings {
            attenuation: AttenuationModel::Inverse,
            ref_distance: 1.0,
            max_distance: 100.0,
            rolloff: 1.0,
            min_volume: 0.0,
        };
        // At ref_distance, should be 1.0
        assert!((calculate_attenuation(1.0, &settings) - 1.0).abs() < 0.001);
        // At 2x ref_distance, should be 0.5
        assert!((calculate_attenuation(2.0, &settings) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_attenuation_inverse_squared() {
        let settings = SpatialSettings {
            attenuation: AttenuationModel::InverseSquared,
            ref_distance: 1.0,
            max_distance: 100.0,
            rolloff: 1.0,
            min_volume: 0.0,
        };
        // At ref_distance, should be 1.0
        assert!((calculate_attenuation(1.0, &settings) - 1.0).abs() < 0.001);
        // At 2x ref_distance, should be 0.25
        assert!((calculate_attenuation(2.0, &settings) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_attenuation_respects_min_volume() {
        let settings = SpatialSettings {
            attenuation: AttenuationModel::Linear,
            ref_distance: 1.0,
            max_distance: 10.0,
            rolloff: 1.0,
            min_volume: 0.2,
        };
        // Beyond max should clamp to min_volume
        assert!((calculate_attenuation(100.0, &settings) - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_panning_center() {
        let listener_pos = Vec3::ZERO;
        let listener_forward = Vec3::NEG_Z;
        let listener_right = Vec3::X;
        let source_pos = Vec3::new(0.0, 0.0, -5.0); // Directly ahead
        
        let pan = calculate_panning(listener_pos, listener_forward, listener_right, source_pos);
        assert!(pan.abs() < 0.001);
    }

    #[test]
    fn test_panning_right() {
        let listener_pos = Vec3::ZERO;
        let listener_forward = Vec3::NEG_Z;
        let listener_right = Vec3::X;
        let source_pos = Vec3::new(5.0, 0.0, 0.0); // To the right
        
        let pan = calculate_panning(listener_pos, listener_forward, listener_right, source_pos);
        assert!((pan - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_panning_left() {
        let listener_pos = Vec3::ZERO;
        let listener_forward = Vec3::NEG_Z;
        let listener_right = Vec3::X;
        let source_pos = Vec3::new(-5.0, 0.0, 0.0); // To the left
        
        let pan = calculate_panning(listener_pos, listener_forward, listener_right, source_pos);
        assert!((pan - (-1.0)).abs() < 0.001);
    }
}
