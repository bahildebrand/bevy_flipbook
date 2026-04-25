//! Demonstrates slot reclamation: press Space to despawn the fox, press Space again to respawn it.
//! The slot ID shown on screen should be the same after each respawn, proving the free list works.

use bevy::{pbr::ExtendedMaterial, prelude::*, render::storage::ShaderStorageBuffer};
use bevy_flipbook::{
    VatBundle, VatHandler, VatMarker, VatMaterial, VatMaterialExtension, VatPlugin, VatSettings,
    remap_info::RemapInfo,
};

const REMAP_INFO_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/models/fox-remap_info.json"
));

const Y_RESOLUTION_MULTIPLIER: f32 = 2.0;

#[derive(Resource)]
struct FoxMaterial(Handle<VatMaterial>);

#[derive(Resource)]
struct FoxRemapInfo(RemapInfo);

/// Root entity of the spawned fox scene, if any.
#[derive(Resource, Default)]
struct FoxScene(Option<Entity>);

/// Text node that shows the current slot ID.
#[derive(Component)]
struct SlotLabel;

fn main() {
    let remap_info =
        RemapInfo::from_json(REMAP_INFO_JSON).expect("failed to parse remap_info.json");

    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            }),
            VatPlugin::<VatMaterialExtension>::default(),
        ))
        .insert_resource(FoxRemapInfo(remap_info))
        .init_resource::<FoxScene>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (replace_materials, toggle_fox, update_slot_label).chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    remap_info: Res<FoxRemapInfo>,
    mut fox_scene: ResMut<FoxScene>,
) {
    // Spawn the fox immediately.
    let scene = commands
        .spawn(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb")),
        ))
        .id();
    fox_scene.0 = Some(scene);

    let vat_texture = asset_server.load("models/fox_vat.exr");

    let os = &remap_info.0.os_remap;
    let first = remap_info
        .0
        .clips_ordered()
        .into_iter()
        .next()
        .expect("remap_info has no animations");

    let slots = buffers.add(ShaderStorageBuffer::new(&[0u8; 4], default()));

    let material = vat_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.5),
            ..default()
        },
        extension: VatMaterialExtension::new(
            vat_texture,
            VatSettings {
                bounds_min: Vec3::from(os.min),
                bounds_max: Vec3::from(os.max),
                frame_count: os.frames,
                y_resolution: os.frames as f32 * Y_RESOLUTION_MULTIPLIER,
                fps: first.1.framerate,
            },
            slots,
        ),
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

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 150.0).looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));

    // UI
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        SlotLabel,
    ));
    commands.spawn((
        Text::new("Space — despawn / respawn fox"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn replace_materials(
    mut commands: Commands,
    query: Query<Entity, Added<MeshMaterial3d<StandardMaterial>>>,
    fox_material: Res<FoxMaterial>,
    remap_info: Res<FoxRemapInfo>,
    mut vat_handler: ResMut<VatHandler>,
) {
    let first = remap_info
        .0
        .clips_ordered()
        .into_iter()
        .next()
        .expect("remap_info has no animations");

    for entity in &query {
        let slot_id = vat_handler.allocate_slot(fox_material.0.clone());
        vat_handler.update_slot(fox_material.0.clone(), slot_id, 0.0, first.1.clone(), 1.0);
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                MeshMaterial3d(fox_material.0.clone()),
                VatBundle::<VatMaterialExtension>::new(slot_id),
            ));
        info!("Allocated slot {slot_id}");
    }
}

fn toggle_fox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut fox_scene: ResMut<FoxScene>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    match fox_scene.0.take() {
        Some(entity) => {
            info!("Despawning fox (entity {entity})");
            commands.entity(entity).despawn();
        }
        None => {
            let scene = commands
                .spawn(SceneRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb")),
                ))
                .id();
            info!("Spawned fox (entity {scene})");
            fox_scene.0 = Some(scene);
        }
    }
}

fn update_slot_label(
    fox_scene: Res<FoxScene>,
    markers: Query<&VatMarker>,
    mut label: Query<&mut Text, With<SlotLabel>>,
) {
    let Ok(mut text) = label.single_mut() else {
        return;
    };

    if fox_scene.0.is_none() {
        **text = "Fox despawned — press Space to respawn".into();
        return;
    }

    let slot_ids: Vec<u32> = markers.iter().map(|m| m.slot_id).collect();
    if slot_ids.is_empty() {
        **text = "Loading…".into();
    } else {
        let ids: Vec<String> = slot_ids.iter().map(|id| id.to_string()).collect();
        **text = format!("Slot IDs: [{}]", ids.join(", "));
    }
}
