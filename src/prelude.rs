//! Common imports for working with Tesseract.
//!
//! ```
//! use tesseract::prelude::*;
//! ```

pub use crate::animation::{AnimationClip, AnimationPlayer, LoopMode, Skeleton};
pub use crate::asset::{AssetManager, Handle};
pub use crate::ecs::{Children, GlobalTransform, Name, Parent, Visibility, World};
pub use crate::engine::{Engine, WindowConfig};
pub use crate::math::{Aabb, Transform};
pub use crate::physics::{
    CollisionShape, PhysicsConfig, PhysicsWorld, Ray, RaycastHit, RigidBody, RigidBodyType,
};
pub use crate::render::{
    Camera, CascadedShadowMap, Light, LightKind, Mesh, PbrMaterial, Projection, Renderer,
    RendererConfig, ShadowMapper, Texture,
};

pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
pub use hecs::Entity;
