pub struct ChunkVec<const CHUNK_SIZE: usize, T> {
    chunks: Vec<*mut T>,
    last_idx: usize,
    len: usize,
}

impl<const CHUNK_SIZE: usize, T> ChunkVec<CHUNK_SIZE, T> {
    pub fn new() -> Self {
        Self {
            chunks: vec![Self::new_chunk()],
            last_idx: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, val: T) {
        if self.last_idx == CHUNK_SIZE {
            self.chunks.push(Self::new_chunk());
            self.last_idx = 0;
        }
        let i = self.chunks.len() - 1;
        unsafe {
            let last = self.chunks.get_unchecked_mut(i);
            *last.add(self.last_idx) = val;
        }
        self.last_idx += 1;
        self.len += 1;
    }

    #[inline]
    fn new_chunk() -> *mut T {
        let layout = std::alloc::Layout::array::<T>(CHUNK_SIZE).unwrap();
        unsafe { std::alloc::alloc(layout) as *mut T }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn bytes_total(&self) -> usize {
        self.chunks.len() * (std::mem::size_of::<usize>() + CHUNK_SIZE * std::mem::size_of::<T>())
    }
}

impl<const CHUNK_SIZE: usize, T> std::ops::Index<usize> for ChunkVec<CHUNK_SIZE, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &*self
                .chunks
                .get_unchecked(index / CHUNK_SIZE)
                .add(index % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize, T> std::ops::IndexMut<usize> for ChunkVec<CHUNK_SIZE, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut *self
                .chunks
                .get_unchecked_mut(index / CHUNK_SIZE)
                .add(index % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize, T> Drop for ChunkVec<CHUNK_SIZE, T> {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::array::<T>(CHUNK_SIZE).unwrap();
        for ptr in self.chunks.iter().copied() {
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    }
}
