#![allow(unused)]
//! Handles allocation of VAT buffer slots

#[derive(Debug, Default)]
pub(crate) struct SlotAllocator {
    next_slot: u32,
    free_list: Vec<u32>,
}

impl SlotAllocator {
    pub(crate) fn allocate(&mut self) -> u32 {
        if let Some(slot) = self.free_list.pop() {
            return slot;
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        slot
    }

    pub(crate) fn free(&mut self, slot: u32) {
        self.free_list.push(slot);
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

    #[test]
    fn test_free_reclaims_slot() {
        let mut allocator = SlotAllocator::default();

        let a = allocator.allocate();
        let b = allocator.allocate();
        assert_eq!(a, 0);
        assert_eq!(b, 1);

        allocator.free(a);
        let c = allocator.allocate();
        assert_eq!(c, 0);

        let d = allocator.allocate();
        assert_eq!(d, 2);
    }
}
