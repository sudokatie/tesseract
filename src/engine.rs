use crate::asset::AssetManager;
use crate::ecs::World;

/// Window configuration.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Tesseract".into(),
            width: 1280,
            height: 720,
            vsync: true,
        }
    }
}

/// The main engine struct.
/// Note: Full implementation requires wgpu/winit which need a window context.
/// This is a minimal implementation for testing.
pub struct Engine {
    pub world: World,
    pub assets: AssetManager,
    pub config: WindowConfig,
}

impl Engine {
    /// Create a new engine (without window for testing).
    pub fn new(config: WindowConfig) -> Self {
        Self {
            world: World::new(),
            assets: AssetManager::new(),
            config,
        }
    }

    /// Create with default config.
    pub fn default_config() -> Self {
        Self::new(WindowConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::default_config();
        assert!(engine.world.is_empty());
        assert_eq!(engine.config.width, 1280);
    }

    #[test]
    fn test_custom_config() {
        let config = WindowConfig {
            title: "Test".into(),
            width: 800,
            height: 600,
            vsync: false,
        };
        let engine = Engine::new(config);
        assert_eq!(engine.config.width, 800);
        assert_eq!(engine.config.title, "Test");
    }
}
