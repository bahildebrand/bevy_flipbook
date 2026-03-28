use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_openvat::{VatMaterial, VatMaterialExtension, VatPlugin, VatSettings};

#[derive(Resource)]
struct FoxMaterial(Handle<VatMaterial>);

// Fox animation clips from fox-remap_info.json
#[derive(Clone, Copy)]
enum FoxClip { Survey, Walk, Run }

impl FoxClip {
    fn start_frame(self) -> f32 {
        // Skip NLA overlap frames (82 and 99 are blended transition frames)
        match self { Self::Survey => 0.0, Self::Walk => 83.0, Self::Run => 100.0 }
    }
    fn frame_count(self) -> f32 {
        match self { Self::Survey => 82.0, Self::Walk => 16.0, Self::Run => 28.0 }
    }
    fn name(self) -> &'static str {
        match self { Self::Survey => "Survey (1)", Self::Walk => "Walk (2)", Self::Run => "Run (3)" }
    }
}

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
        .add_systems(Update, (replace_materials, orbit_camera, switch_clip))
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
                bounds_min: Vec3::new(-8.0, -46.2, -18.5),
                bounds_max: Vec3::new(55.1, 52.0, 54.7),
                // remap_info "Frames": 128, texture height = 256 (pos + normals)
                frame_count: 128,
                y_resolution: 256.0,
                fps: 30.0,
                current_time: 0.0,
                clip_start_frame: 0.0,
                clip_frame_count: 82.0,
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
///   1 / 2 / 3       — switch animation clip (Survey / Walk / Run)
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

fn switch_clip(
    keys: Res<ButtonInput<KeyCode>>,
    fox_material: Res<FoxMaterial>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
) {
    let clip = if keys.just_pressed(KeyCode::Digit1) {
        Some(FoxClip::Survey)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(FoxClip::Walk)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(FoxClip::Run)
    } else {
        None
    };

    if let Some(clip) = clip {
        if let Some(mat) = vat_materials.get_mut(&fox_material.0) {
            mat.extension.settings.clip_start_frame = clip.start_frame();
            mat.extension.settings.clip_frame_count = clip.frame_count();
            mat.extension.settings.current_time = 0.0;
            info!("Switched to clip: {}", clip.name());
        }
    }
}
