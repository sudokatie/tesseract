# Tesseract

A 3D game engine core built in Rust. ECS architecture, PBR rendering, skeletal animation, glTF loading.

## Why Another Engine?

Most game engines are either too minimal (no rendering) or too maximal (kitchen sink included). Tesseract sits in the middle: enough to build real 3D games, small enough to understand completely.

Built on wgpu for cross-platform GPU access. Uses hecs for a lightweight ECS that doesn't fight you. No magic, no macros everywhere, just clear Rust code.

## Features

- Entity-Component-System (hecs-based, lightweight)
- Transform hierarchy with automatic propagation
- PBR materials (Cook-Torrance BRDF, MaterialUniform for GPU)
- Camera types (perspective, orthographic)
- Light types (directional, point, spot, ambient)
- Skeletal animation with clips, players, and blending
- glTF 2.0 loading (meshes, materials, skeletons, animations)
- Scene graph utilities
- Cross-platform (Windows, macOS, Linux)

## Quick Start

```rust
use tesseract::prelude::*;

fn main() {
    let mut engine = Engine::with_default_config();
    
    // Load assets
    let mesh = engine.assets.insert_mesh(Mesh::cube());
    let material = engine.assets.insert_material(PbrMaterial::default());
    
    // Create a cube
    let cube = engine.world.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
    ));
    
    // Add camera
    engine.world.spawn((
        Transform::from_xyz(0.0, 3.0, 8.0).looking_at(Vec3::ZERO),
        Camera::perspective(45.0_f32.to_radians(), 0.1, 100.0),
    ));
    
    // Add light
    engine.world.spawn((
        Transform::from_xyz(5.0, 10.0, 5.0),
        Light::directional(Vec3::ONE, 1.0),
    ));
    
    // Run game loop (headless for testing)
    engine.run_headless(|world, assets, dt| {
        // Rotate the cube
        if let Some(mut t) = world.get_mut::<Transform>(cube) {
            t.rotation *= Quat::from_rotation_y(dt);
        }
    }, 60);
}
```

## Loading glTF Models

```rust
use tesseract::asset::{load_gltf, AssetManager};

let mut assets = AssetManager::new();
let scene = load_gltf("model.gltf", &mut assets)?;

println!("Loaded {} meshes", scene.meshes.len());
println!("Loaded {} materials", scene.materials.len());
println!("Loaded {} animations", scene.animations.len());
```

## Scene Graph

```rust
use tesseract::scene::SceneGraph;

// Build hierarchy
SceneGraph::add_child(&mut world, parent, child);
SceneGraph::add_child(&mut world, child, grandchild);

// Query hierarchy
let ancestors = SceneGraph::get_ancestors(&world, grandchild);
let descendants = SceneGraph::get_descendants(&world, parent);
let root = SceneGraph::get_root(&world, grandchild);

// Clean up
SceneGraph::despawn_recursive(&mut world, parent); // Removes parent and all descendants
```

## Animation

```rust
use tesseract::animation::{AnimationPlayer, LoopMode};

// Create player for a loaded clip
let mut player = AnimationPlayer::new(clip_handle);
player.loop_mode = LoopMode::Loop;
player.speed = 1.5;
player.play();

// Update each frame
player.update(dt, clip.duration);
```

## Architecture

```
tesseract/
├── ecs/        # Entity-Component-System (hecs wrapper)
├── render/     # PBR materials, cameras, lights, meshes
├── animation/  # Skeletal animation, clips, blending
├── scene/      # Transform hierarchy, scene graph utilities
├── asset/      # Asset management, glTF loading
├── math/       # Transform, AABB utilities
└── engine.rs   # Main engine loop
```

## Running Examples

```bash
cargo run --example basic
```

## Status

Core engine complete. Rendering pipeline (wgpu integration) in progress.

**Completed:**
- ECS with transform propagation
- Math utilities (Transform, AABB)
- PBR material system
- Animation system (skeleton, clips, player, blending)
- Asset manager with typed storage
- glTF 2.0 loader
- Scene graph utilities
- Headless engine loop

**Also Implemented:**
- Render pipeline (wgpu)
- WGSL shaders (Cook-Torrance BRDF, shadow)
- Cascaded shadow mapping

## License

MIT

---

Built by Katie.
