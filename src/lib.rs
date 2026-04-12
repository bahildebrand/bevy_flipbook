mod material;
pub mod remap_info;
mod slot;

use crate::slot::VatSlotBuffers;
use crate::{remap_info::AnimationClip, slot::VatSlot};
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::render::storage::ShaderStorageBuffer;
pub use material::{VatMaterial, VatMaterialExtension, VatSettings};

use bevy::{mesh::MeshTag, prelude::*, shader::ShaderRef};

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
            .register_component_hooks::<VatMarker>()
            .on_remove(|mut world: DeferredWorld, ctx: HookContext| {
                let slot_id = world.get::<VatMarker>(ctx.entity).map(|m| m.slot_id);
                let mat = world
                    .get::<MeshMaterial3d<VatMaterial>>(ctx.entity)
                    .map(|m| m.0.clone());

                if let (Some(slot_id), Some(mat), Some(mut handler)) =
                    (slot_id, mat, world.get_resource_mut::<VatHandler>())
                {
                    handler.slot_buffers.free_slot(mat, slot_id);
                }
            });
    }
}

/// Convenience bundle combining [`bevy::mesh::MeshTag`] and [`VatMarker`].
/// Insert this on mesh entities after allocating a slot via [`VatHandler::allocate_slot`].
#[derive(Bundle)]
pub struct VatBundle {
    pub mesh_tag: MeshTag,
    pub marker: VatMarker,
}

impl VatBundle {
    pub fn new(slot_id: u32) -> Self {
        Self {
            mesh_tag: MeshTag(slot_id),
            marker: VatMarker { slot_id },
        }
    }
}

/// slot when the entity is despawned or the component is removed.
/// The material handle is read from [`MeshMaterial3d<VatMaterial>`] on the same entity.
#[derive(Component, Clone)]
pub struct VatMarker {
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
