use crate::math::Transform;
use glam::{Quat, Vec3};

/// A pose is a collection of bone transforms.
pub type Pose = Vec<(usize, Transform)>;

/// Blend two poses together.
pub fn blend_poses(a: &Pose, b: &Pose, factor: f32) -> Pose {
    let factor = factor.clamp(0.0, 1.0);
    
    // Create a map of bone_index -> transform for pose b
    let b_map: std::collections::HashMap<usize, &Transform> = b.iter().map(|(i, t)| (*i, t)).collect();
    
    a.iter()
        .map(|(bone_idx, a_transform)| {
            let blended = if let Some(b_transform) = b_map.get(bone_idx) {
                Transform {
                    position: a_transform.position.lerp(b_transform.position, factor),
                    rotation: a_transform.rotation.slerp(b_transform.rotation, factor),
                    scale: a_transform.scale.lerp(b_transform.scale, factor),
                }
            } else {
                *a_transform
            };
            (*bone_idx, blended)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_at_zero() {
        let a = vec![(0, Transform::from_xyz(0.0, 0.0, 0.0))];
        let b = vec![(0, Transform::from_xyz(1.0, 0.0, 0.0))];
        
        let result = blend_poses(&a, &b, 0.0);
        assert!((result[0].1.position.x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_blend_at_one() {
        let a = vec![(0, Transform::from_xyz(0.0, 0.0, 0.0))];
        let b = vec![(0, Transform::from_xyz(1.0, 0.0, 0.0))];
        
        let result = blend_poses(&a, &b, 1.0);
        assert!((result[0].1.position.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_blend_at_half() {
        let a = vec![(0, Transform::from_xyz(0.0, 0.0, 0.0))];
        let b = vec![(0, Transform::from_xyz(1.0, 0.0, 0.0))];
        
        let result = blend_poses(&a, &b, 0.5);
        assert!((result[0].1.position.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_blend_rotation() {
        // Use a smaller rotation to avoid ambiguity at 180 degrees
        let a = vec![(0, Transform::from_rotation(Quat::IDENTITY))];
        let b = vec![(0, Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)))];
        
        let result = blend_poses(&a, &b, 0.5);
        // At half blend between 0 and 90 degrees, should be 45 degrees
        let (y, _, _) = result[0].1.rotation.to_euler(glam::EulerRot::YXZ);
        assert!((y - std::f32::consts::FRAC_PI_4).abs() < 0.01);
    }
}
