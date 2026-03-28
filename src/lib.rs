use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

pub struct VatPlugin;

impl Plugin for VatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VatMaterial>::default())
            .add_systems(Update, tick_vat_time);
    }
}

fn tick_vat_time(
    time: Res<Time>,
    mut vat_materials: ResMut<Assets<VatMaterial>>,
) {
    for (_, mat) in vat_materials.iter_mut() {
        mat.extension.settings.current_time += time.delta_secs();
    }
}

/// Convenience alias for the full extended material type.
pub type VatMaterial = ExtendedMaterial<StandardMaterial, VatMaterialExtension>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct VatMaterialExtension {
    #[texture(100)]
    #[sampler(101)]
    pub vat_texture: Handle<Image>,

    #[uniform(102)]
    pub settings: VatSettings,
}

impl MaterialExtension for VatMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        "shaders/vat.wgsl".into()
    }
}

#[derive(ShaderType, Debug, Clone)]
pub struct VatSettings {
    pub bounds_min: Vec3,
    /// Total animation frames from remap_info "Frames" field.
    pub frame_count: u32,
    pub bounds_max: Vec3,
    /// Actual texture pixel height (frame_count * 2 for pos+normals in one texture).
    pub y_resolution: f32,
    pub fps: f32,
    pub current_time: f32,
    pub clip_start_frame: f32,
    pub clip_frame_count: f32,
}
