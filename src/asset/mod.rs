mod manager;
pub mod gltf;

pub use manager::{AssetId, AssetManager, Handle};
pub use gltf::{load_gltf, GltfError, GltfNode, GltfScene};
