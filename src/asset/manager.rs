use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an asset.
pub type AssetId = u64;

/// Handle to a loaded asset.
#[derive(Debug)]
pub struct Handle<T> {
    id: AssetId,
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// Create a new handle with the given id.
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Get the asset id.
    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Manages loading and storage of assets.
#[derive(Default)]
pub struct AssetManager {
    // For now, just track handles. Full implementation adds typed storage.
    next_id: u64,
}

impl AssetManager {
    /// Create a new asset manager.
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Generate a new unique asset id.
    pub fn next_id(&mut self) -> AssetId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Insert an asset and get a handle.
    /// Note: Full implementation would store by type.
    pub fn insert<T>(&mut self, _asset: T) -> Handle<T> {
        Handle::new(self.next_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestAsset(i32);

    #[test]
    fn test_handle_clone() {
        let h1: Handle<TestAsset> = Handle::new(1);
        let h2 = h1;
        assert_eq!(h1.id(), h2.id());
    }

    #[test]
    fn test_handle_eq() {
        let h1: Handle<TestAsset> = Handle::new(1);
        let h2: Handle<TestAsset> = Handle::new(1);
        let h3: Handle<TestAsset> = Handle::new(2);
        
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_asset_manager_insert() {
        let mut manager = AssetManager::new();
        let h1 = manager.insert(TestAsset(1));
        let h2 = manager.insert(TestAsset(2));
        
        assert_ne!(h1.id(), h2.id());
    }
}
