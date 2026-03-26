use crate::math::Transform;
use glam::Mat4;

/// A single bone in a skeleton.
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub local_transform: Transform,
}

/// Skeletal data for animation.
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub inverse_bind_matrices: Vec<Mat4>,
}

impl Skeleton {
    /// Create a new skeleton.
    pub fn new(bones: Vec<Bone>, inverse_bind_matrices: Vec<Mat4>) -> Self {
        Self {
            bones,
            inverse_bind_matrices,
        }
    }

    /// Find a bone by name.
    pub fn bone_index(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name == name)
    }

    /// Compute world transforms for all bones given local transforms.
    pub fn compute_world_transforms(&self, local: &[Transform]) -> Vec<Mat4> {
        let mut world = vec![Mat4::IDENTITY; self.bones.len()];
        
        for (i, bone) in self.bones.iter().enumerate() {
            let local_matrix = if i < local.len() {
                local[i].to_matrix()
            } else {
                bone.local_transform.to_matrix()
            };
            
            world[i] = match bone.parent {
                Some(parent) => world[parent] * local_matrix,
                None => local_matrix,
            };
        }
        
        world
    }

    /// Compute skinning matrices (world * inverse_bind).
    pub fn compute_skinning_matrices(&self, world: &[Mat4]) -> Vec<Mat4> {
        world
            .iter()
            .zip(&self.inverse_bind_matrices)
            .map(|(w, ib)| *w * *ib)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_bone_index() {
        let skeleton = Skeleton::new(
            vec![
                Bone {
                    name: "root".into(),
                    parent: None,
                    local_transform: Transform::default(),
                },
                Bone {
                    name: "spine".into(),
                    parent: Some(0),
                    local_transform: Transform::default(),
                },
            ],
            vec![Mat4::IDENTITY, Mat4::IDENTITY],
        );
        
        assert_eq!(skeleton.bone_index("root"), Some(0));
        assert_eq!(skeleton.bone_index("spine"), Some(1));
        assert_eq!(skeleton.bone_index("missing"), None);
    }

    #[test]
    fn test_world_transforms() {
        let skeleton = Skeleton::new(
            vec![
                Bone {
                    name: "root".into(),
                    parent: None,
                    local_transform: Transform::from_xyz(1.0, 0.0, 0.0),
                },
                Bone {
                    name: "child".into(),
                    parent: Some(0),
                    local_transform: Transform::from_xyz(0.0, 1.0, 0.0),
                },
            ],
            vec![Mat4::IDENTITY, Mat4::IDENTITY],
        );
        
        let world = skeleton.compute_world_transforms(&[
            Transform::from_xyz(1.0, 0.0, 0.0),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ]);
        
        let root_pos = world[0].transform_point3(Vec3::ZERO);
        let child_pos = world[1].transform_point3(Vec3::ZERO);
        
        assert!((root_pos - Vec3::new(1.0, 0.0, 0.0)).length() < 0.001);
        assert!((child_pos - Vec3::new(1.0, 1.0, 0.0)).length() < 0.001);
    }
}
