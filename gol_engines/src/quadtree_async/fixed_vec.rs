use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    mem::MaybeUninit,
    ptr::NonNull,
};

/// A fixed-size vector that does not reallocate memory.
/// 
/// Most of its methods are unsafe and should be used with caution.
/// It is not thread-safe and should be synchronized externally.
pub struct FixedVec<T, const CHUNK_SIZE: usize> {
    ptr: NonNull<FixedVecBuffer<T, CHUNK_SIZE>>,
}

/// Same as `FixedVec`, but does not own the buffer.
/// 
/// IT SHOULD NEVER OUTLIVE THE `FixedVec` THAT CREATED IT.
pub struct FixedVecWeakRef<T, const CHUNK_SIZE: usize> {
    ptr: NonNull<FixedVecBuffer<T, CHUNK_SIZE>>,
}

struct FixedVecBuffer<T, const CHUNK_SIZE: usize> {
    size: usize,
    data: [MaybeUninit<T>; CHUNK_SIZE],
}

impl<T, const CHUNK_SIZE: usize> FixedVec<T, CHUNK_SIZE> {
    pub fn new() -> Self {
        let layout = Layout::new::<FixedVecBuffer<T, CHUNK_SIZE>>();
        let ptr = unsafe {
            let raw = alloc(layout) as *mut FixedVecBuffer<T, CHUNK_SIZE>;
            if raw.is_null() {
                handle_alloc_error(layout);
            }
            raw.write(FixedVecBuffer::new());
            NonNull::new_unchecked(raw)
        };
        FixedVec { ptr }
    }

    pub fn weak_ref(&self) -> FixedVecWeakRef<T, CHUNK_SIZE> {
        FixedVecWeakRef { ptr: self.ptr }
    }

    /// # Safety
    /// The caller must ensure that the length of the vector is less than the capacity.
    pub unsafe fn push(&mut self, value: T) {
        self.ptr.as_mut().push(value)
    }

    pub fn len(&self) -> usize {
        unsafe { self.ptr.as_ref().size }
    }

    /// # Safety
    /// The caller must ensure that the index is less than the length of the vector.
    pub unsafe fn get(&self, index: usize) -> &T {
        self.ptr.as_ref().get(index)
    }

    /// # Safety
    /// The caller must ensure that the index is less than the length of the vector.
    pub unsafe fn get_mut(&mut self, index: usize) -> &mut T {
        self.ptr.as_mut().get_mut(index)
    }
}

impl <T, const CHUNK_SIZE: usize> FixedVecWeakRef<T, CHUNK_SIZE> {
    /// # Safety
    /// The caller must ensure that the length of the vector is less than the capacity.
    pub unsafe fn push(&mut self, value: T) {
        self.ptr.as_mut().push(value)
    }

    pub fn len(&self) -> usize {
        unsafe { self.ptr.as_ref().size }
    }

    /// # Safety
    /// The caller must ensure that the index is less than the length of the vector.
    pub unsafe fn get(&self, index: usize) -> &T {
        self.ptr.as_ref().get(index)
    }

    /// # Safety
    /// The caller must ensure that the index is less than the length of the vector.
    pub unsafe fn get_mut(&mut self, index: usize) -> &mut T {
        self.ptr.as_mut().get_mut(index)
    }
}

impl<T, const CHUNK_SIZE: usize> Clone for FixedVecWeakRef<T, CHUNK_SIZE> {
    fn clone(&self) -> Self {
        FixedVecWeakRef { ptr: self.ptr }
    }
}

impl<T, const CHUNK_SIZE: usize> FixedVecBuffer<T, CHUNK_SIZE> {
    fn new() -> Self {
        Self {
            size: 0,
            data: std::array::from_fn(|_| MaybeUninit::uninit()),
        }
    }

    unsafe fn push(&mut self, value: T) {
        self.data.get_unchecked_mut(self.size).write(value);
        self.size += 1;
    }

    unsafe fn get(&self, index: usize) -> &T {
        &*self.data.get_unchecked(index).as_ptr()
    }

    unsafe fn get_mut(&mut self, index: usize) -> &mut T {
        &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
    }
}

impl<T, const CHUNK_SIZE: usize> Drop for FixedVec<T, CHUNK_SIZE> {
    fn drop(&mut self) {
        unsafe {
            let buffer = self.ptr.as_mut();
            for i in 0..buffer.size {
                buffer.data.get_unchecked_mut(i).assume_init_drop();
            }
            let layout = Layout::new::<FixedVecBuffer<T, CHUNK_SIZE>>();
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}
