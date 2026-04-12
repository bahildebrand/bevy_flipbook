use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
};

use crate::remap_info::RemapInfo;

/// Convenience alias for the full extended material type.
pub type VatMaterial = ExtendedMaterial<StandardMaterial, VatMaterialExtension>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct VatMaterialExtension {
    #[texture(100)]
    #[sampler(101)]
    pub vat_texture: Handle<Image>,

    #[uniform(102)]
    pub settings: VatSettings,

    #[storage(103, read_only)]
    pub slots: Handle<ShaderStorageBuffer>,
}

impl VatMaterialExtension {
    pub fn new(
        vat_texture: Handle<Image>,
        settings: VatSettings,
        slots: Handle<ShaderStorageBuffer>,
    ) -> Self {
        Self {
            vat_texture,
            settings,
            slots,
        }
    }
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
    /// Global time when the current clip started — shader computes elapsed as globals.time - time_offset.
    pub time_offset: f32,
    pub clip_start_frame: f32,
    pub clip_frame_count: f32,
}

impl From<RemapInfo> for VatSettings {
    fn from(info: RemapInfo) -> Self {
        let os = &info.os_remap;
        Self {
            bounds_min: Vec3::from(os.min),
            bounds_max: Vec3::from(os.max),
            frame_count: os.frames,
            y_resolution: os.frames as f32 * 2.0, // pos+normals in one texture
            fps: info
                .clips_ordered()
                .into_iter()
                .next()
                .expect("remap_info has no animations")
                .1
                .framerate,
            time_offset: 0.0,
            clip_start_frame: 0.0,
            clip_frame_count: os.frames as f32, // default to entire range if no clips
        }
    }
}
