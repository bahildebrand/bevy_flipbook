use bevy::{
    mesh::MeshTag,
    pbr::ExtendedMaterial,
    prelude::*,
    render::storage::ShaderStorageBuffer,
};
use bevy_flipbook::{
    remap_info::RemapInfo, VatHandler, VatMaterial, VatMaterialExtension, VatPlugin, VatSettings,
    VatSlotComponent,
};

const REMAP_INFO_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/fox-remap_info.json"));

const Y_RESOLUTION_MULTIPLIER: f32 = 2.0;

const GRID_SIZE: usize = 3;
const FOX_SPACING: f32 = 140.0;

#[derive(Resource)]
struct FoxMaterial(Handle<VatMaterial>);

#[derive(Resource)]
struct FoxRemapInfo(RemapInfo);

/// Marks a scene root with the index of the clip it should play.
#[derive(Component)]
struct FoxInstance {
    clip_index: usize,
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
        .add_systems(Update, replace_materials)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    remap_info: Res<FoxRemapInfo>,
) {
    let vat_texture = asset_server.load("models/fox_vat.exr");
    let os = &remap_info.0.os_remap;
    let clips = remap_info.0.clips_ordered();
    let first = clips.first().copied().expect("remap_info has no animations");

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

    let num_clips = clips.len();
    let half_extent = (GRID_SIZE as f32 - 1.0) * FOX_SPACING * 0.5;
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let clip_index = (row * GRID_SIZE + col) % num_clips;
            let x = col as f32 * FOX_SPACING - half_extent;
            let z = row as f32 * FOX_SPACING - half_extent;
            commands.spawn((
                SceneRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb")),
                ),
                Transform::from_translation(Vec3::new(x, 0.0, z)),
                FoxInstance { clip_index },
            ));
        }
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));

    let dist = half_extent + FOX_SPACING * 2.5;
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, dist * 0.8, dist))
            .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));
}

fn replace_materials(
    mut commands: Commands,
    query: Query<Entity, Added<MeshMaterial3d<StandardMaterial>>>,
    fox_instances: Query<&FoxInstance>,
    parents: Query<&ChildOf>,
    fox_material: Res<FoxMaterial>,
    remap_info: Res<FoxRemapInfo>,
    mut vat_handler: ResMut<VatHandler>,
) {
    let clips = remap_info.0.clips_ordered();

    for entity in &query {
        let Some(fox) = find_ancestor(&fox_instances, &parents, entity) else {
            continue;
        };

        let (_, clip) = clips[fox.clip_index % clips.len()];
        let slot_id = vat_handler.allocate_slot(fox_material.0.clone());
        vat_handler.update_slot(fox_material.0.clone(), slot_id, 0.0, clip.clone());

        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                MeshMaterial3d(fox_material.0.clone()),
                MeshTag(slot_id),
                VatSlotComponent { mat: fox_material.0.clone(), slot_id },
            ));
    }
}

/// Walk up the hierarchy from `entity` looking for a component of type `T`.
fn find_ancestor<'w, T: Component>(
    query: &'w Query<&T>,
    parents: &Query<&ChildOf>,
    entity: Entity,
) -> Option<&'w T> {
    if let Ok(component) = query.get(entity) {
        return Some(component);
    }
    parents
        .get(entity)
        .ok()
        .and_then(|child_of| find_ancestor(query, parents, child_of.parent()))
}
