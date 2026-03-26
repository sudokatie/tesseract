/// Texture data (placeholder for now).
/// Full implementation requires wgpu device/queue.
#[derive(Clone, Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

/// Texture format.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureFormat {
    Rgba8,
    Rgba16Float,
    Depth32Float,
}

impl Texture {
    /// Create a placeholder texture.
    pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
        Self {
            width,
            height,
            format,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_creation() {
        let tex = Texture::new(512, 512, TextureFormat::Rgba8);
        assert_eq!(tex.width, 512);
        assert_eq!(tex.height, 512);
        assert_eq!(tex.format, TextureFormat::Rgba8);
    }
}
