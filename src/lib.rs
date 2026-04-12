mod material;
pub mod remap_info;
mod slot;

use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::render::storage::ShaderStorageBuffer;
pub use material::{VatMaterial, VatMaterialExtension, VatSettings};

use crate::slot::VatSlotBuffers;
use crate::{remap_info::AnimationClip, slot::VatSlot};

use bevy::{prelude::*, shader::ShaderRef};

pub fn vat_vertex_shader() -> ShaderRef {
    "shaders/vat.wgsl".into()
}

pub struct VatPlugin;

impl Plugin for VatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VatMaterial>::default())
            .init_resource::<VatHandler>()
            .add_systems(
                PostUpdate,
                update_slot_buffers.run_if(resource_changed::<VatHandler>),
            );

        app.world_mut()
            .register_component_hooks::<VatSlotComponent>()
            .on_remove(|mut world: DeferredWorld, ctx: HookContext| {
                let Some(slot) = world.get::<VatSlotComponent>(ctx.entity).cloned() else {
                    return;
                };
                if let Some(mut handler) = world.get_resource_mut::<VatHandler>() {
                    handler.slot_buffers.free_slot(slot.mat, slot.slot_id);
                }
            });
    }
}

/// Attach this alongside [`bevy::mesh::MeshTag`] to automatically reclaim the
/// slot when the entity is despawned or the component is removed.
#[derive(Component, Clone)]
pub struct VatSlotComponent {
    pub mat: Handle<VatMaterial>,
    pub slot_id: u32,
}

#[derive(Resource, Default)]
pub struct VatHandler {
    slot_buffers: VatSlotBuffers,
}

impl VatHandler {
    pub fn allocate_slot(&mut self, mat_handle: Handle<VatMaterial>) -> u32 {
        self.slot_buffers.allocate_slot(mat_handle)
    }

    pub fn free_slot(&mut self, mat_handle: Handle<VatMaterial>, slot_id: u32) {
        self.slot_buffers.free_slot(mat_handle, slot_id);
    }

    pub fn update_slot(
        &mut self,
        mat_handle: Handle<VatMaterial>,
        slot_id: u32,
        time_offset: f32,
        animation_clip: AnimationClip,
    ) {
        self.slot_buffers
            .update_slot(mat_handle, slot_id, time_offset, animation_clip);
    }

    pub(crate) fn dirty_buffer_iter(
        &mut self,
    ) -> impl Iterator<Item = (&Handle<VatMaterial>, &Vec<VatSlot>)> {
        self.slot_buffers.dirty_buffer_iter()
    }
}

fn update_slot_buffers(
    mut vat_handler: ResMut<VatHandler>,
    mut vat_mats: ResMut<Assets<VatMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (handle, buffer) in vat_handler.dirty_buffer_iter() {
        if let Some(material) = vat_mats.get_mut(handle) {
            let storage_buffer = ShaderStorageBuffer::from(buffer.clone());
            let buffer_handle = buffers.add(storage_buffer);

            material.extension.slots = buffer_handle;
        }
    }
}
