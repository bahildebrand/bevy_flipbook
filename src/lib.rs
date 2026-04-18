mod material;
pub mod remap_info;
mod slot;

use std::marker::PhantomData;

use crate::slot::VatSlotBuffers;
use crate::{remap_info::AnimationClip, slot::VatSlot};
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer};
pub use material::{VatMaterial, VatMaterialExtension, VatSettings, VatSlotAccess};

use bevy::{mesh::MeshTag, prelude::*, shader::ShaderRef};

pub fn vat_vertex_shader() -> ShaderRef {
    "shaders/vat.wgsl".into()
}

/// Plugin that registers VAT rendering for a given [`MaterialExtension`] `E`.
///
/// `E` must implement [`VatSlotAccess`] so the plugin can write updated slot
/// buffers back into the material. Use the default type parameter when you
/// don't need a custom extension:
///
/// ```rust,no_run
/// app.add_plugins(VatPlugin::default());
/// ```
///
/// For a custom extension:
///
/// ```rust,no_run
/// app.add_plugins(VatPlugin::<MyExtension>::default());
/// ```
pub struct VatPlugin<E: MaterialExtension + VatSlotAccess = VatMaterialExtension> {
    _phantom: PhantomData<E>,
}

impl<E: MaterialExtension + VatSlotAccess> Default for VatPlugin<E> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: MaterialExtension + VatSlotAccess> Plugin for VatPlugin<E>
where
    ExtendedMaterial<StandardMaterial, E>: Material,
    <ExtendedMaterial<StandardMaterial, E> as AsBindGroup>::Data:
        PartialEq + Eq + std::hash::Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, E>>::default())
            .init_resource::<VatHandler<E>>()
            .add_systems(
                PostUpdate,
                update_slot_buffers::<E>.run_if(resource_changed::<VatHandler<E>>),
            );

        app.world_mut()
            .register_component_hooks::<VatMarker<E>>()
            .on_remove(|mut world: DeferredWorld, ctx: HookContext| {
                let slot_id = world.get::<VatMarker<E>>(ctx.entity).map(|m| m.slot_id);
                let mat = world
                    .get::<MeshMaterial3d<ExtendedMaterial<StandardMaterial, E>>>(ctx.entity)
                    .map(|m| m.0.clone());

                if let (Some(slot_id), Some(mat), Some(mut handler)) =
                    (slot_id, mat, world.get_resource_mut::<VatHandler<E>>())
                {
                    handler.slot_buffers.free_slot(mat, slot_id);
                }
            });
    }
}

/// Convenience bundle combining [`bevy::mesh::MeshTag`] and [`VatMarker`].
/// Insert this on mesh entities after allocating a slot via [`VatHandler::allocate_slot`].
#[derive(Bundle)]
pub struct VatBundle<E: MaterialExtension + Send + Sync + 'static = VatMaterialExtension> {
    pub mesh_tag: MeshTag,
    pub marker: VatMarker<E>,
}

impl<E: MaterialExtension> VatBundle<E> {
    pub fn new(slot_id: u32) -> Self {
        Self {
            mesh_tag: MeshTag(slot_id),
            marker: VatMarker::new(slot_id),
        }
    }
}

/// Marker component that tracks which VAT slot an entity owns. The slot is
/// freed when the entity is despawned or the component is removed.
/// The material handle is read from
/// [`MeshMaterial3d<ExtendedMaterial<StandardMaterial, E>>`] on the same entity.
#[derive(Component, Clone)]
pub struct VatMarker<E: MaterialExtension + Send + Sync + 'static = VatMaterialExtension> {
    pub slot_id: u32,
    _phantom: PhantomData<E>,
}

impl<E: MaterialExtension> VatMarker<E> {
    fn new(slot_id: u32) -> Self {
        Self {
            slot_id,
            _phantom: PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct VatHandler<E: MaterialExtension = VatMaterialExtension> {
    slot_buffers: VatSlotBuffers<E>,
}

impl<E: MaterialExtension> Default for VatHandler<E> {
    fn default() -> Self {
        Self {
            slot_buffers: VatSlotBuffers::default(),
        }
    }
}

impl<E: MaterialExtension> VatHandler<E> {
    pub fn allocate_slot(
        &mut self,
        mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>,
    ) -> u32 {
        self.slot_buffers.allocate_slot(mat_handle)
    }

    pub fn update_slot(
        &mut self,
        mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>,
        slot_id: u32,
        time_offset: f32,
        animation_clip: AnimationClip,
    ) {
        self.slot_buffers
            .update_slot(mat_handle, slot_id, time_offset, animation_clip);
    }

    pub(crate) fn dirty_buffer_iter(
        &mut self,
    ) -> impl Iterator<Item = (&Handle<ExtendedMaterial<StandardMaterial, E>>, &Vec<VatSlot>)> {
        self.slot_buffers.dirty_buffer_iter()
    }
}

fn update_slot_buffers<E: MaterialExtension + VatSlotAccess>(
    mut vat_handler: ResMut<VatHandler<E>>,
    mut vat_mats: ResMut<Assets<ExtendedMaterial<StandardMaterial, E>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (handle, buffer) in vat_handler.dirty_buffer_iter() {
        if let Some(material) = vat_mats.get_mut(handle) {
            let storage_buffer = ShaderStorageBuffer::from(buffer.clone());
            let buffer_handle = buffers.add(storage_buffer);

            material.extension.set_slots(buffer_handle);
        }
    }
}
