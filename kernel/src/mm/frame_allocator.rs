//! Minimal Physical Frame Allocator for Zeroth OS
//! Tracks 4KB memory pages with extreme efficiency (< 1MB overhead for 4GB RAM)

pub const PAGE_SIZE: usize = 4096;

pub struct FrameAllocator {
    next_free_frame: usize,
    memory_limit: usize,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        Self {
            next_free_frame: 0x1000000, // Start allocating above 16MB
            memory_limit: 0x40000000,   // Default 1GB ceiling
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.memory_limit = limit;
    }

    pub fn allocate_frame(&mut self) -> Option<usize> {
        if self.next_free_frame + PAGE_SIZE <= self.memory_limit {
            let frame = self.next_free_frame;
            self.next_free_frame += PAGE_SIZE;
            Some(frame)
        } else {
            None
        }
    }
}

pub static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();
