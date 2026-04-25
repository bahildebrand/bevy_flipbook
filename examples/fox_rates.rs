use bevy::{pbr::ExtendedMaterial, prelude::*, render::storage::ShaderStorageBuffer};
use bevy_flipbook::{
    VatBundle, VatHandler, VatMaterial, VatMaterialExtension, VatPlugin, VatSettings,
    remap_info::RemapInfo,
};

const REMAP_INFO_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/models/fox-remap_info.json"
));

const Y_RESOLUTION_MULTIPLIER: f32 = 2.0;

const FOX_SPACING: f32 = 140.0;
const RATES: [f32; 3] = [0.5, 1.0, 1.5];

#[derive(Resource)]
struct FoxMaterial(Handle<VatMaterial>);

#[derive(Resource)]
struct FoxRemapInfo(RemapInfo);

#[derive(Component)]
struct FoxInstance {
    rate: f32,
}

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

    let half_extent = (RATES.len() as f32 - 1.0) * FOX_SPACING * 0.5;
    for (i, &rate) in RATES.iter().enumerate() {
        let x = i as f32 * FOX_SPACING - half_extent;
        commands.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb"))),
            Transform::from_translation(Vec3::new(x, 0.0, 0.0)),
            FoxInstance { rate },
        ));
    }

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
        Transform::from_translation(Vec3::new(0.0, 120.0, 300.0))
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
    let first = remap_info
        .0
        .clips_ordered()
        .into_iter()
        .next()
        .expect("remap_info has no animations");

    for entity in &query {
        let Some(fox) = find_ancestor(&fox_instances, &parents, entity) else {
            continue;
        };

        let slot_id = vat_handler.allocate_slot(fox_material.0.clone());
        vat_handler.update_slot(fox_material.0.clone(), slot_id, 0.0, first.1.clone(), fox.rate);

        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                MeshMaterial3d(fox_material.0.clone()),
                VatBundle::<VatMaterialExtension>::new(slot_id),
            ));
    }
}

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
