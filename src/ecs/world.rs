use hecs::{Entity, NoSuchEntity, Query, QueryBorrow, World as HecsWorld};

/// Wrapper around hecs::World with convenience methods.
pub struct World {
    inner: HecsWorld,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Create a new empty world.
    pub fn new() -> Self {
        Self {
            inner: HecsWorld::new(),
        }
    }

    /// Spawn an entity with the given components.
    pub fn spawn<T: hecs::DynamicBundle>(&mut self, bundle: T) -> Entity {
        self.inner.spawn(bundle)
    }

    /// Despawn an entity, removing it from the world.
    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.inner.despawn(entity)
    }

    /// Insert a component into an existing entity.
    pub fn insert<T: Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), NoSuchEntity> {
        self.inner.insert_one(entity, component)
    }

    /// Remove a component from an entity.
    pub fn remove<T: Send + Sync + 'static>(&mut self, entity: Entity) -> Result<T, hecs::ComponentError> {
        self.inner.remove_one::<T>(entity)
    }

    /// Get an immutable reference to a component.
    pub fn get<T: Send + Sync + 'static>(&self, entity: Entity) -> Option<hecs::Ref<'_, T>> {
        self.inner.get::<&T>(entity).ok()
    }

    /// Get a mutable reference to a component.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self, entity: Entity) -> Option<hecs::RefMut<'_, T>> {
        self.inner.get::<&mut T>(entity).ok()
    }

    /// Query for entities with specific components.
    pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    /// Query for entities with specific components (mutable).
    pub fn query_mut<Q: Query>(&mut self) -> QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    /// Check if an entity exists.
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }

    /// Get the number of entities.
    pub fn len(&self) -> u32 {
        self.inner.len()
    }

    /// Check if the world is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all entities.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Transform;

    #[derive(Debug, Clone, PartialEq)]
    struct TestComponent(i32);

    #[test]
    fn test_spawn_and_query() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(42),));
        
        let mut found = false;
        for (entity, (comp,)) in world.query::<(&TestComponent,)>().iter() {
            assert_eq!(entity, e);
            assert_eq!(comp.0, 42);
            found = true;
        }
        assert!(found);
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(1),));
        assert!(world.contains(e));
        
        world.despawn(e).unwrap();
        assert!(!world.contains(e));
    }

    #[test]
    fn test_insert() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(1),));
        
        world.insert(e, Transform::default()).unwrap();
        
        assert!(world.get::<Transform>(e).is_some());
    }

    #[test]
    fn test_remove() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(42),));
        
        let removed = world.remove::<TestComponent>(e).unwrap();
        assert_eq!(removed.0, 42);
        assert!(world.get::<TestComponent>(e).is_none());
    }

    #[test]
    fn test_get() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(99),));
        
        let comp = world.get::<TestComponent>(e).unwrap();
        assert_eq!(comp.0, 99);
    }

    #[test]
    fn test_get_mut() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(1),));
        
        {
            let mut comp = world.get_mut::<TestComponent>(e).unwrap();
            comp.0 = 100;
        }
        
        let comp = world.get::<TestComponent>(e).unwrap();
        assert_eq!(comp.0, 100);
    }

    #[test]
    fn test_len() {
        let mut world = World::new();
        assert_eq!(world.len(), 0);
        assert!(world.is_empty());
        
        world.spawn((TestComponent(1),));
        world.spawn((TestComponent(2),));
        assert_eq!(world.len(), 2);
        assert!(!world.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut world = World::new();
        world.spawn((TestComponent(1),));
        world.spawn((TestComponent(2),));
        
        world.clear();
        assert!(world.is_empty());
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let e = world.spawn((TestComponent(5), Transform::from_xyz(1.0, 2.0, 3.0)));
        
        let comp = world.get::<TestComponent>(e).unwrap();
        let trans = world.get::<Transform>(e).unwrap();
        
        assert_eq!(comp.0, 5);
        assert_eq!(trans.position.x, 1.0);
    }
}
