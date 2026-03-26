//! Basic example: Create entities and run a headless game loop.
//!
//! Run with: cargo run --example basic

use tesseract::prelude::*;

fn main() {
    println!("Tesseract Basic Example");
    println!("=======================\n");

    // Create engine with default config
    let mut engine = Engine::new(WindowConfig {
        title: "Basic Example".into(),
        width: 800,
        height: 600,
        vsync: true,
    });

    // Create meshes (stored for future rendering)
    let _cube_mesh = engine.assets.insert_mesh(Mesh::cube());
    let _plane_mesh = engine.assets.insert_mesh(Mesh::plane(10.0));

    // Create materials (stored for future rendering)
    let _red_material = engine.assets.insert_material(PbrMaterial::with_albedo([1.0, 0.2, 0.2, 1.0]));
    let _gray_material = engine.assets.insert_material(PbrMaterial::with_albedo([0.5, 0.5, 0.5, 1.0]));

    // Spawn a rotating cube
    let cube = engine.world.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        GlobalTransform::default(),
        Name::new("Cube"),
        Visibility::default(),
    ));
    println!("Created cube entity: {:?}", cube);

    // Spawn a ground plane
    let ground = engine.world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Name::new("Ground"),
        Visibility::default(),
    ));
    println!("Created ground entity: {:?}", ground);

    // Spawn a camera
    let camera = engine.world.spawn((
        Transform::from_xyz(0.0, 3.0, 8.0).looking_at(Vec3::ZERO),
        Camera::perspective(45.0_f32.to_radians(), 0.1, 100.0),
        Name::new("Main Camera"),
    ));
    println!("Created camera entity: {:?}", camera);

    // Spawn a directional light
    let light = engine.world.spawn((
        Transform::from_xyz(5.0, 10.0, 5.0),
        Light::directional(Vec3::ONE, 1.0),
        Name::new("Sun"),
    ));
    println!("Created light entity: {:?}", light);

    println!("\nScene created with {} entities", engine.world.len());
    println!("Running 100 frames...\n");

    // Run headless for 100 frames
    let mut frame = 0;
    engine.run_headless(
        |world, _assets, dt| {
            // Rotate the cube
            if let Some(mut transform) = world.get_mut::<Transform>(cube) {
                transform.rotation *= Quat::from_rotation_y(dt * 2.0);
            }
            frame += 1;
        },
        100,
    );

    println!("Completed {} frames", frame);
    println!("Final cube rotation: {:?}", 
        engine.world.get::<Transform>(cube).map(|t| t.rotation));
}
