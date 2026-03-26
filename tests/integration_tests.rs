//! Integration tests for Tesseract engine.

use tesseract::prelude::*;
use tesseract::asset::{AssetManager, Handle};
use tesseract::ecs::{Children, Parent, World, transform_propagation_system};
use tesseract::scene::SceneGraph;

// ============ Engine Tests ============

#[test]
fn test_engine_creates_world() {
    let engine = Engine::with_default_config();
    assert!(engine.world.is_empty());
}

#[test]
fn test_engine_spawn_entities() {
    let mut engine = Engine::with_default_config();
    
    let e1 = engine.world.spawn((Transform::default(),));
    let e2 = engine.world.spawn((Transform::default(),));
    
    assert_eq!(engine.world.len(), 2);
    assert!(engine.world.contains(e1));
    assert!(engine.world.contains(e2));
}

#[test]
fn test_engine_headless_updates() {
    let mut engine = Engine::with_default_config();
    let entity = engine.world.spawn((
        Transform::default(),
        GlobalTransform::default(),
    ));
    
    let mut count = 0;
    engine.run_headless(|world, _assets, _dt| {
        if let Some(mut t) = world.get_mut::<Transform>(entity) {
            t.position.x += 1.0;
        }
        count += 1;
    }, 5);
    
    assert_eq!(count, 5);
    let pos = engine.world.get::<Transform>(entity).unwrap().position;
    assert_eq!(pos.x, 5.0);
}

// ============ Scene Graph Tests ============

#[test]
fn test_scene_hierarchy_propagation() {
    let mut world = World::new();
    
    let parent = world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));
    
    let child = world.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        GlobalTransform::default(),
    ));
    
    SceneGraph::add_child(&mut world, parent, child);
    transform_propagation_system(&mut world);
    
    let child_global = world.get::<GlobalTransform>(child).unwrap();
    let pos = child_global.matrix.transform_point3(Vec3::ZERO);
    
    // Child at (0, 5, 0) with parent at (10, 0, 0) = world pos (10, 5, 0)
    assert!((pos.x - 10.0).abs() < 0.001);
    assert!((pos.y - 5.0).abs() < 0.001);
}

#[test]
fn test_deep_hierarchy() {
    let mut world = World::new();
    
    let root = world.spawn((
        Transform::from_xyz(1.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));
    
    let mid = world.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        GlobalTransform::default(),
    ));
    
    let leaf = world.spawn((
        Transform::from_xyz(0.0, 0.0, 1.0),
        GlobalTransform::default(),
    ));
    
    SceneGraph::add_child(&mut world, root, mid);
    SceneGraph::add_child(&mut world, mid, leaf);
    transform_propagation_system(&mut world);
    
    let leaf_global = world.get::<GlobalTransform>(leaf).unwrap();
    let pos = leaf_global.matrix.transform_point3(Vec3::ZERO);
    
    // Accumulated: (1, 0, 0) + (0, 1, 0) + (0, 0, 1) = (1, 1, 1)
    assert!((pos - Vec3::new(1.0, 1.0, 1.0)).length() < 0.001);
}

// ============ Asset Manager Tests ============

#[test]
fn test_asset_manager_meshes() {
    let mut assets = AssetManager::new();
    
    let cube = assets.insert_mesh(Mesh::cube());
    let plane = assets.insert_mesh(Mesh::plane(5.0));
    
    assert!(assets.get_mesh(cube).is_some());
    assert!(assets.get_mesh(plane).is_some());
    assert_ne!(cube.id(), plane.id());
}

#[test]
fn test_asset_manager_materials() {
    let mut assets = AssetManager::new();
    
    let red = assets.insert_material(PbrMaterial::with_albedo([1.0, 0.0, 0.0, 1.0]));
    let metal = assets.insert_material(PbrMaterial::metallic(1.0, 0.2));
    
    let red_mat = assets.get_material(red).unwrap();
    let metal_mat = assets.get_material(metal).unwrap();
    
    assert_eq!(red_mat.albedo[0], 1.0);
    assert_eq!(metal_mat.metallic, 1.0);
}

