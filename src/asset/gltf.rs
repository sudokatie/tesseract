//! glTF 2.0 asset loading.

use std::path::Path;

use crate::animation::{AnimationClip, Bone, Channel, Interpolation, Keyframe, Property, Skeleton};
use crate::asset::{AssetManager, Handle};
use crate::math::Transform;
use crate::render::{Mesh, PbrMaterial, Texture, TextureFormat, Vertex};
use glam::{Mat4, Quat, Vec3};
use thiserror::Error;

/// Error loading glTF files.
#[derive(Debug, Error)]
pub enum GltfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("glTF error: {0}")]
    Gltf(#[from] gltf::Error),
    #[error("Missing buffer data")]
    MissingBuffer,
    #[error("Unsupported primitive mode")]
    UnsupportedPrimitive,
}

/// Result of loading a glTF file.
pub struct GltfScene {
    pub meshes: Vec<Handle<Mesh>>,
    pub materials: Vec<Handle<PbrMaterial>>,
    pub textures: Vec<Handle<Texture>>,
    pub animations: Vec<Handle<AnimationClip>>,
    pub skeleton: Option<Handle<Skeleton>>,
    pub nodes: Vec<GltfNode>,
}

/// A node in the glTF scene graph.
#[derive(Clone, Debug)]
pub struct GltfNode {
    pub name: String,
    pub transform: Transform,
    pub mesh_index: Option<usize>,
    pub children: Vec<usize>,
}

/// Load a glTF file and return the scene data.
pub fn load_gltf(
    path: impl AsRef<Path>,
    assets: &mut AssetManager,
) -> Result<GltfScene, GltfError> {
    let path = path.as_ref();
    let (document, buffers, images) = gltf::import(path)?;

    // Load textures
    let textures: Vec<Handle<Texture>> = images
        .iter()
        .map(|img| {
            let texture = Texture::new(
                img.width,
                img.height,
                TextureFormat::Rgba8,
            );
            assets.insert_texture(texture)
        })
        .collect();

    // Load materials
    let materials: Vec<Handle<PbrMaterial>> = document
        .materials()
        .map(|mat| {
            let pbr = mat.pbr_metallic_roughness();
            let material = PbrMaterial {
                albedo: pbr.base_color_factor(),
                metallic: pbr.metallic_factor(),
                roughness: pbr.roughness_factor(),
                albedo_texture: pbr
                    .base_color_texture()
                    .map(|t| textures[t.texture().source().index()]),
                normal_texture: mat
                    .normal_texture()
                    .map(|t| textures[t.texture().source().index()]),
                metallic_roughness_texture: pbr
                    .metallic_roughness_texture()
                    .map(|t| textures[t.texture().source().index()]),
                ao_texture: mat
                    .occlusion_texture()
                    .map(|t| textures[t.texture().source().index()]),
                emissive: mat.emissive_factor(),
                emissive_texture: mat
                    .emissive_texture()
                    .map(|t| textures[t.texture().source().index()]),
            };
            assets.insert_material(material)
        })
        .collect();

    // Load meshes
    let mut meshes: Vec<Handle<Mesh>> = Vec::new();
    for gltf_mesh in document.meshes() {
        for prim in gltf_mesh.primitives() {
            if let Ok(mesh) = load_primitive(&prim, &buffers) {
                meshes.push(assets.insert_mesh(mesh));
            }
        }
    }

    // Load nodes
    let nodes: Vec<GltfNode> = document
        .nodes()
        .map(|node| {
            let (translation, rotation, scale) = node.transform().decomposed();
            GltfNode {
                name: node.name().unwrap_or("").to_string(),
                transform: Transform {
                    position: Vec3::from(translation),
                    rotation: Quat::from_array(rotation),
                    scale: Vec3::from(scale),
                },
                mesh_index: node.mesh().map(|m| m.index()),
                children: node.children().map(|c| c.index()).collect(),
            }
        })
        .collect();

    // Load skeleton (first skin found)
    let skeleton = document.skins().next().map(|skin| {
        let bones = skin
            .joints()
            .enumerate()
            .map(|(i, joint)| {
                let (t, r, s) = joint.transform().decomposed();
                Bone {
                    name: joint.name().unwrap_or("").to_string(),
                    parent: find_parent_bone(&skin, i),
                    local_transform: Transform {
                        position: Vec3::from(t),
                        rotation: Quat::from_array(r),
                        scale: Vec3::from(s),
                    },
                }
            })
            .collect();

        let inverse_bind_matrices = load_inverse_bind_matrices(&skin, &buffers);
        assets.insert_skeleton(Skeleton::new(bones, inverse_bind_matrices))
    });

    // Load animations
    let animations: Vec<Handle<AnimationClip>> = document
        .animations()
        .map(|anim| {
            let clip = load_animation(&anim, &buffers);
            assets.insert_animation(clip)
        })
        .collect();

    Ok(GltfScene {
        meshes,
        materials,
        textures,
        animations,
        skeleton,
        nodes,
    })
}

