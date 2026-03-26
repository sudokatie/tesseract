use crate::asset::Handle;
use crate::render::Texture;

/// Physically-based rendering material.
#[derive(Clone, Debug)]
pub struct PbrMaterial {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub albedo_texture: Option<Handle<Texture>>,
    pub normal_texture: Option<Handle<Texture>>,
    pub metallic_roughness_texture: Option<Handle<Texture>>,
    pub ao_texture: Option<Handle<Texture>>,
    pub emissive: [f32; 3],
    pub emissive_texture: Option<Handle<Texture>>,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            ao_texture: None,
            emissive: [0.0, 0.0, 0.0],
            emissive_texture: None,
        }
    }
}

impl PbrMaterial {
    /// Create a material with the given albedo color.
    pub fn with_albedo(color: [f32; 4]) -> Self {
        Self {
            albedo: color,
            ..Default::default()
        }
    }

    /// Create a metallic material.
    pub fn metallic(metallic: f32, roughness: f32) -> Self {
        Self {
            metallic,
            roughness,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_material() {
        let mat = PbrMaterial::default();
        assert_eq!(mat.albedo, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mat.metallic, 0.0);
        assert_eq!(mat.roughness, 0.5);
    }

    #[test]
    fn test_with_albedo() {
        let mat = PbrMaterial::with_albedo([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(mat.albedo, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_metallic() {
        let mat = PbrMaterial::metallic(1.0, 0.2);
        assert_eq!(mat.metallic, 1.0);
        assert_eq!(mat.roughness, 0.2);
    }
}
