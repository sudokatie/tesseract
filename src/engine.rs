use crate::asset::AssetManager;
use crate::ecs::{transform_propagation_system, World};

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
/// Manages ECS world, assets, and the game loop.
pub struct Engine {
    pub world: World,
    pub assets: AssetManager,
    pub config: WindowConfig,
}

impl Engine {
    /// Create a new engine with the given config.
    pub fn new(config: WindowConfig) -> Self {
        Self {
            world: World::new(),
            assets: AssetManager::new(),
            config,
        }
    }

    /// Create with default config.
    pub fn with_default_config() -> Self {
        Self::new(WindowConfig::default())
    }

    /// Run the game loop with the given update function.
    /// Note: Full windowed version requires wgpu initialization.
    /// This is a headless version for testing.
    pub fn run_headless<F>(&mut self, mut update: F, max_frames: usize)
    where
        F: FnMut(&mut World, &mut AssetManager, f32),
    {
        let dt = 1.0 / 60.0; // Fixed timestep
        for _ in 0..max_frames {
            update(&mut self.world, &mut self.assets, dt);
            transform_propagation_system(&mut self.world);
        }
    }

    /// Update one frame (useful for testing).
    pub fn update<F>(&mut self, mut update: F, dt: f32)
    where
        F: FnMut(&mut World, &mut AssetManager, f32),
    {
        update(&mut self.world, &mut self.assets, dt);
        transform_propagation_system(&mut self.world);
    }
}

/// Run the engine with a window.
/// This function takes ownership and runs the event loop.
#[cfg(feature = "window")]
pub fn run_windowed<F>(config: WindowConfig, mut setup: F)
where
    F: FnMut(&mut Engine) + 'static,
{
    use pollster::FutureExt;
    use winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
    };

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(winit::dpi::PhysicalSize::new(config.width, config.height))
        .build(&event_loop)
        .expect("Failed to create window");

    let mut engine = Engine::new(config);
    setup(&mut engine);

    let mut last_time = std::time::Instant::now();

    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    target.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_size),
                    ..
                } => {
                    // Handle resize
                }
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    transform_propagation_system(&mut engine.world);

                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Transform;
    use crate::render::Mesh;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::with_default_config();
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

    #[test]
    fn test_headless_run() {
        let mut engine = Engine::with_default_config();
        let cube = engine.assets.insert_mesh(Mesh::cube());

        engine.world.spawn((Transform::default(),));

        let mut frame_count = 0;
        engine.run_headless(
            |_world, _assets, _dt| {
                frame_count += 1;
            },
            10,
        );

        assert_eq!(frame_count, 10);
    }

    #[test]
    fn test_update() {
        let mut engine = Engine::with_default_config();
        let mut updated = false;

        engine.update(
            |_world, _assets, dt| {
                updated = true;
                assert!(dt > 0.0);
            },
            0.016,
        );

        assert!(updated);
    }
}
