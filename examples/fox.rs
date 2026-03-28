use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_openvat::{VatMaterial, VatMaterialExtension, VatPlugin, VatSettings};

#[derive(Resource)]
struct FoxMaterial(Handle<VatMaterial>);

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self { yaw: 0.3, pitch: 0.4, radius: 120.0, focus: Vec3::new(0.0, 10.0, 0.0) }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            }),
            VatPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (replace_materials, orbit_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb")),
    ));

    let vat_texture = asset_server.load("models/fox_vat.exr");

    let material = vat_materials.add(ExtendedMaterial {
        base: StandardMaterial { base_color: Color::srgb(0.8, 0.7, 0.5), ..default() },
        extension: VatMaterialExtension {
            vat_texture,
            settings: VatSettings {
                bounds_min: Vec3::new(-9.0, -22.5, -4.2),
                bounds_max: Vec3::new(54.6, 20.0, 9.2),
                total_frames: 250.0,
                fps: 30.0,
                current_time: 0.0,
                clip_start_frame: 1.0,
                clip_frame_count: 82.0,
                _padding: 0.0,
            },
        },
    });

    commands.insert_resource(FoxMaterial(material));

    commands.spawn((
        DirectionalLight { illuminance: 10_000.0, shadows_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));

    let orbit = OrbitCamera::default();
    let t = orbit_to_transform(&orbit);
    commands.spawn((Camera3d::default(), t, orbit));
}

/// Controls:
///   Arrow Left/Right — orbit horizontally
///   Arrow Up/Down   — orbit vertically
///   Z / X           — zoom in / out
fn orbit_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok((mut orbit, mut transform)) = query.single_mut() else { return };
    let dt = time.delta_secs();

    if keys.pressed(KeyCode::ArrowLeft)  { orbit.yaw   += 1.5 * dt; }
    if keys.pressed(KeyCode::ArrowRight) { orbit.yaw   -= 1.5 * dt; }
    if keys.pressed(KeyCode::ArrowUp)    { orbit.pitch  = (orbit.pitch + 1.5 * dt).clamp(-1.4, 1.4); }
    if keys.pressed(KeyCode::ArrowDown)  { orbit.pitch  = (orbit.pitch - 1.5 * dt).clamp(-1.4, 1.4); }
    if keys.pressed(KeyCode::KeyZ)       { orbit.radius = (orbit.radius - 60.0 * dt).clamp(10.0, 500.0); }
    if keys.pressed(KeyCode::KeyX)       { orbit.radius = (orbit.radius + 60.0 * dt).clamp(10.0, 500.0); }

    *transform = orbit_to_transform(&orbit);
}

fn orbit_to_transform(orbit: &OrbitCamera) -> Transform {
    let rotation = Quat::from_rotation_y(orbit.yaw) * Quat::from_rotation_x(-orbit.pitch);
    let offset = rotation * Vec3::new(0.0, 0.0, orbit.radius);
    Transform::from_translation(orbit.focus + offset).looking_at(orbit.focus, Vec3::Y)
}

fn replace_materials(
    mut commands: Commands,
    query: Query<Entity, Added<MeshMaterial3d<StandardMaterial>>>,
    fox_material: Res<FoxMaterial>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(fox_material.0.clone()));
    }
}
