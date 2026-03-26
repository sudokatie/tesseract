use glam::Mat4;
use hecs::Entity;

/// World-space transformation matrix, computed from the hierarchy.
#[derive(Clone, Copy, Debug, Default)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

impl GlobalTransform {
    /// Create a global transform from a matrix.
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }
}

/// Parent entity in the hierarchy.
#[derive(Clone, Copy, Debug)]
pub struct Parent(pub Entity);

/// Child entities in the hierarchy.
#[derive(Clone, Debug, Default)]
pub struct Children(pub Vec<Entity>);

impl Children {
    /// Create a new Children component with the given entities.
    pub fn new(children: Vec<Entity>) -> Self {
        Self(children)
    }

    /// Add a child entity.
    pub fn push(&mut self, child: Entity) {
        self.0.push(child);
    }

    /// Remove a child entity.
    pub fn remove(&mut self, child: Entity) {
        self.0.retain(|&e| e != child);
    }

    /// Check if a child exists.
    pub fn contains(&self, child: Entity) -> bool {
        self.0.contains(&child)
    }

    /// Get the number of children.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Visibility of an entity for rendering.
#[derive(Clone, Copy, Debug)]
pub struct Visibility {
    pub visible: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

impl Visibility {
    /// Create a visible entity.
    pub fn visible() -> Self {
        Self { visible: true }
    }

    /// Create a hidden entity.
    pub fn hidden() -> Self {
        Self { visible: false }
    }
}

/// Debug name for an entity.
#[derive(Clone, Debug, Default)]
pub struct Name(pub String);

impl Name {
    /// Create a new name component.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_transform_default() {
        let gt = GlobalTransform::default();
        assert_eq!(gt.matrix, Mat4::IDENTITY);
    }

    #[test]
    fn test_children_operations() {
        let mut world = hecs::World::new();
        let e1 = world.spawn(());
        let e2 = world.spawn(());
        
        let mut children = Children::default();
        assert!(children.is_empty());
        
        children.push(e1);
        children.push(e2);
        assert_eq!(children.len(), 2);
        assert!(children.contains(e1));
        
        children.remove(e1);
        assert_eq!(children.len(), 1);
        assert!(!children.contains(e1));
    }

    #[test]
    fn test_visibility_default() {
        let v = Visibility::default();
        assert!(v.visible);
    }

    #[test]
    fn test_visibility_hidden() {
        let v = Visibility::hidden();
        assert!(!v.visible);
    }

    #[test]
    fn test_name() {
        let n = Name::new("test entity");
        assert_eq!(n.0, "test entity");
        
        let n2: Name = "from str".into();
        assert_eq!(n2.0, "from str");
    }
}
