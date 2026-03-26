use crate::math::Transform;
use glam::Mat4;

/// Camera projection type.
#[derive(Clone, Copy, Debug)]
pub enum Projection {
    Perspective { fov: f32, near: f32, far: f32 },
    Orthographic { size: f32, near: f32, far: f32 },
}

/// Camera component.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub projection: Projection,
}

impl Camera {
    /// Create a perspective camera.
    /// FOV is in radians.
    pub fn perspective(fov: f32, near: f32, far: f32) -> Self {
        Self {
            projection: Projection::Perspective { fov, near, far },
        }
    }

    /// Create an orthographic camera.
    pub fn orthographic(size: f32, near: f32, far: f32) -> Self {
        Self {
            projection: Projection::Orthographic { size, near, far },
        }
    }

    /// Get the view matrix from a transform.
    pub fn view_matrix(&self, transform: &Transform) -> Mat4 {
        transform.to_matrix().inverse()
    }

    /// Get the projection matrix.
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective { fov, near, far } => {
                Mat4::perspective_rh(fov, aspect, near, far)
            }
            Projection::Orthographic { size, near, far } => {
                let half_height = size * 0.5;
                let half_width = half_height * aspect;
                Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, near, far)
            }
        }
    }

    /// Get the combined view-projection matrix.
    pub fn view_projection(&self, transform: &Transform, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix(transform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Transform;
    use glam::{Mat4, Vec3};

    #[test]
    fn test_perspective_camera() {
        let cam = Camera::perspective(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        match cam.projection {
            Projection::Perspective { fov, near, far } => {
                assert!((fov - std::f32::consts::FRAC_PI_4).abs() < 0.001);
                assert!((near - 0.1).abs() < 0.001);
                assert!((far - 100.0).abs() < 0.001);
            }
            _ => panic!("expected perspective"),
        }
    }

    #[test]
    fn test_orthographic_camera() {
        let cam = Camera::orthographic(10.0, 0.1, 100.0);
        match cam.projection {
            Projection::Orthographic { size, .. } => {
                assert!((size - 10.0).abs() < 0.001);
            }
            _ => panic!("expected orthographic"),
        }
    }

    #[test]
    fn test_view_matrix() {
        let cam = Camera::perspective(1.0, 0.1, 100.0);
        let transform = Transform::from_xyz(0.0, 0.0, 5.0);
        let view = cam.view_matrix(&transform);
        
        // View matrix should translate origin by -5 in Z
        let p = view.transform_point3(Vec3::ZERO);
        assert!((p.z - (-5.0)).abs() < 0.001);
    }

    #[test]
    fn test_projection_matrix() {
        let cam = Camera::perspective(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        let proj = cam.projection_matrix(1.0);
        // Just verify it's not identity and has reasonable values
        assert!(proj != Mat4::IDENTITY);
        assert!(proj.w_axis.z < 0.0); // Perspective projection has negative w
    }
}
