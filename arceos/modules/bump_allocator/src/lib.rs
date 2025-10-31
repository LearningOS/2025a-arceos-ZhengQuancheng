#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,      // 内存区域起始位置
    end: usize,        // 内存区域结束位置
    b_pos: usize,      // 字节分配的位置指针
    p_pos: usize,      // 页分配的位置指针
    count: usize,      // 记录字节分配的次数
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        todo!()
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        // 计算对齐后的分配位置
        let align = layout.align();
        let size = layout.size();
        let aligned_pos = (self.b_pos + align - 1) & !(align - 1);
        let new_b_pos = aligned_pos + size;
        // 检查是否有足够的空间
        if new_b_pos > self.p_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        // 更新分配位置和计数器
        self.b_pos = new_b_pos;
        self.count += 1;
        // 返回分配的内存地址
        core::ptr::NonNull::new(aligned_pos as *mut u8).ok_or(allocator::AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if self.count > 0 {
            // 更新计数器
            self.count -= 1;
            // 所有分配的字节都被释放
            if self.count == 0 {
                // 重置分配位置
                self.b_pos = self.start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        (self.b_pos - self.start) + (self.end - self.p_pos)
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        // 计算对齐后的分配位置
        let align = 1 << align_pow2;
        let size = num_pages * Self::PAGE_SIZE;
        let aligned_pos = (self.p_pos - size) & !(align - 1);
        // 检查是否有足够的空间
        if aligned_pos < self.b_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        // 更新分配位置
        self.p_pos = aligned_pos;
        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        todo!()
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / Self::PAGE_SIZE
    }
}