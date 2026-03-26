use std::collections::HashMap;
use std::marker::PhantomData;

use crate::animation::{AnimationClip, Skeleton};
use crate::render::{Mesh, PbrMaterial, Texture};

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
        *self
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
pub struct AssetManager {
    next_id: AssetId,
    meshes: HashMap<AssetId, Mesh>,
    textures: HashMap<AssetId, Texture>,
    materials: HashMap<AssetId, PbrMaterial>,
    animations: HashMap<AssetId, AnimationClip>,
    skeletons: HashMap<AssetId, Skeleton>,
    path_to_id: HashMap<String, AssetId>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    /// Create a new asset manager.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            animations: HashMap::new(),
            skeletons: HashMap::new(),
            path_to_id: HashMap::new(),
        }
    }

    /// Generate a new unique asset id.
    fn allocate_id(&mut self) -> AssetId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Insert a mesh and get a handle.
    pub fn insert_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        let id = self.allocate_id();
        self.meshes.insert(id, mesh);
        Handle::new(id)
    }

    /// Get a mesh by handle.
    pub fn get_mesh(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.meshes.get(&handle.id())
    }

    /// Insert a texture and get a handle.
    pub fn insert_texture(&mut self, texture: Texture) -> Handle<Texture> {
        let id = self.allocate_id();
        self.textures.insert(id, texture);
        Handle::new(id)
    }

    /// Get a texture by handle.
    pub fn get_texture(&self, handle: Handle<Texture>) -> Option<&Texture> {
        self.textures.get(&handle.id())
    }

    /// Insert a material and get a handle.
    pub fn insert_material(&mut self, material: PbrMaterial) -> Handle<PbrMaterial> {
        let id = self.allocate_id();
        self.materials.insert(id, material);
        Handle::new(id)
    }

    /// Get a material by handle.
    pub fn get_material(&self, handle: Handle<PbrMaterial>) -> Option<&PbrMaterial> {
        self.materials.get(&handle.id())
    }

    /// Insert an animation clip and get a handle.
    pub fn insert_animation(&mut self, clip: AnimationClip) -> Handle<AnimationClip> {
        let id = self.allocate_id();
        self.animations.insert(id, clip);
        Handle::new(id)
    }

    /// Get an animation clip by handle.
    pub fn get_animation(&self, handle: Handle<AnimationClip>) -> Option<&AnimationClip> {
        self.animations.get(&handle.id())
    }

    /// Insert a skeleton and get a handle.
    pub fn insert_skeleton(&mut self, skeleton: Skeleton) -> Handle<Skeleton> {
        let id = self.allocate_id();
        self.skeletons.insert(id, skeleton);
        Handle::new(id)
    }

    /// Get a skeleton by handle.
    pub fn get_skeleton(&self, handle: Handle<Skeleton>) -> Option<&Skeleton> {
        self.skeletons.get(&handle.id())
    }

    /// Check if a path has been loaded.
    pub fn is_loaded(&self, path: &str) -> bool {
        self.path_to_id.contains_key(path)
    }

    /// Get the asset id for a path, if loaded.
    pub fn get_id_for_path(&self, path: &str) -> Option<AssetId> {
        self.path_to_id.get(path).copied()
    }

    /// Register a path -> id mapping.
    pub fn register_path(&mut self, path: &str, id: AssetId) {
        self.path_to_id.insert(path.to_string(), id);
    }

    /// Get total number of loaded assets.
    pub fn asset_count(&self) -> usize {
        self.meshes.len()
            + self.textures.len()
            + self.materials.len()
            + self.animations.len()
            + self.skeletons.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::TextureFormat;

    #[test]
    fn test_handle_clone() {
        let h1: Handle<Mesh> = Handle::new(1);
        let h2 = h1;
        assert_eq!(h1.id(), h2.id());
    }

    #[test]
    fn test_handle_eq() {
        let h1: Handle<Mesh> = Handle::new(1);
        let h2: Handle<Mesh> = Handle::new(1);
        let h3: Handle<Mesh> = Handle::new(2);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_insert_mesh() {
        let mut manager = AssetManager::new();
        let mesh = Mesh::cube();
        let handle = manager.insert_mesh(mesh);

        assert!(manager.get_mesh(handle).is_some());
        assert_eq!(manager.asset_count(), 1);
    }

    #[test]
    fn test_insert_texture() {
        let mut manager = AssetManager::new();
        let texture = Texture::new(512, 512, TextureFormat::Rgba8);
        let handle = manager.insert_texture(texture);

        assert!(manager.get_texture(handle).is_some());
    }

    #[test]
    fn test_insert_material() {
        let mut manager = AssetManager::new();
        let material = PbrMaterial::default();
        let handle = manager.insert_material(material);

        assert!(manager.get_material(handle).is_some());
    }

    #[test]
    fn test_unique_ids() {
        let mut manager = AssetManager::new();
        let h1 = manager.insert_mesh(Mesh::cube());
        let h2 = manager.insert_mesh(Mesh::plane(1.0));

        assert_ne!(h1.id(), h2.id());
    }

    #[test]
    fn test_path_registration() {
        let mut manager = AssetManager::new();
        let mesh = Mesh::cube();
        let handle = manager.insert_mesh(mesh);

        manager.register_path("models/cube.gltf", handle.id());

        assert!(manager.is_loaded("models/cube.gltf"));
        assert_eq!(manager.get_id_for_path("models/cube.gltf"), Some(handle.id()));
        assert!(!manager.is_loaded("models/sphere.gltf"));
    }

    #[test]
    fn test_get_invalid_handle() {
        let manager = AssetManager::new();
        let fake_handle: Handle<Mesh> = Handle::new(999);

        assert!(manager.get_mesh(fake_handle).is_none());
    }
}
