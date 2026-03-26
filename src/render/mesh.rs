use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Vertex data for rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent: [1.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Mesh data.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Create a new mesh from vertices and indices.
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Create a unit cube centered at origin.
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back face
            Vertex::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 0.0]),
            Vertex::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 1.0]),
            // Right face
            Vertex::new([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, -0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([0.5, 0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]),
            // Left face
            Vertex::new([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]),
            // Top face
            Vertex::new([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 1.0]),
            // Bottom face
            Vertex::new([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [1.0, 0.0]),
            Vertex::new([0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [0.0, 1.0]),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            8, 9, 10, 10, 11, 8, // Right
            12, 13, 14, 14, 15, 12, // Left
            16, 17, 18, 18, 19, 16, // Top
            20, 21, 22, 22, 23, 20, // Bottom
        ];

        Self { vertices, indices }
    }

    /// Create a plane in the XZ plane.
    pub fn plane(size: f32) -> Self {
        let half = size * 0.5;
        let vertices = vec![
            Vertex::new([-half, 0.0, -half], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([half, 0.0, -half], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([half, 0.0, half], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-half, 0.0, half], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self { vertices, indices }
    }

    /// Get the bounding box of this mesh.
    pub fn bounds(&self) -> crate::math::Aabb {
        let points: Vec<Vec3> = self
            .vertices
            .iter()
            .map(|v| Vec3::from(v.position))
            .collect();
        crate::math::Aabb::from_points(&points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_vertex_size() {
        // 3 + 3 + 2 + 4 = 12 floats = 48 bytes
        assert_eq!(size_of::<Vertex>(), 48);
    }

    #[test]
    fn test_cube_mesh() {
        let cube = Mesh::cube();
        assert_eq!(cube.vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(cube.indices.len(), 36); // 6 faces * 2 triangles * 3 indices
    }

    #[test]
    fn test_plane_mesh() {
        let plane = Mesh::plane(1.0);
        assert_eq!(plane.vertices.len(), 4);
        assert_eq!(plane.indices.len(), 6);
    }

    #[test]
    fn test_mesh_bounds() {
        let cube = Mesh::cube();
        let bounds = cube.bounds();
        assert!((bounds.min.x - (-0.5)).abs() < 0.001);
        assert!((bounds.max.x - 0.5).abs() < 0.001);
    }
}
