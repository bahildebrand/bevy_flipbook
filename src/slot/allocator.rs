#![allow(unused)]
//! Handles allocation of VAT buffer slots

#[derive(Debug, Default)]
pub(crate) struct SlotAllocator {
    next_slot: u32,
}

impl SlotAllocator {
    // TODO: Maintain free list and reclaim slots
    pub(crate) fn allocate(&mut self) -> u32 {
        let next_slot = self.next_slot;
        self.next_slot += 1;

        next_slot
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_allocate() {
        let mut allocator = SlotAllocator::default();

        assert_eq!(allocator.next_slot, 0);

        let next_slot = allocator.allocate();
        assert_eq!(next_slot, 0);
        let next_slot = allocator.allocate();
        assert_eq!(next_slot, 1);
    }
}
