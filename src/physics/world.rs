//! Physics world for simulation.

use glam::Vec3;
use super::{RigidBody, RigidBodyType, Ray, RaycastHit, raycast::raycast_shape};
use hecs::Entity;
use std::collections::HashMap;

/// Configuration for the physics world.
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// Gravity vector.
    pub gravity: Vec3,
    /// Fixed timestep for simulation (in seconds).
    pub timestep: f32,
    /// Maximum substeps per update.
    pub max_substeps: u32,
    /// Velocity threshold for sleeping bodies.
    pub sleep_threshold: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
            max_substeps: 4,
            sleep_threshold: 0.01,
        }
    }
}

/// Physics world managing rigid bodies.
pub struct PhysicsWorld {
    /// Configuration.
    pub config: PhysicsConfig,
    /// Accumulated time for fixed timestep.
    accumulator: f32,
}

impl PhysicsWorld {
    /// Create a new physics world with default configuration.
    pub fn new() -> Self {
        Self {
            config: PhysicsConfig::default(),
            accumulator: 0.0,
        }
    }

    /// Create a physics world with custom configuration.
    pub fn with_config(config: PhysicsConfig) -> Self {
        Self {
            config,
            accumulator: 0.0,
        }
    }

    /// Set gravity.
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.config.gravity = gravity;
    }

    /// Step the physics simulation.
    /// 
    /// Uses fixed timestep with accumulator for deterministic simulation.
    /// Positions are stored in the ECS Transform component.
    pub fn step(
        &mut self,
        dt: f32,
        bodies: &mut HashMap<Entity, (Vec3, RigidBody)>,
    ) {
        self.accumulator += dt;
        let mut substeps = 0;

        while self.accumulator >= self.config.timestep && substeps < self.config.max_substeps {
            self.integrate(self.config.timestep, bodies);
            self.accumulator -= self.config.timestep;
            substeps += 1;
        }
    }

    /// Integrate all bodies by one timestep.
    fn integrate(&self, dt: f32, bodies: &mut HashMap<Entity, (Vec3, RigidBody)>) {
        for (_, (position, body)) in bodies.iter_mut() {
            if body.sleeping || body.body_type != RigidBodyType::Dynamic {
                continue;
            }

            // Apply gravity
            if body.gravity_enabled {
                body.force += self.config.gravity * body.mass;
            }

            // Integrate velocity
            let acceleration = body.force * body.inv_mass;
            body.velocity += acceleration * dt;

            // Apply damping
            body.velocity *= 1.0 - body.linear_damping;
            body.angular_velocity *= 1.0 - body.angular_damping;

            // Integrate position
            *position += body.velocity * dt;

            // Clear forces for next frame
            body.clear_forces();

            // Check for sleep
            if body.velocity.length_squared() < self.config.sleep_threshold * self.config.sleep_threshold {
                body.sleep();
            }
        }
    }

    /// Raycast against all bodies.
    pub fn raycast(
        &self,
        ray: &Ray,
        max_distance: f32,
        bodies: &HashMap<Entity, (Vec3, RigidBody)>,
    ) -> Option<(Entity, RaycastHit)> {
        let mut closest: Option<(Entity, RaycastHit)> = None;
        let mut closest_dist = max_distance;

        for (entity, (position, body)) in bodies.iter() {
            if let Some(hit) = raycast_shape(ray, *position, &body.shape) {
                if hit.distance < closest_dist {
                    closest_dist = hit.distance;
                    closest = Some((*entity, hit));
                }
            }
        }

        closest
    }

    /// Raycast against all bodies, returning all hits.
    pub fn raycast_all(
        &self,
        ray: &Ray,
        max_distance: f32,
        bodies: &HashMap<Entity, (Vec3, RigidBody)>,
    ) -> Vec<(Entity, RaycastHit)> {
        let mut hits = Vec::new();

        for (entity, (position, body)) in bodies.iter() {
            if let Some(hit) = raycast_shape(ray, *position, &body.shape) {
                if hit.distance <= max_distance {
                    hits.push((*entity, hit));
                }
            }
        }

        // Sort by distance
        hits.sort_by(|a, b| a.1.distance.partial_cmp(&b.1.distance).unwrap());
        hits
    }

    /// Check overlap between two bodies.
    pub fn check_overlap(
        &self,
        pos_a: Vec3,
        body_a: &RigidBody,
        pos_b: Vec3,
        body_b: &RigidBody,
    ) -> bool {
        use crate::physics::{CollisionShape, Sphere, BoxShape};

        match (&body_a.shape, &body_b.shape) {
            (CollisionShape::Sphere(a), CollisionShape::Sphere(b)) => {
                a.intersects_sphere(pos_a, b, pos_b)
            }
            (CollisionShape::Box(a), CollisionShape::Box(b)) => {
                a.intersects_box(pos_a, b, pos_b)
            }
            (CollisionShape::Sphere(s), CollisionShape::Box(b)) |
            (CollisionShape::Box(b), CollisionShape::Sphere(s)) => {
                // Simplified sphere-box check
                let radius_sum = s.radius + b.half_extents.length();
                (pos_a - pos_b).length_squared() <= radius_sum * radius_sum
            }
            _ => {
                // Fallback to bounding sphere check
                let radius_a = body_a.shape.bounding_radius();
                let radius_b = body_b.shape.bounding_radius();
                let sum = radius_a + radius_b;
                (pos_a - pos_b).length_squared() <= sum * sum
            }
        }
    }

    /// Find all overlapping pairs.
    pub fn find_overlaps(
        &self,
        bodies: &HashMap<Entity, (Vec3, RigidBody)>,
    ) -> Vec<(Entity, Entity)> {
        let mut overlaps = Vec::new();
        let body_list: Vec<_> = bodies.iter().collect();

        for i in 0..body_list.len() {
            for j in (i + 1)..body_list.len() {
                let (entity_a, (pos_a, body_a)) = body_list[i];
                let (entity_b, (pos_b, body_b)) = body_list[j];

                if self.check_overlap(*pos_a, body_a, *pos_b, body_b) {
                    overlaps.push((*entity_a, *entity_b));
                }
            }
        }

        overlaps
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::CollisionShape;

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::new();
        assert_eq!(world.config.gravity, Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn test_set_gravity() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec3::new(0.0, -20.0, 0.0));
        assert_eq!(world.config.gravity, Vec3::new(0.0, -20.0, 0.0));
    }

    #[test]
    fn test_step_integration() {
        let mut world = PhysicsWorld::new();
        let mut bodies = HashMap::new();
        
        let entity = Entity::from_bits((1u64 << 32) | 1).unwrap();
        let body = RigidBody::new(CollisionShape::sphere(1.0), 1.0);
        bodies.insert(entity, (Vec3::ZERO, body));

        // Step simulation
        world.step(1.0 / 60.0, &mut bodies);

        // Body should have moved due to gravity
        let (pos, _) = bodies.get(&entity).unwrap();
        assert!(pos.y < 0.0);
    }

    #[test]
    fn test_static_body_no_move() {
        let mut world = PhysicsWorld::new();
        let mut bodies = HashMap::new();
        
        let entity = Entity::from_bits((1u64 << 32) | 1).unwrap();
        let body = RigidBody::new_static(CollisionShape::sphere(1.0));
        bodies.insert(entity, (Vec3::ZERO, body));

        world.step(1.0 / 60.0, &mut bodies);

        let (pos, _) = bodies.get(&entity).unwrap();
        assert_eq!(*pos, Vec3::ZERO);
    }

    #[test]
    fn test_raycast() {
        let world = PhysicsWorld::new();
        let mut bodies = HashMap::new();
        
        let entity = Entity::from_bits((1u64 << 32) | 1).unwrap();
        let body = RigidBody::new(CollisionShape::sphere(1.0), 1.0);
        bodies.insert(entity, (Vec3::new(5.0, 0.0, 0.0), body));

        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let hit = world.raycast(&ray, 100.0, &bodies);

        assert!(hit.is_some());
        let (hit_entity, hit_info) = hit.unwrap();
        assert_eq!(hit_entity, entity);
        assert!((hit_info.distance - 4.0).abs() < 0.1);
    }

    #[test]
    fn test_check_overlap() {
        let world = PhysicsWorld::new();
        
        let body_a = RigidBody::new(CollisionShape::sphere(1.0), 1.0);
        let body_b = RigidBody::new(CollisionShape::sphere(1.0), 1.0);

        // Overlapping
        assert!(world.check_overlap(
            Vec3::ZERO,
            &body_a,
            Vec3::new(1.0, 0.0, 0.0),
            &body_b
        ));

        // Not overlapping
        assert!(!world.check_overlap(
            Vec3::ZERO,
            &body_a,
            Vec3::new(5.0, 0.0, 0.0),
            &body_b
        ));
    }

    #[test]
    fn test_find_overlaps() {
        let world = PhysicsWorld::new();
        let mut bodies = HashMap::new();
        
        let entity1 = Entity::from_bits((1u64 << 32) | 1).unwrap();
        let entity2 = Entity::from_bits((1u64 << 32) | 2).unwrap();
        let entity3 = Entity::from_bits((1u64 << 32) | 3).unwrap();

        let body = RigidBody::new(CollisionShape::sphere(1.0), 1.0);
        
        bodies.insert(entity1, (Vec3::ZERO, body.clone()));
        bodies.insert(entity2, (Vec3::new(1.0, 0.0, 0.0), body.clone())); // Overlaps with entity1
        bodies.insert(entity3, (Vec3::new(10.0, 0.0, 0.0), body)); // Doesn't overlap

        let overlaps = world.find_overlaps(&bodies);
        assert_eq!(overlaps.len(), 1);
    }
}
