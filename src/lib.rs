mod material;
pub mod remap_info;
mod slot;

use bevy::{prelude::*, shader::ShaderRef};
pub use material::{VatMaterial, VatMaterialExtension, VatSettings};

pub fn vat_vertex_shader() -> ShaderRef {
    "shaders/vat.wgsl".into()
}

pub struct VatPlugin;

impl Plugin for VatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VatMaterial>::default());
    }
}
