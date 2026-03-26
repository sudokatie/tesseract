use crate::ecs::{Children, GlobalTransform, Parent, World};
use crate::math::Transform;
use glam::Mat4;
use hecs::Entity;

/// Scene graph helper functions.
pub struct SceneGraph;

impl SceneGraph {
    /// Add a child entity to a parent.
    /// Updates both Parent component on child and Children component on parent.
    pub fn add_child(world: &mut World, parent: Entity, child: Entity) {
        // Set parent on child
        let _ = world.insert(child, Parent(parent));

        // Check if parent has children component
        let has_children = world.get::<Children>(parent).is_some();
        
        if has_children {
            // Add to existing children
            if let Some(mut children) = world.get_mut::<Children>(parent) {
                if !children.contains(child) {
                    children.push(child);
                }
            }
        } else {
            // Create new children component
            let _ = world.insert(parent, Children::new(vec![child]));
        }
    }

    /// Remove a child from its parent.
    pub fn remove_child(world: &mut World, parent: Entity, child: Entity) {
        // Remove parent from child
        let _ = world.remove::<Parent>(child);

        // Remove from parent's children
        if let Some(mut children) = world.get_mut::<Children>(parent) {
            children.remove(child);
        }
    }

    /// Get the parent of an entity, if any.
    pub fn get_parent(world: &World, entity: Entity) -> Option<Entity> {
        world.get::<Parent>(entity).map(|p| p.0)
    }

    /// Get all children of an entity.
    pub fn get_children(world: &World, entity: Entity) -> Vec<Entity> {
        world
            .get::<Children>(entity)
            .map(|c| c.0.clone())
            .unwrap_or_default()
    }

    /// Get all ancestors of an entity (parent, grandparent, etc.)
    pub fn get_ancestors(world: &World, entity: Entity) -> Vec<Entity> {
        let mut ancestors = Vec::new();
        let mut current = entity;

        while let Some(parent) = Self::get_parent(world, current) {
            ancestors.push(parent);
            current = parent;
        }

        ancestors
    }

    /// Get all descendants of an entity (children, grandchildren, etc.)
    pub fn get_descendants(world: &World, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();
        let mut stack = Self::get_children(world, entity);

        while let Some(child) = stack.pop() {
            descendants.push(child);
            stack.extend(Self::get_children(world, child));
        }

        descendants
    }

    /// Get the root ancestor of an entity.
    pub fn get_root(world: &World, entity: Entity) -> Entity {
        let mut current = entity;
        while let Some(parent) = Self::get_parent(world, current) {
            current = parent;
        }
        current
    }

    /// Check if an entity is an ancestor of another.
    pub fn is_ancestor_of(world: &World, ancestor: Entity, descendant: Entity) -> bool {
        let mut current = descendant;
        while let Some(parent) = Self::get_parent(world, current) {
            if parent == ancestor {
                return true;
            }
            current = parent;
        }
        false
    }

    /// Despawn an entity and all its descendants.
    pub fn despawn_recursive(world: &mut World, entity: Entity) {
        // Collect all descendants first
        let descendants = Self::get_descendants(world, entity);

        // Remove from parent if any
        if let Some(parent) = Self::get_parent(world, entity) {
            if let Some(mut children) = world.get_mut::<Children>(parent) {
                children.remove(entity);
            }
        }

        // Despawn all descendants (in reverse to handle children before parents)
        for descendant in descendants.into_iter().rev() {
            let _ = world.despawn(descendant);
        }

        // Despawn the entity itself
        let _ = world.despawn(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_child() {
        let mut world = World::new();
        let parent = world.spawn((Transform::default(),));
        let child = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, parent, child);

        assert_eq!(SceneGraph::get_parent(&world, child), Some(parent));
        assert!(SceneGraph::get_children(&world, parent).contains(&child));
    }

    #[test]
    fn test_remove_child() {
        let mut world = World::new();
        let parent = world.spawn((Transform::default(),));
        let child = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, parent, child);
        SceneGraph::remove_child(&mut world, parent, child);

        assert_eq!(SceneGraph::get_parent(&world, child), None);
        assert!(!SceneGraph::get_children(&world, parent).contains(&child));
    }

    #[test]
    fn test_get_ancestors() {
        let mut world = World::new();
        let grandparent = world.spawn((Transform::default(),));
        let parent = world.spawn((Transform::default(),));
        let child = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, grandparent, parent);
        SceneGraph::add_child(&mut world, parent, child);

        let ancestors = SceneGraph::get_ancestors(&world, child);
        assert_eq!(ancestors, vec![parent, grandparent]);
    }

    #[test]
    fn test_get_descendants() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(),));
        let child1 = world.spawn((Transform::default(),));
        let child2 = world.spawn((Transform::default(),));
        let grandchild = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, root, child1);
        SceneGraph::add_child(&mut world, root, child2);
        SceneGraph::add_child(&mut world, child1, grandchild);

        let descendants = SceneGraph::get_descendants(&world, root);
        assert_eq!(descendants.len(), 3);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&child2));
        assert!(descendants.contains(&grandchild));
    }

    #[test]
    fn test_get_root() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(),));
        let child = world.spawn((Transform::default(),));
        let grandchild = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, root, child);
        SceneGraph::add_child(&mut world, child, grandchild);

        assert_eq!(SceneGraph::get_root(&world, grandchild), root);
        assert_eq!(SceneGraph::get_root(&world, child), root);
        assert_eq!(SceneGraph::get_root(&world, root), root);
    }

    #[test]
    fn test_is_ancestor_of() {
        let mut world = World::new();
        let grandparent = world.spawn((Transform::default(),));
        let parent = world.spawn((Transform::default(),));
        let child = world.spawn((Transform::default(),));
        let unrelated = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, grandparent, parent);
        SceneGraph::add_child(&mut world, parent, child);

        assert!(SceneGraph::is_ancestor_of(&world, grandparent, child));
        assert!(SceneGraph::is_ancestor_of(&world, parent, child));
        assert!(!SceneGraph::is_ancestor_of(&world, child, parent));
        assert!(!SceneGraph::is_ancestor_of(&world, unrelated, child));
    }

    #[test]
    fn test_despawn_recursive() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(),));
        let child = world.spawn((Transform::default(),));
        let grandchild = world.spawn((Transform::default(),));

        SceneGraph::add_child(&mut world, root, child);
        SceneGraph::add_child(&mut world, child, grandchild);

        SceneGraph::despawn_recursive(&mut world, root);

        assert!(!world.contains(root));
        assert!(!world.contains(child));
        assert!(!world.contains(grandchild));
    }
}
