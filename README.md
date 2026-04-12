# 🎞️ bevy_flipbook

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-232326)](https://bevyengine.org)

GPU-driven vertex animation texture (VAT) playback for [Bevy](https://bevyengine.org), designed for use with [OpenVAT](https://github.com/sharpen3d/openvat)-baked assets.

Bake your Blender animations into a single texture, load the mesh once, and let the vertex shader handle the rest. No skinning, no bones, no per-frame CPU work. Multiple entities can share one material while each plays a different animation clip via per-instance GPU slots.

## Features

- **Vertex animation textures** - positions and normals sampled from a single EXR texture in the vertex shader
- **Per-instance animation slots** - each mesh entity gets its own slot in a GPU storage buffer, enabling different clips on the same material
- **Automatic instancing** - many entities share one `VatMaterial`, one draw call
- **Runtime clip switching** - swap animation clips at any time through `VatHandler`
- **Slot reclamation** - slots are automatically freed when entities are despawned (via component hooks)
- **OpenVAT remap_info parsing** - reads the `*-remap_info.json` metadata exported by OpenVAT
- **Frame interpolation** - smooth blending between frames in the shader

## Quick Start

```rust
use bevy::prelude::*;
use bevy::pbr::ExtendedMaterial;
use bevy::render::storage::ShaderStorageBuffer;
use bevy_flipbook::{
    VatBundle, VatHandler, VatMaterial, VatMaterialExtension,
    VatPlugin, VatSettings, remap_info::RemapInfo,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VatPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, assign_materials)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // Load your GLTF mesh (rest pose) and the baked VAT texture
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fox.glb")),
    ));

    let vat_texture = asset_server.load("models/fox_vat.exr");

    // Parse OpenVAT remap info for bounds and clip metadata
    let remap: RemapInfo = RemapInfo::from_json(
        include_str!("assets/models/fox-remap_info.json")
    ).unwrap();

    let os = &remap.os_remap;
    let slots = buffers.add(ShaderStorageBuffer::new(&[0u8; 4], default()));

    let material = vat_materials.add(ExtendedMaterial {
        base: StandardMaterial::default(),
        extension: VatMaterialExtension::new(
            vat_texture,
            VatSettings {
                bounds_min: Vec3::from(os.min),
                bounds_max: Vec3::from(os.max),
                frame_count: os.frames,
                y_resolution: os.frames as f32 * 2.0,
                fps: 30.0,
            },
            slots,
        ),
    });

    // Store the material handle for later use
    commands.insert_resource(MyMaterial(material));
}

#[derive(Resource)]
struct MyMaterial(Handle<VatMaterial>);

fn assign_materials(
    mut commands: Commands,
    query: Query<Entity, -dded<MeshMaterial3d<StandardMaterial>>>,
    mat: Res<MyMaterial>,
    remap: Res<MyRemapInfo>,
    mut handler: ResMut<VatHandler>,
) {
    for entity in &query {
        // Allocate a slot and set the initial clip
        let slot_id = handler.allocate_slot(mat.0.clone());
        handler.update_slot(mat.0.clone(), slot_id, 0.0, clip);

        // Replace the GLTF material and attach the VAT bundle
        commands.entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                MeshMaterial3d(mat.0.clone()),
                VatBundle::new(slot_id),
            ));
    }
}
```

## How It Works

### Pipeline

```
Blender + OpenVAT
    ├─ mesh.glb          (rest-pose geometry with UV1 = VAT column)
    ├─ mesh_vat.exr      (positions top half, normals bottom half)
    └─ mesh-remap_info.json  (bounding box, clip ranges, FPS)
         │
         ▼
    bevy_flipbook
    ├─ RemapInfo          parses the JSON metadata
    ├─ VatMaterialExtension   binds texture + settings + slot buffer
    ├─ VatHandler         allocates slots, updates clip parameters
    └─ vat.wgsl           vertex shader samples the texture per-instance
```

### Slot System

Each mesh entity owns a **slot index** stored as a `MeshTag`. The vertex shader reads `slots[tag]` to get the entity's current clip parameters (start frame, frame count, time offset). This means hundreds of entities can share a single material/draw-call while independently playing different animations.

Slots are managed through `VatHandler`:

```rust
// Allocate
let slot_id = handler.allocate_slot(material_handle.clone());

// Set animation clip
handler.update_slot(material_handle.clone(), slot_id, time.elapsed_secs(), clip);
```

The `VatBundle` convenience bundle inserts both `MeshTag(slot_id)` and `VatMarker { slot_id }`. The `VatMarker` component has an `on_remove` hook that automatically returns the slot to the free list when the entity is despawned.

## Examples

Run examples with:

```sh
cargo run --example <name>
```

| Example | Description |
|---------|-------------|
| `fox` | Single fox with keyboard-driven clip switching (1/2/3) |
| `fox_grid` | 3×3 grid of foxes on one material, each cycling a different clip |
| `fox_reclaim` | Despawn/respawn with Space to verify slot reclamation |

## Asset Preparation

1. Install the [OpenVAT](https://github.com/sharpen3d/openvat) Blender add-on
2. Bake your animated mesh — this produces a `.glb`, a `_vat.exr`, and a `-remap_info.json`
3. Place all three files in your Bevy `assets/` directory

## Compatibility

| bevy_flipbook | Bevy |
|:---:|:---:|
| 0.1 | 0.18 |

## License

Licensed under the [MIT License](LICENSE).
