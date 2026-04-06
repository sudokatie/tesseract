//! Rigid body component for physics simulation.

use glam::Vec3;
use super::CollisionShape;

/// Type of rigid body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    /// Static body - doesn't move, infinite mass.
    Static,
    /// Kinematic body - controlled by code, not physics.
    Kinematic,
    /// Dynamic body - affected by forces and collisions.
    Dynamic,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        RigidBodyType::Dynamic
    }
}

/// A rigid body for physics simulation.
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// Type of rigid body.
    pub body_type: RigidBodyType,
    /// Collision shape.
    pub shape: CollisionShape,
    /// Mass (ignored for static/kinematic bodies).
    pub mass: f32,
    /// Inverse mass (0 for static bodies).
    pub inv_mass: f32,
    /// Linear velocity.
    pub velocity: Vec3,
    /// Angular velocity.
    pub angular_velocity: Vec3,
    /// Accumulated force (reset each frame).
    pub force: Vec3,
    /// Accumulated torque (reset each frame).
    pub torque: Vec3,
    /// Linear damping (0-1).
    pub linear_damping: f32,
    /// Angular damping (0-1).
    pub angular_damping: f32,
    /// Coefficient of restitution (bounciness).
    pub restitution: f32,
    /// Coefficient of friction.
    pub friction: f32,
    /// Whether gravity affects this body.
    pub gravity_enabled: bool,
    /// Whether the body is currently sleeping (inactive).
    pub sleeping: bool,
}

impl RigidBody {
    /// Create a new dynamic rigid body.
    pub fn new(shape: CollisionShape, mass: f32) -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            shape,
            mass,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            linear_damping: 0.01,
            angular_damping: 0.01,
            restitution: 0.3,
            friction: 0.5,
            gravity_enabled: true,
            sleeping: false,
        }
    }

    /// Create a static rigid body.
    pub fn new_static(shape: CollisionShape) -> Self {
        Self {
            body_type: RigidBodyType::Static,
            shape,
            mass: f32::INFINITY,
            inv_mass: 0.0,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            restitution: 0.3,
            friction: 0.5,
            gravity_enabled: false,
            sleeping: false,
        }
    }

    /// Create a kinematic rigid body.
    pub fn new_kinematic(shape: CollisionShape) -> Self {
        Self {
            body_type: RigidBodyType::Kinematic,
            shape,
            mass: f32::INFINITY,
            inv_mass: 0.0,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            restitution: 0.3,
            friction: 0.5,
            gravity_enabled: false,
            sleeping: false,
        }
    }

    /// Set the body type.
    pub fn with_type(mut self, body_type: RigidBodyType) -> Self {
        self.body_type = body_type;
        if body_type == RigidBodyType::Static || body_type == RigidBodyType::Kinematic {
            self.inv_mass = 0.0;
        }
        self
    }

    /// Set restitution (bounciness).
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// Set friction.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    /// Set damping.
    pub fn with_damping(mut self, linear: f32, angular: f32) -> Self {
        self.linear_damping = linear.clamp(0.0, 1.0);
        self.angular_damping = angular.clamp(0.0, 1.0);
        self
    }

    /// Enable or disable gravity.
    pub fn with_gravity(mut self, enabled: bool) -> Self {
        self.gravity_enabled = enabled;
        self
    }

    /// Apply a force at the center of mass.
    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.force += force;
        }
    }

    /// Apply an impulse at the center of mass.
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.velocity += impulse * self.inv_mass;
        }
    }

    /// Apply torque.
    pub fn apply_torque(&mut self, torque: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.torque += torque;
        }
    }

    /// Apply angular impulse.
    pub fn apply_angular_impulse(&mut self, impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            // Simplified - assumes uniform density sphere for inertia
            self.angular_velocity += impulse * self.inv_mass;
        }
    }

    /// Clear accumulated forces.
    pub fn clear_forces(&mut self) {
        self.force = Vec3::ZERO;
        self.torque = Vec3::ZERO;
    }

    /// Check if this is a dynamic body.
    pub fn is_dynamic(&self) -> bool {
        self.body_type == RigidBodyType::Dynamic
    }

    /// Check if this is a static body.
    pub fn is_static(&self) -> bool {
        self.body_type == RigidBodyType::Static
    }

    /// Check if this is a kinematic body.
    pub fn is_kinematic(&self) -> bool {
        self.body_type == RigidBodyType::Kinematic
    }

    /// Wake the body from sleep.
    pub fn wake(&mut self) {
        self.sleeping = false;
    }

    /// Put the body to sleep.
    pub fn sleep(&mut self) {
        self.sleeping = true;
        self.velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_body() {
        let body = RigidBody::new(CollisionShape::sphere(1.0), 10.0);
        assert!(body.is_dynamic());
        assert_eq!(body.inv_mass, 0.1);
        assert!(body.gravity_enabled);
    }

    #[test]
    fn test_static_body() {
        let body = RigidBody::new_static(CollisionShape::sphere(1.0));
        assert!(body.is_static());
        assert_eq!(body.inv_mass, 0.0);
        assert!(!body.gravity_enabled);
    }

    #[test]
    fn test_kinematic_body() {
        let body = RigidBody::new_kinematic(CollisionShape::sphere(1.0));
        assert!(body.is_kinematic());
        assert_eq!(body.inv_mass, 0.0);
    }

    #[test]
    fn test_apply_force() {
        let mut body = RigidBody::new(CollisionShape::sphere(1.0), 10.0);
        body.apply_force(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(body.force, Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_apply_impulse() {
        let mut body = RigidBody::new(CollisionShape::sphere(1.0), 10.0);
        body.apply_impulse(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(body.velocity, Vec3::new(1.0, 0.0, 0.0)); // 10 * 0.1 = 1
    }

    #[test]
    fn test_clear_forces() {
        let mut body = RigidBody::new(CollisionShape::sphere(1.0), 10.0);
        body.apply_force(Vec3::new(10.0, 20.0, 30.0));
        body.apply_torque(Vec3::new(1.0, 2.0, 3.0));
        body.clear_forces();
        assert_eq!(body.force, Vec3::ZERO);
        assert_eq!(body.torque, Vec3::ZERO);
    }

    #[test]
    fn test_sleep_wake() {
        let mut body = RigidBody::new(CollisionShape::sphere(1.0), 10.0);
        body.velocity = Vec3::new(1.0, 2.0, 3.0);
        
        body.sleep();
        assert!(body.sleeping);
        assert_eq!(body.velocity, Vec3::ZERO);
        
        body.wake();
        assert!(!body.sleeping);
    }
}
