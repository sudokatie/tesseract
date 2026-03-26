use glam::Vec3;

/// Type of light source.
#[derive(Clone, Copy, Debug)]
pub enum LightKind {
    Directional,
    Point { range: f32 },
    Spot { angle: f32, range: f32 },
    Ambient,
}

/// Light component.
#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub kind: LightKind,
    pub color: Vec3,
    pub intensity: f32,
}

impl Light {
    /// Create a directional light (like the sun).
    pub fn directional(color: Vec3, intensity: f32) -> Self {
        Self {
            kind: LightKind::Directional,
            color,
            intensity,
        }
    }

    /// Create a point light.
    pub fn point(color: Vec3, intensity: f32, range: f32) -> Self {
        Self {
            kind: LightKind::Point { range },
            color,
            intensity,
        }
    }

    /// Create a spot light.
    pub fn spot(color: Vec3, intensity: f32, angle: f32, range: f32) -> Self {
        Self {
            kind: LightKind::Spot { angle, range },
            color,
            intensity,
        }
    }

    /// Create an ambient light.
    pub fn ambient(color: Vec3, intensity: f32) -> Self {
        Self {
            kind: LightKind::Ambient,
            color,
            intensity,
        }
    }

    /// Check if this light casts shadows.
    pub fn casts_shadows(&self) -> bool {
        !matches!(self.kind, LightKind::Ambient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directional_light() {
        let light = Light::directional(Vec3::ONE, 1.0);
        assert!(matches!(light.kind, LightKind::Directional));
        assert!(light.casts_shadows());
    }

    #[test]
    fn test_point_light() {
        let light = Light::point(Vec3::ONE, 1.0, 10.0);
        match light.kind {
            LightKind::Point { range } => assert!((range - 10.0).abs() < 0.001),
            _ => panic!("expected point light"),
        }
    }

    #[test]
    fn test_spot_light() {
        let light = Light::spot(Vec3::ONE, 1.0, 0.5, 20.0);
        match light.kind {
            LightKind::Spot { angle, range } => {
                assert!((angle - 0.5).abs() < 0.001);
                assert!((range - 20.0).abs() < 0.001);
            }
            _ => panic!("expected spot light"),
        }
    }

    #[test]
    fn test_ambient_light() {
        let light = Light::ambient(Vec3::ONE, 0.5);
        assert!(matches!(light.kind, LightKind::Ambient));
        assert!(!light.casts_shadows());
    }
}
