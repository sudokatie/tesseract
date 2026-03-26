# Tesseract

A 3D game engine core built in Rust. Entity-Component-System architecture, PBR rendering, shadow mapping, skeletal animation.

## Why Another Engine?

Most game engines are either too minimal (no rendering) or too maximal (kitchen sink included). Tesseract sits in the middle: enough to build real 3D games, small enough to understand completely.

Built on wgpu for cross-platform GPU access. Uses hecs for a lightweight ECS that doesn't fight you. No magic, no macros everywhere, just clear Rust code.

## Features

- ECS architecture (hecs-based)
- Transform hierarchy with automatic propagation
- PBR materials (Cook-Torrance BRDF)
- Shadow mapping (cascaded for directional lights)
- Skeletal animation with blending
- glTF 2.0 asset loading
- Cross-platform (Windows, macOS, Linux, Web)

## Quick Start

```rust
use tesseract::prelude::*;

fn main() {
    let mut engine = Engine::new(WindowConfig::default());
    
    // Create a cube
    let mesh = engine.assets.insert(Mesh::cube());
    let material = engine.assets.insert(PbrMaterial::default());
    
    engine.world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Mesh(mesh),
        Material(material),
        Visibility::default(),
    ));
    
    // Add camera and light
    engine.world.spawn((
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO),
        Camera::perspective(45.0_f32.to_radians(), 0.1, 100.0),
    ));
    
    engine.world.spawn((
        Transform::default(),
        Light::directional(Vec3::ONE, 1.0),
    ));
    
    // Run (full implementation)
    // engine.run(|world, dt| { ... });
}
```

## Architecture

```
tesseract/
├── ecs/        # Entity-Component-System
├── render/     # PBR rendering, shadows, meshes
├── animation/  # Skeletal animation, blending
├── scene/      # Transform hierarchy, scene loading
├── asset/      # Asset management
└── math/       # Transform, AABB utilities
```

## Status

Work in progress. Core ECS, math, and animation modules complete. Rendering pipeline in development.

## License

MIT

---

Built by Katie.
