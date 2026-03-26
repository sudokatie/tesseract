use glam::{Mat4, Quat, Vec3};

/// A 3D transform with position, rotation, and scale.
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Create a transform at the given position.
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            ..Default::default()
        }
    }

    /// Create a transform with the given rotation.
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Create a transform with the given scale.
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Create a transform with uniform scale.
    pub fn from_scale_uniform(scale: f32) -> Self {
        Self::from_scale(Vec3::splat(scale))
    }

    /// Set rotation to look at a target point.
    pub fn looking_at(mut self, target: Vec3) -> Self {
        let forward = (target - self.position).normalize_or_zero();
        if forward.length_squared() > 0.0 {
            self.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        }
        self
    }

    /// Convert to a 4x4 transformation matrix (TRS order).
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Compose this transform with a parent transform.
    /// Result is: parent * self (parent applied first).
    pub fn compose(&self, parent: &Transform) -> Transform {
        Transform {
            position: parent.position + parent.rotation * (parent.scale * self.position),
            rotation: parent.rotation * self.rotation,
            scale: parent.scale * self.scale,
        }
    }

    /// Get the forward direction (-Z in local space).
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction (+X in local space).
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (+Y in local space).
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_transform() {
        let t = Transform::default();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_from_xyz() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        assert_eq!(t.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_to_matrix_identity() {
        let t = Transform::default();
        let m = t.to_matrix();
        assert_eq!(m, Mat4::IDENTITY);
    }

    #[test]
    fn test_to_matrix_translation() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        let m = t.to_matrix();
        let expected = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(m, expected);
    }

    #[test]
    fn test_to_matrix_scale() {
        let t = Transform::from_scale(Vec3::new(2.0, 2.0, 2.0));
        let m = t.to_matrix();
        let expected = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(m, expected);
    }

    #[test]
    fn test_compose_translation() {
        let parent = Transform::from_xyz(1.0, 0.0, 0.0);
        let child = Transform::from_xyz(0.0, 1.0, 0.0);
        let composed = child.compose(&parent);
        assert!((composed.position - Vec3::new(1.0, 1.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_compose_scale() {
        let parent = Transform::from_scale(Vec3::splat(2.0));
        let child = Transform::from_xyz(1.0, 0.0, 0.0);
        let composed = child.compose(&parent);
        assert!((composed.position - Vec3::new(2.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_forward_default() {
        let t = Transform::default();
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_Z).length() < 0.001);
    }

    #[test]
    fn test_looking_at() {
        let t = Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO);
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_Z).length() < 0.001);
    }
}
