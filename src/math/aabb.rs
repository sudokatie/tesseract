use glam::{Mat4, Vec3};

/// Axis-aligned bounding box.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }
}

impl Aabb {
    /// Create an AABB from min and max points.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB that encompasses all given points.
    pub fn from_points(points: &[Vec3]) -> Self {
        if points.is_empty() {
            return Self::default();
        }

        let mut min = points[0];
        let mut max = points[0];

        for &p in &points[1..] {
            min = min.min(p);
            max = max.max(p);
        }

        Self { min, max }
    }

    /// Check if a point is inside the AABB.
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if this AABB intersects another.
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Transform the AABB by a matrix.
    /// Returns a new AABB that encompasses the transformed corners.
    pub fn transform(&self, matrix: Mat4) -> Aabb {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let transformed: Vec<Vec3> = corners
            .iter()
            .map(|&c| matrix.transform_point3(c))
            .collect();

        Aabb::from_points(&transformed)
    }

    /// Get the center of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size (extents) of the AABB.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Get the half-size of the AABB.
    pub fn half_size(&self) -> Vec3 {
        self.size() * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_points() {
        let points = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let aabb = Aabb::from_points(&points);
        assert_eq!(aabb.min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_from_points_empty() {
        let aabb = Aabb::from_points(&[]);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }

    #[test]
    fn test_contains_inside() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains(Vec3::ZERO));
        assert!(aabb.contains(Vec3::new(0.5, 0.5, 0.5)));
    }

    #[test]
    fn test_contains_outside() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(!aabb.contains(Vec3::new(2.0, 0.0, 0.0)));
        assert!(!aabb.contains(Vec3::new(0.0, -2.0, 0.0)));
    }

    #[test]
    fn test_contains_boundary() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains(Vec3::new(1.0, 0.0, 0.0)));
        assert!(aabb.contains(Vec3::new(-1.0, -1.0, -1.0)));
    }

    #[test]
    fn test_intersects_overlapping() {
        let a = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn test_intersects_separate() {
        let a = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(0.0, 0.0, 0.0));
        let b = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_center() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
    }

    #[test]
    fn test_size() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.size(), Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_transform_translation() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let matrix = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let transformed = aabb.transform(matrix);
        assert!((transformed.min - Vec3::new(0.0, -1.0, -1.0)).length() < 0.001);
        assert!((transformed.max - Vec3::new(2.0, 1.0, 1.0)).length() < 0.001);
    }
}
