use super::components::{Children, GlobalTransform, Parent};
use super::world::World;
use crate::math::Transform;
use glam::Mat4;
use hecs::Entity;

/// System execution phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Handle user input.
    Input,
    /// Main game logic update.
    Update,
    /// Post-update processing (physics, AI).
    PostUpdate,
    /// Pre-render preparation (transform propagation).
    PreRender,
    /// Rendering.
    Render,
}

/// Trait for systems that process the world.
pub trait System {
    /// Run the system on the world.
    fn run(&mut self, world: &mut World);
}

/// Propagate transforms through the hierarchy.
/// Updates GlobalTransform for all entities based on their local Transform
/// and parent hierarchy.
pub fn transform_propagation_system(world: &mut World) {
    // Collect root entities (those without Parent component)
    let mut roots: Vec<(Entity, Transform)> = Vec::new();
    
    for (entity, transform) in world.query::<&Transform>().iter() {
        if world.get::<Parent>(entity).is_none() {
            roots.push((entity, *transform));
        }
    }
    
    // Process each root and its descendants
    // Roots have no parent, so parent_matrix is identity
    for (entity, _transform) in roots {
        propagate_recursive(world, entity, Mat4::IDENTITY);
    }
}

fn propagate_recursive(world: &mut World, entity: Entity, parent_matrix: Mat4) {
    // Get the local transform for this entity
    let local_matrix = world
        .get::<Transform>(entity)
        .map(|t| t.to_matrix())
        .unwrap_or(Mat4::IDENTITY);
    
    // Compute world matrix
    let world_matrix = parent_matrix * local_matrix;
    
    // Update GlobalTransform if present
    if let Some(mut global) = world.get_mut::<GlobalTransform>(entity) {
        global.matrix = world_matrix;
    }
    
    // Get children (need to collect to avoid borrow issues)
    let children: Vec<Entity> = world
        .get::<Children>(entity)
        .map(|c| c.0.clone())
        .unwrap_or_default();
    
    // Recurse to children
    for child in children {
        propagate_recursive(world, child, world_matrix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_root_transform_propagation() {
        let mut world = World::new();
        
        let e = world.spawn((
            Transform::from_xyz(1.0, 2.0, 3.0),
            GlobalTransform::default(),
        ));
        
        transform_propagation_system(&mut world);
        
        let global = world.get::<GlobalTransform>(e).unwrap();
        let pos = global.matrix.transform_point3(Vec3::ZERO);
        assert!((pos - Vec3::new(1.0, 2.0, 3.0)).length() < 0.001);
    }

    #[test]
    fn test_child_transform_propagation() {
        let mut world = World::new();
        
        let parent_entity = world.spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
        
        let child_entity = world.spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            Parent(parent_entity),
        ));
        
        // Add child to parent's Children component
        world.insert(parent_entity, Children::new(vec![child_entity])).unwrap();
        
        transform_propagation_system(&mut world);
        
        let child_global = world.get::<GlobalTransform>(child_entity).unwrap();
        let pos = child_global.matrix.transform_point3(Vec3::ZERO);
        // Parent at (1,0,0) + child offset (0,1,0) = (1,1,0)
        assert!((pos - Vec3::new(1.0, 1.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_deep_hierarchy() {
        let mut world = World::new();
        
        let grandparent = world.spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
        
        let parent_entity = world.spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
            Parent(grandparent),
        ));
        
        let child = world.spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            GlobalTransform::default(),
            Parent(parent_entity),
        ));
        
        world.insert(grandparent, Children::new(vec![parent_entity])).unwrap();
        world.insert(parent_entity, Children::new(vec![child])).unwrap();
        
        transform_propagation_system(&mut world);
        
        let child_global = world.get::<GlobalTransform>(child).unwrap();
        let pos = child_global.matrix.transform_point3(Vec3::ZERO);
        // (1,0,0) + (0,1,0) + (0,0,1) = (1,1,1)
        assert!((pos - Vec3::new(1.0, 1.0, 1.0)).length() < 0.001);
    }

    #[test]
    fn test_scaled_parent() {
        let mut world = World::new();
        
        let parent_entity = world.spawn((
            Transform::from_scale(glam::Vec3::splat(2.0)),
            GlobalTransform::default(),
        ));
        
        let child_entity = world.spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent_entity),
        ));
        
        world.insert(parent_entity, Children::new(vec![child_entity])).unwrap();
        
        transform_propagation_system(&mut world);
        
        let child_global = world.get::<GlobalTransform>(child_entity).unwrap();
        let pos = child_global.matrix.transform_point3(Vec3::ZERO);
        // Child at (1,0,0) scaled by 2 = (2,0,0)
        assert!((pos - Vec3::new(2.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_multiple_roots() {
        let mut world = World::new();
        
        let e1 = world.spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
        
        let e2 = world.spawn((
            Transform::from_xyz(0.0, 2.0, 0.0),
            GlobalTransform::default(),
        ));
        
        transform_propagation_system(&mut world);
        
        let g1 = world.get::<GlobalTransform>(e1).unwrap();
        let g2 = world.get::<GlobalTransform>(e2).unwrap();
        
        assert!((g1.matrix.transform_point3(Vec3::ZERO) - Vec3::new(1.0, 0.0, 0.0)).length() < 0.001);
        assert!((g2.matrix.transform_point3(Vec3::ZERO) - Vec3::new(0.0, 2.0, 0.0)).length() < 0.001);
    }
}
