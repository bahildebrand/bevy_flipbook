use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_flipbook::{
    remap_info::RemapInfo, VatMaterial, VatMaterialExtension, VatPlugin, VatSettings,
};

const REMAP_INFO_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/fox-remap_info.json"));

// Texture height is frame_count * 2 because positions and normals are packed into one texture.
const Y_RESOLUTION_MULTIPLIER: f32 = 2.0;

#[derive(Resource)]
struct FoxMaterial(Handle<VatMaterial>);

#[derive(Resource)]
struct FoxRemapInfo(RemapInfo);

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.3,
            pitch: 0.4,
            radius: 120.0,
            focus: Vec3::new(0.0, 10.0, 0.0),
        }
    }
}

fn main() {
    let remap_info = RemapInfo::from_json(REMAP_INFO_JSON).expect("failed to parse remap_info.json");

    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            }),
            VatPlugin,
        ))
        .insert_resource(FoxRemapInfo(remap_info))
        .add_systems(Startup, setup)
        .add_systems(Update, (replace_materials, orbit_camera, switch_clip))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
    remap_info: Res<FoxRemapInfo>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb")),
    ));

    let vat_texture = asset_server.load("models/fox_vat.exr");

    let os = &remap_info.0.os_remap;
    // Start on the first clip ordered by start_frame.
    let first = remap_info
        .0
        .clips_ordered()
        .into_iter()
        .next()
        .expect("remap_info has no animations");

    let material = vat_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.5),
            ..default()
        },
        extension: VatMaterialExtension {
            vat_texture,
            settings: VatSettings {
                bounds_min: Vec3::from(os.min),
                bounds_max: Vec3::from(os.max),
                frame_count: os.frames,
                y_resolution: os.frames as f32 * Y_RESOLUTION_MULTIPLIER,
                fps: first.1.framerate,
                time_offset: 0.0,
                clip_start_frame: first.1.start_frame as f32,
                clip_frame_count: first.1.frame_count() as f32,
            },
        },
    });

    commands.insert_resource(FoxMaterial(material));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
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
///   1 / 2 / 3 / … — switch to animation clip N (ordered by start_frame)
fn orbit_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok((mut orbit, mut transform)) = query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    if keys.pressed(KeyCode::ArrowLeft) {
        orbit.yaw += 1.5 * dt;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        orbit.yaw -= 1.5 * dt;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        orbit.pitch = (orbit.pitch + 1.5 * dt).clamp(-1.4, 1.4);
    }
    if keys.pressed(KeyCode::ArrowDown) {
        orbit.pitch = (orbit.pitch - 1.5 * dt).clamp(-1.4, 1.4);
    }
    if keys.pressed(KeyCode::KeyZ) {
        orbit.radius = (orbit.radius - 60.0 * dt).clamp(10.0, 500.0);
    }
    if keys.pressed(KeyCode::KeyX) {
        orbit.radius = (orbit.radius + 60.0 * dt).clamp(10.0, 500.0);
    }

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
    time: Res<Time>,
    fox_material: Res<FoxMaterial>,
    remap_info: Res<FoxRemapInfo>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
) {
    const DIGIT_KEYS: &[KeyCode] = &[
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    let clips = remap_info.0.clips_ordered();
    let selected = DIGIT_KEYS
        .iter()
        .enumerate()
        .find(|(_, key)| keys.just_pressed(**key))
        .and_then(|(i, _)| clips.get(i).copied());

    if let Some((name, clip)) = selected {
        if let Some(mat) = vat_materials.get_mut(&fox_material.0) {
            mat.extension.settings.clip_start_frame = clip.start_frame as f32;
            mat.extension.settings.clip_frame_count = clip.frame_count() as f32;
            mat.extension.settings.fps = clip.framerate;
            mat.extension.settings.time_offset = time.elapsed_secs();
            info!("Switched to clip: {name}");
        }
    }
}