#[test]
fn test_asset_count() {
    let mut assets = AssetManager::new();
    
    assets.insert_mesh(Mesh::cube());
    assets.insert_mesh(Mesh::plane(1.0));
    assets.insert_material(PbrMaterial::default());
    
    assert_eq!(assets.asset_count(), 3);
}

// ============ Animation Tests ============

#[test]
fn test_animation_player_loop() {
    use tesseract::animation::{AnimationClip, AnimationPlayer, Channel, Keyframe, Property, Interpolation, LoopMode};
    
    let mut assets = AssetManager::new();
    
    let clip = AnimationClip {
        name: "test".into(),
        duration: 1.0,
        channels: vec![
            Channel {
                bone_index: 0,
                property: Property::Position,
                interpolation: Interpolation::Linear,
                keyframes: vec![
                    Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
                    Keyframe { time: 1.0, value: [1.0, 0.0, 0.0, 0.0] },
                ],
            },
        ],
    };
    
    let handle = assets.insert_animation(clip);
    let mut player = AnimationPlayer::new(handle);
    player.loop_mode = LoopMode::Loop;
    player.play();
    
    // Advance past duration
    player.update(1.5, 1.0);
    
    // Should have looped back
    assert!(player.time < 1.0);
    assert!(player.time >= 0.0);
}

// ============ Math Tests ============

#[test]
fn test_transform_compose() {
    let parent = Transform::from_xyz(10.0, 0.0, 0.0);
    let child = Transform::from_xyz(0.0, 5.0, 0.0);
    
    let composed = child.compose(&parent);
    
    assert!((composed.position.x - 10.0).abs() < 0.001);
    assert!((composed.position.y - 5.0).abs() < 0.001);
}

#[test]
fn test_transform_matrix_roundtrip() {
    let t = Transform {
        position: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    
    let matrix = t.to_matrix();
    let origin = matrix.transform_point3(Vec3::ZERO);
    
    assert!((origin - t.position).length() < 0.001);
}

#[test]
fn test_aabb_from_mesh() {
    let cube = Mesh::cube();
    let bounds = cube.bounds();
    
    assert!((bounds.min.x - (-0.5)).abs() < 0.001);
    assert!((bounds.max.x - 0.5).abs() < 0.001);
    assert!((bounds.size() - Vec3::splat(1.0)).length() < 0.001);
}

// ============ Camera Tests ============

#[test]
fn test_camera_view_projection() {
    let cam = Camera::perspective(45.0_f32.to_radians(), 0.1, 100.0);
    let transform = Transform::from_xyz(0.0, 0.0, 10.0);
    
    let vp = cam.view_projection(&transform, 16.0 / 9.0);
    
    // Origin should be projected to center-ish of screen
    let projected = vp.project_point3(Vec3::ZERO);
    assert!(projected.x.abs() < 1.0);
    assert!(projected.y.abs() < 1.0);
}

// ============ Light Tests ============

#[test]
fn test_light_types() {
    let dir = Light::directional(Vec3::ONE, 1.0);
    let point = Light::point(Vec3::ONE, 1.0, 10.0);
    let spot = Light::spot(Vec3::ONE, 1.0, 0.5, 20.0);
    let ambient = Light::ambient(Vec3::ONE, 0.2);
    
    assert!(dir.casts_shadows());
    assert!(point.casts_shadows());
    assert!(spot.casts_shadows());
    assert!(!ambient.casts_shadows());
}

// ============ PBR Material Tests ============

#[test]
fn test_material_uniform_conversion() {
    let mat = PbrMaterial {
        albedo: [0.8, 0.2, 0.1, 1.0],
        metallic: 0.9,
        roughness: 0.3,
        emissive: [0.1, 0.1, 0.1],
        ..Default::default()
    };
    
    let uniform = mat.to_uniform();
    
    assert_eq!(uniform.albedo, [0.8, 0.2, 0.1, 1.0]);
    assert_eq!(uniform.metallic_roughness[0], 0.9);
    assert_eq!(uniform.metallic_roughness[1], 0.3);
}
