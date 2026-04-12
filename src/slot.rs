mod allocator;

use std::collections::HashMap;

use crate::{VatMaterial, remap_info::AnimationClip, slot::allocator::SlotAllocator};

use bevy::{prelude::*, render::render_resource::ShaderType};

#[derive(Default)]
pub(crate) struct VatSlotBuffers {
    buffers: HashMap<Handle<VatMaterial>, VatSlotBuffer>,
}

impl VatSlotBuffers {
    pub fn allocate_slot(&mut self, mat_handle: Handle<VatMaterial>) -> u32 {
        let buffer = self.buffers.entry(mat_handle).or_default();

        let slot_id = buffer.allocator.allocate();

        let slot = VatSlot::default();
        buffer.buffer.push(slot);

        buffer.dirty = true;

        slot_id
    }

    // TODO: actually handle errors here
    pub fn update_slot(
        &mut self,
        mat_handle: Handle<VatMaterial>,
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

    pub fn dirty_buffer_iter(
        &mut self,
    ) -> impl Iterator<Item = (&Handle<VatMaterial>, &Vec<VatSlot>)> {
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
}

impl VatSlot {
    pub fn new(time_offset: f32, clip_start_frame: f32, clip_frame_count: f32) -> Self {
        Self {
            time_offset,
            clip_start_frame,
            clip_frame_count,
        }
    }
}
