mod allocator;

use std::collections::HashMap;

use crate::{remap_info::AnimationClip, slot::allocator::SlotAllocator};

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::ShaderType,
};

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
    pub fn allocate_slot(
        &mut self,
        mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>,
    ) -> u32 {
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
        rate: f32,
    ) {
        let buffer = self.buffers.get_mut(&mat_handle).unwrap();
        let slot = buffer.buffer.get_mut(slot_id as usize).unwrap();

        slot.time_offset = time_offset;
        slot.clip_start_frame = animation_clip.start_frame as f32;
        slot.clip_frame_count = animation_clip.frame_count() as f32;
        slot.rate = rate;

        buffer.dirty = true;
    }

    pub fn free_slot(
        &mut self,
        mat_handle: Handle<ExtendedMaterial<StandardMaterial, E>>,
        slot_id: u32,
    ) {
        if let Some(buffer) = self.buffers.get_mut(&mat_handle) {
            buffer.allocator.free(slot_id);
        }
    }

    pub(crate) fn has_dirty(&self) -> bool {
        self.buffers.values().any(|b| b.dirty)
    }

    pub fn dirty_buffer_iter(
        &mut self,
    ) -> impl Iterator<
        Item = (
            &Handle<ExtendedMaterial<StandardMaterial, E>>,
            &Vec<VatSlot>,
            &mut bool,
        ),
    > {
        self.buffers
            .iter_mut()
            .filter(|(_, buffer)| buffer.dirty)
            .map(|(handle, buffer)| (handle, &buffer.buffer, &mut buffer.dirty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::VatMaterialExtension;
    use bevy::pbr::ExtendedMaterial;
    use bevy::prelude::{Handle, StandardMaterial};

    type TestMat = ExtendedMaterial<StandardMaterial, VatMaterialExtension>;

    fn make_clip(start: u32, end: u32) -> crate::remap_info::AnimationClip {
        crate::remap_info::AnimationClip {
            start_frame: start,
            end_frame: end,
            framerate: 30.0,
            looping: true,
        }
    }

    #[test]
    fn has_dirty_initially_false() {
        let buffers = VatSlotBuffers::<VatMaterialExtension>::default();
        assert!(!buffers.has_dirty());
    }

    #[test]
    fn has_dirty_true_after_allocate() {
        let mut buffers = VatSlotBuffers::<VatMaterialExtension>::default();
        buffers.allocate_slot(Handle::<TestMat>::default());
        assert!(buffers.has_dirty());
    }

    #[test]
    fn has_dirty_clears_when_caller_acknowledges() {
        let mut buffers = VatSlotBuffers::<VatMaterialExtension>::default();
        buffers.allocate_slot(Handle::<TestMat>::default());
        buffers
            .dirty_buffer_iter()
            .for_each(|(_, _, dirty)| *dirty = false);
        assert!(!buffers.has_dirty());
    }

    #[test]
    fn has_dirty_remains_if_caller_does_not_acknowledge() {
        let mut buffers = VatSlotBuffers::<VatMaterialExtension>::default();
        buffers.allocate_slot(Handle::<TestMat>::default());
        // Consume the iterator without clearing dirty (simulates a failed upload).
        buffers.dirty_buffer_iter().for_each(|_| ());
        assert!(
            buffers.has_dirty(),
            "dirty must stay set so the upload retries next frame"
        );
    }

    #[test]
    fn has_dirty_remains_if_iter_dropped_unconsumed() {
        let mut buffers = VatSlotBuffers::<VatMaterialExtension>::default();
        buffers.allocate_slot(Handle::<TestMat>::default());
        let _ = buffers.dirty_buffer_iter(); // dropped without polling
        assert!(buffers.has_dirty());
    }

    #[test]
    fn has_dirty_true_after_update_slot() {
        let mut buffers = VatSlotBuffers::<VatMaterialExtension>::default();
        let handle = Handle::<TestMat>::default();
        let slot_id = buffers.allocate_slot(handle.clone());
        buffers
            .dirty_buffer_iter()
            .for_each(|(_, _, dirty)| *dirty = false);
        assert!(!buffers.has_dirty(), "should be clean after flush");

        buffers.update_slot(handle, slot_id, 0.0, make_clip(0, 10));
        assert!(buffers.has_dirty(), "should be dirty after update_slot");
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
    pub rate: f32,
}

impl VatSlot {
    pub fn new(time_offset: f32, clip_start_frame: f32, clip_frame_count: f32, rate: f32) -> Self {
        Self {
            time_offset,
            clip_start_frame,
            clip_frame_count,
            rate,
        }
    }
}
