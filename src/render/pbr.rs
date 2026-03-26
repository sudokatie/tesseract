use bytemuck::{Pod, Zeroable};

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

    /// Convert to a GPU uniform.
    pub fn to_uniform(&self) -> MaterialUniform {
        MaterialUniform {
            albedo: self.albedo,
            metallic_roughness: [self.metallic, self.roughness, 0.0, 0.0],
            emissive: [self.emissive[0], self.emissive[1], self.emissive[2], 0.0],
            flags: [
                self.albedo_texture.is_some() as u32,
                self.normal_texture.is_some() as u32,
                self.metallic_roughness_texture.is_some() as u32,
                self.ao_texture.is_some() as u32,
            ],
        }
    }
}

/// GPU-side material uniform data.
/// Aligned to 16 bytes for uniform buffer compatibility.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MaterialUniform {
    pub albedo: [f32; 4],
    pub metallic_roughness: [f32; 4], // metallic, roughness, _, _
    pub emissive: [f32; 4],
    pub flags: [u32; 4], // has_albedo_tex, has_normal_tex, has_mr_tex, has_ao_tex
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

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

    #[test]
    fn test_material_uniform_size() {
        // 4 * 4 floats + 4 uints = 64 bytes
        assert_eq!(size_of::<MaterialUniform>(), 64);
    }

    #[test]
    fn test_to_uniform() {
        let mat = PbrMaterial {
            albedo: [1.0, 0.5, 0.0, 1.0],
            metallic: 0.8,
            roughness: 0.2,
            emissive: [0.1, 0.2, 0.3],
            ..Default::default()
        };

        let uniform = mat.to_uniform();
        assert_eq!(uniform.albedo, [1.0, 0.5, 0.0, 1.0]);
        assert_eq!(uniform.metallic_roughness[0], 0.8);
        assert_eq!(uniform.metallic_roughness[1], 0.2);
        assert_eq!(uniform.emissive[0], 0.1);
        assert_eq!(uniform.flags, [0, 0, 0, 0]); // no textures
    }

    #[test]
    fn test_uniform_flags_with_textures() {
        let mat = PbrMaterial {
            albedo_texture: Some(crate::asset::Handle::new(1)),
            normal_texture: Some(crate::asset::Handle::new(2)),
            ..Default::default()
        };

        let uniform = mat.to_uniform();
        assert_eq!(uniform.flags[0], 1); // has albedo
        assert_eq!(uniform.flags[1], 1); // has normal
        assert_eq!(uniform.flags[2], 0); // no metallic_roughness
        assert_eq!(uniform.flags[3], 0); // no ao
    }
}
