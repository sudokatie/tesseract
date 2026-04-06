//! Collision shapes for physics simulation.

use glam::{Vec3, Mat4};

/// A collision shape for physics simulation.
#[derive(Debug, Clone)]
pub enum CollisionShape {
    Sphere(Sphere),
    Box(Box),
    Capsule(Capsule),
}

impl CollisionShape {
    /// Create a sphere collision shape.
    pub fn sphere(radius: f32) -> Self {
        CollisionShape::Sphere(Sphere { radius })
    }

    /// Create a box collision shape.
    pub fn box_shape(half_extents: Vec3) -> Self {
        CollisionShape::Box(Box { half_extents })
    }

    /// Create a capsule collision shape.
    pub fn capsule(radius: f32, height: f32) -> Self {
        CollisionShape::Capsule(Capsule { radius, height })
    }

    /// Get the bounding sphere radius for broad phase collision.
    pub fn bounding_radius(&self) -> f32 {
        match self {
            CollisionShape::Sphere(s) => s.radius,
            CollisionShape::Box(b) => b.half_extents.length(),
            CollisionShape::Capsule(c) => c.radius + c.height / 2.0,
        }
    }

    /// Compute the axis-aligned bounding box (AABB) for this shape.
    pub fn aabb(&self, transform: &Mat4) -> (Vec3, Vec3) {
        // Extract translation from transform matrix
        let center = transform.col(3).truncate();
        let radius = self.bounding_radius();
        let extent = Vec3::new(radius, radius, radius);
        (center - extent, center + extent)
    }
}

/// Sphere collision shape.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    /// Check if a point is inside the sphere.
    pub fn contains_point(&self, center: Vec3, point: Vec3) -> bool {
        (point - center).length_squared() <= self.radius * self.radius
    }

    /// Check collision with another sphere.
    pub fn intersects_sphere(&self, center_a: Vec3, other: &Sphere, center_b: Vec3) -> bool {
        let dist_sq = (center_b - center_a).length_squared();
        let sum_radii = self.radius + other.radius;
        dist_sq <= sum_radii * sum_radii
    }
}

/// Box (AABB) collision shape.
#[derive(Debug, Clone, Copy)]
pub struct Box {
    pub half_extents: Vec3,
}

impl Box {
    pub fn new(half_extents: Vec3) -> Self {
        Self { half_extents }
    }

    /// Create a unit cube.
    pub fn unit() -> Self {
        Self { half_extents: Vec3::ONE * 0.5 }
    }

    /// Check if a point is inside the box.
    pub fn contains_point(&self, center: Vec3, point: Vec3) -> bool {
        let local = point - center;
        local.x.abs() <= self.half_extents.x
            && local.y.abs() <= self.half_extents.y
            && local.z.abs() <= self.half_extents.z
    }

    /// Check collision with another box (AABB test).
    pub fn intersects_box(&self, center_a: Vec3, other: &Box, center_b: Vec3) -> bool {
        let min_a = center_a - self.half_extents;
        let max_a = center_a + self.half_extents;
        let min_b = center_b - other.half_extents;
        let max_b = center_b + other.half_extents;

        min_a.x <= max_b.x && max_a.x >= min_b.x
            && min_a.y <= max_b.y && max_a.y >= min_b.y
            && min_a.z <= max_b.z && max_a.z >= min_b.z
    }
}

/// Capsule collision shape (cylinder with hemispherical caps).
#[derive(Debug, Clone, Copy)]
pub struct Capsule {
    pub radius: f32,
    pub height: f32, // Height of the cylindrical portion
}

impl Capsule {
    pub fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }

    /// Total height including caps.
    pub fn total_height(&self) -> f32 {
        self.height + 2.0 * self.radius
    }

    /// Check if a point is inside the capsule (oriented along Y axis).
    pub fn contains_point(&self, center: Vec3, point: Vec3) -> bool {
        let local = point - center;
        let half_height = self.height / 2.0;

        // Clamp Y to the line segment
        let y_clamped = local.y.clamp(-half_height, half_height);
        let closest = Vec3::new(0.0, y_clamped, 0.0);
        
        let dist_sq = (local - closest).length_squared();
        dist_sq <= self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_contains_point() {
        let sphere = Sphere::new(1.0);
        let center = Vec3::ZERO;

        assert!(sphere.contains_point(center, Vec3::ZERO));
        assert!(sphere.contains_point(center, Vec3::new(0.5, 0.0, 0.0)));
        assert!(!sphere.contains_point(center, Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere_intersection() {
        let s1 = Sphere::new(1.0);
        let s2 = Sphere::new(1.0);

        // Overlapping
        assert!(s1.intersects_sphere(Vec3::ZERO, &s2, Vec3::new(1.0, 0.0, 0.0)));
        
        // Just touching
        assert!(s1.intersects_sphere(Vec3::ZERO, &s2, Vec3::new(2.0, 0.0, 0.0)));
        
        // Separated
        assert!(!s1.intersects_sphere(Vec3::ZERO, &s2, Vec3::new(3.0, 0.0, 0.0)));
    }

    #[test]
    fn test_box_contains_point() {
        let box_shape = Box::new(Vec3::ONE);
        let center = Vec3::ZERO;

        assert!(box_shape.contains_point(center, Vec3::ZERO));
        assert!(box_shape.contains_point(center, Vec3::new(0.5, 0.5, 0.5)));
        assert!(!box_shape.contains_point(center, Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_box_intersection() {
        let b1 = Box::new(Vec3::ONE);
        let b2 = Box::new(Vec3::ONE);

        // Overlapping
        assert!(b1.intersects_box(Vec3::ZERO, &b2, Vec3::new(1.0, 0.0, 0.0)));
        
        // Just touching
        assert!(b1.intersects_box(Vec3::ZERO, &b2, Vec3::new(2.0, 0.0, 0.0)));
        
        // Separated
        assert!(!b1.intersects_box(Vec3::ZERO, &b2, Vec3::new(3.0, 0.0, 0.0)));
    }

    #[test]
    fn test_capsule_contains_point() {
        let capsule = Capsule::new(0.5, 2.0);
        let center = Vec3::ZERO;

        assert!(capsule.contains_point(center, Vec3::ZERO));
        assert!(capsule.contains_point(center, Vec3::new(0.0, 1.0, 0.0)));
        assert!(!capsule.contains_point(center, Vec3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn test_bounding_radius() {
        let sphere = CollisionShape::sphere(2.0);
        assert_eq!(sphere.bounding_radius(), 2.0);

        let box_shape = CollisionShape::box_shape(Vec3::new(1.0, 1.0, 1.0));
        assert!((box_shape.bounding_radius() - 3.0_f32.sqrt()).abs() < 0.001);
    }
}
