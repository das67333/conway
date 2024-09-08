pub struct Deque<const BLOCK_SIZE: usize, T> {
    blocks: Vec<*mut T>,
    last_idx: usize,
    len: usize,
}

impl<const BLOCK_SIZE: usize, T> Deque<BLOCK_SIZE, T> {
    pub fn new() -> Self {
        Self {
            blocks: vec![Self::new_block()],
            last_idx: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, val: T) {
        if self.last_idx == BLOCK_SIZE {
            self.blocks.push(Self::new_block());
            self.last_idx = 0;
        }
        let i = self.blocks.len() - 1;
        unsafe {
            let last = self.blocks.get_unchecked_mut(i);
            *last.add(self.last_idx) = val;
        }
        self.last_idx += 1;
        self.len += 1;
    }

    #[inline]
    fn new_block() -> *mut T {
        let layout = std::alloc::Layout::array::<T>(BLOCK_SIZE).unwrap();
        unsafe { std::alloc::alloc(layout) as *mut T }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<const BLOCK_SIZE: usize, T> std::ops::Index<usize> for Deque<BLOCK_SIZE, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &*self
                .blocks
                .get_unchecked(index / BLOCK_SIZE)
                .add(index % BLOCK_SIZE)
        }
    }
}

impl<const BLOCK_SIZE: usize, T> std::ops::IndexMut<usize> for Deque<BLOCK_SIZE, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut *self
                .blocks
                .get_unchecked_mut(index / BLOCK_SIZE)
                .add(index % BLOCK_SIZE)
        }
    }
}

impl<const BLOCK_SIZE: usize, T> Drop for Deque<BLOCK_SIZE, T> {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::array::<T>(BLOCK_SIZE).unwrap();
        for ptr in self.blocks.iter().copied() {
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    }
}