fn load_primitive(
    prim: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
) -> Result<Mesh, GltfError> {
    let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or(GltfError::MissingBuffer)?
        .collect();

    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|n| n.collect())
        .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|tc| tc.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    let tangents: Vec<[f32; 4]> = reader
        .read_tangents()
        .map(|t| t.collect())
        .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

    let vertices: Vec<Vertex> = positions
        .iter()
        .zip(normals.iter())
        .zip(uvs.iter())
        .zip(tangents.iter())
        .map(|(((pos, norm), uv), tan)| Vertex {
            position: *pos,
            normal: *norm,
            uv: *uv,
            tangent: *tan,
        })
        .collect();

    let indices: Vec<u32> = reader
        .read_indices()
        .map(|idx| idx.into_u32().collect())
        .unwrap_or_else(|| (0..vertices.len() as u32).collect());

    Ok(Mesh::new(vertices, indices))
}

fn find_parent_bone(skin: &gltf::Skin, bone_index: usize) -> Option<usize> {
    let joints: Vec<_> = skin.joints().collect();
    let bone_node = &joints[bone_index];

    // Search for a joint that has this bone as a child
    for (i, joint) in joints.iter().enumerate() {
        if i == bone_index {
            continue;
        }
        for child in joint.children() {
            if child.index() == bone_node.index() {
                return Some(i);
            }
        }
    }
    None
}

fn load_inverse_bind_matrices(
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
) -> Vec<Mat4> {
    skin.reader(|buffer| Some(&buffers[buffer.index()]))
        .read_inverse_bind_matrices()
        .map(|ibm| ibm.map(|m| Mat4::from_cols_array_2d(&m)).collect())
        .unwrap_or_else(|| vec![Mat4::IDENTITY; skin.joints().count()])
}

fn load_animation(anim: &gltf::Animation, buffers: &[gltf::buffer::Data]) -> AnimationClip {
    let mut duration = 0.0f32;
    let channels: Vec<Channel> = anim
        .channels()
        .filter_map(|channel| {
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            let times: Vec<f32> = reader.read_inputs()?.collect();
            let outputs: Vec<[f32; 4]> = match reader.read_outputs()? {
                gltf::animation::util::ReadOutputs::Translations(t) => {
                    t.map(|v| [v[0], v[1], v[2], 0.0]).collect()
                }
                gltf::animation::util::ReadOutputs::Rotations(r) => {
                    r.into_f32().map(|q| [q[0], q[1], q[2], q[3]]).collect()
                }
                gltf::animation::util::ReadOutputs::Scales(s) => {
                    s.map(|v| [v[0], v[1], v[2], 1.0]).collect()
                }
                gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => return None,
            };

            if let Some(&last_time) = times.last() {
                duration = duration.max(last_time);
            }

            let property = match channel.target().property() {
                gltf::animation::Property::Translation => Property::Position,
                gltf::animation::Property::Rotation => Property::Rotation,
                gltf::animation::Property::Scale => Property::Scale,
                gltf::animation::Property::MorphTargetWeights => return None,
            };

            let interpolation = match channel.sampler().interpolation() {
                gltf::animation::Interpolation::Linear => Interpolation::Linear,
                gltf::animation::Interpolation::Step => Interpolation::Step,
                gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
            };

            let keyframes: Vec<Keyframe> = times
                .iter()
                .zip(outputs.iter())
                .map(|(&time, &value)| Keyframe { time, value })
                .collect();

            Some(Channel {
                bone_index: channel.target().node().index(),
                property,
                interpolation,
                keyframes,
            })
        })
        .collect();

    AnimationClip {
        name: anim.name().unwrap_or("").to_string(),
        duration,
        channels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gltf_node_default() {
        let node = GltfNode {
            name: "test".into(),
            transform: Transform::default(),
            mesh_index: None,
            children: vec![],
        };
        assert_eq!(node.name, "test");
        assert!(node.mesh_index.is_none());
    }

    #[test]
    fn test_gltf_scene_empty() {
        // Can't test actual loading without a file, but can test structure
        let scene = GltfScene {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            animations: vec![],
            skeleton: None,
            nodes: vec![],
        };
        assert!(scene.meshes.is_empty());
        assert!(scene.skeleton.is_none());
    }
}
