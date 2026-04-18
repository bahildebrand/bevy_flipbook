mod allocator;

use std::collections::HashMap;

use crate::{remap_info::AnimationClip, slot::allocator::SlotAllocator};

use bevy::{pbr::{ExtendedMaterial, MaterialExtension}, prelude::*, render::render_resource::ShaderType};

pub(crate) struct VatSlotBuffers<E: MaterialExtension> {
    buffers: HashMap<Handle<ExtendedMaterial<StandardMaterial, E>>, VatSlotBuffer>,
}

impl<E: MaterialExtension> Default for VatSlotBuffers<E> {
    fn default() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }
}

impl<E: MaterialExtension> VatSlotBuffers<E> {
    pub fn allocate_slot(&mut self, mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>) -> u32 {
        let buffer = self.buffers.entry(mat_handle).or_default();

        let slot_id = buffer.allocator.allocate();

        if slot_id as usize >= buffer.buffer.len() {
            buffer.buffer.push(VatSlot::default());
        }

        buffer.dirty = true;

        slot_id
    }

    // TODO: actually handle errors here
    pub fn update_slot(
        &mut self,
        mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>,
        slot_id: u32,
        time_offset: f32,
        animation_clip: AnimationClip,
    ) {
        let buffer = self.buffers.get_mut(&mat_handle).unwrap();
        let slot = buffer.buffer.get_mut(slot_id as usize).unwrap();

        slot.time_offset = time_offset;
        slot.clip_start_frame = animation_clip.start_frame as f32;
        slot.clip_frame_count = animation_clip.frame_count() as f32;

        buffer.dirty = true;
    }

    pub fn free_slot(&mut self, mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>, slot_id: u32) {
        if let Some(buffer) = self.buffers.get_mut(&mat_handle) {
            buffer.allocator.free(slot_id);
        }
    }

    pub fn dirty_buffer_iter(
        &mut self,
    ) -> impl Iterator<Item = (&Handle<ExtendedMaterial<StandardMaterial, E>>, &Vec<VatSlot>)> {
        self.buffers
            .iter_mut()
            .filter(|(_, buffer)| buffer.dirty)
            .map(|(handle, buffer)| {
                buffer.dirty = false;
                (handle, &buffer.buffer)
            })
    }
}

#[derive(Default)]
struct VatSlotBuffer {
    buffer: Vec<VatSlot>,
    allocator: SlotAllocator,
    dirty: bool,
}

#[derive(Default, ShaderType, Clone)]
pub(crate) struct VatSlot {
    pub time_offset: f32,
    pub clip_start_frame: f32,
    pub clip_frame_count: f32,
    _padding: u32,
}

impl VatSlot {
    pub fn new(time_offset: f32, clip_start_frame: f32, clip_frame_count: f32) -> Self {
        Self {
            time_offset,
            clip_start_frame,
            clip_frame_count,
            _padding: 0,
        }
    }
}
