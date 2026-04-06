//! Physics system for rigid body simulation and collision detection.
//!
//! Provides a simple physics world with rigid bodies, collision shapes,
//! and raycasting.

mod shapes;
mod rigidbody;
mod world;
pub mod raycast;

pub use shapes::{CollisionShape, Sphere, Box as BoxShape, Capsule};
pub use rigidbody::{RigidBody, RigidBodyType};
pub use world::{PhysicsWorld, PhysicsConfig};
pub use raycast::{Ray, RaycastHit};
