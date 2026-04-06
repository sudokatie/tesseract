//! Raycasting for physics queries.

use glam::Vec3;
use super::shapes::{CollisionShape, Sphere, Box as BoxShape};

/// A ray for raycasting.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origin of the ray.
    pub origin: Vec3,
    /// Direction of the ray (should be normalized).
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at distance t.
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Result of a raycast hit.
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// Distance along the ray to the hit point.
    pub distance: f32,
    /// World position of the hit.
    pub point: Vec3,
    /// Surface normal at the hit point.
    pub normal: Vec3,
}

/// Raycast against a sphere.
pub fn raycast_sphere(ray: &Ray, center: Vec3, sphere: &Sphere) -> Option<RaycastHit> {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);

    // Find the nearest positive t
    let t = if t1 >= 0.0 {
        t1
    } else if t2 >= 0.0 {
        t2
    } else {
        return None;
    };

    let point = ray.point_at(t);
    let normal = (point - center).normalize();

    Some(RaycastHit {
        distance: t,
        point,
        normal,
    })
}

/// Raycast against an axis-aligned box.
pub fn raycast_box(ray: &Ray, center: Vec3, box_shape: &BoxShape) -> Option<RaycastHit> {
    let min = center - box_shape.half_extents;
    let max = center + box_shape.half_extents;

    let inv_dir = Vec3::new(
        1.0 / ray.direction.x,
        1.0 / ray.direction.y,
        1.0 / ray.direction.z,
    );

    let t1 = (min.x - ray.origin.x) * inv_dir.x;
    let t2 = (max.x - ray.origin.x) * inv_dir.x;
    let t3 = (min.y - ray.origin.y) * inv_dir.y;
    let t4 = (max.y - ray.origin.y) * inv_dir.y;
    let t5 = (min.z - ray.origin.z) * inv_dir.z;
    let t6 = (max.z - ray.origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        return None;
    }

    let t = if tmin >= 0.0 { tmin } else { tmax };
    let point = ray.point_at(t);

    // Calculate normal (which face was hit)
    let epsilon = 0.0001;
    let normal = if (point.x - min.x).abs() < epsilon {
        Vec3::new(-1.0, 0.0, 0.0)
    } else if (point.x - max.x).abs() < epsilon {
        Vec3::new(1.0, 0.0, 0.0)
    } else if (point.y - min.y).abs() < epsilon {
        Vec3::new(0.0, -1.0, 0.0)
    } else if (point.y - max.y).abs() < epsilon {
        Vec3::new(0.0, 1.0, 0.0)
    } else if (point.z - min.z).abs() < epsilon {
        Vec3::new(0.0, 0.0, -1.0)
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };

    Some(RaycastHit {
        distance: t,
        point,
        normal,
    })
}

/// Raycast against a collision shape.
pub fn raycast_shape(ray: &Ray, center: Vec3, shape: &CollisionShape) -> Option<RaycastHit> {
    match shape {
        CollisionShape::Sphere(s) => raycast_sphere(ray, center, s),
        CollisionShape::Box(b) => raycast_box(ray, center, b),
        CollisionShape::Capsule(_) => {
            // Capsule raycasting is complex - approximate with sphere for now
            let radius = shape.bounding_radius();
            raycast_sphere(ray, center, &Sphere::new(radius))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_creation() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(ray.origin, Vec3::ZERO);
        assert!((ray.direction.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_ray_point_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let point = ray.point_at(5.0);
        assert_eq!(point, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_raycast_sphere_hit() {
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let sphere = Sphere::new(1.0);
        let center = Vec3::ZERO;

        let hit = raycast_sphere(&ray, center, &sphere);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert!((hit.distance - 4.0).abs() < 0.001);
        assert!((hit.point.x + 1.0).abs() < 0.001);
        assert!((hit.normal.x + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_raycast_sphere_miss() {
        let ray = Ray::new(Vec3::new(-5.0, 5.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let sphere = Sphere::new(1.0);
        let center = Vec3::ZERO;

        let hit = raycast_sphere(&ray, center, &sphere);
        assert!(hit.is_none());
    }

    #[test]
    fn test_raycast_sphere_inside() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let sphere = Sphere::new(2.0);
        let center = Vec3::ZERO;

        let hit = raycast_sphere(&ray, center, &sphere);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert!((hit.distance - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_raycast_box_hit() {
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let box_shape = BoxShape::new(Vec3::ONE);
        let center = Vec3::ZERO;

        let hit = raycast_box(&ray, center, &box_shape);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert!((hit.distance - 4.0).abs() < 0.001);
        assert!((hit.point.x + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_raycast_box_miss() {
        let ray = Ray::new(Vec3::new(-5.0, 5.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let box_shape = BoxShape::new(Vec3::ONE);
        let center = Vec3::ZERO;

        let hit = raycast_box(&ray, center, &box_shape);
        assert!(hit.is_none());
    }

    #[test]
    fn test_raycast_shape() {
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let shape = CollisionShape::sphere(1.0);
        let center = Vec3::ZERO;

        let hit = raycast_shape(&ray, center, &shape);
        assert!(hit.is_some());
    }
}
