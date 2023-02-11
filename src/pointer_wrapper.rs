use core::ops::Deref;

/// Raw pointer that implements the `Send` trait since it's only acting on static memory
///
/// Should always point at the beginning of your audio buffer in use
#[derive(Debug)]
pub struct BufferPointer(pub *const f32);
unsafe impl Send for BufferPointer {}

impl BufferPointer {
    pub fn add(&self, offset: usize) -> BufferPointer {
        unsafe { BufferPointer(self.0.add(offset)) }
    }
}

/// Since we know that our pointer is always pointing at some buffer in memory, it can
/// never be dangling. Thats's why it is safe to dereference it with `unsafe`.
impl Deref for BufferPointer {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

#[derive(Debug)]
pub struct BufferSlice {
    pub ptr: BufferPointer,
    pub length: f32,
}

impl BufferSlice {
    pub fn from_slice(slice: &[f32]) -> BufferSlice {
        BufferSlice {
            ptr: BufferPointer(slice.as_ptr()),
            length: slice.len() as f32,
        }
    }

    pub fn get_sub_slice(&self, mut offset: usize, mut length: f32) -> BufferSlice {
        // truncate offset if too far
        if offset >= self.length as usize {
            offset = (self.length - 1.0) as usize;
        }

        // afterwards truncate length if too long
        if offset as f32 + length >= self.length {
            length = self.length - offset as f32;
        }

        BufferSlice {
            ptr: self.ptr.add(offset),
            length: length,
        }
    }

    pub fn get_value_at(&self, position: usize) -> f32 {
        *self.get_sub_slice(position, 1.0).ptr
    }
}
