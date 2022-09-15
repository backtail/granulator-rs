/// Raw pointer that implements the `Send` trait since it's only acting on static memory
///
/// Should always point at the beginning of your audio buffer in use
pub struct BufferPointer(pub *const f32);
unsafe impl Send for BufferPointer {}

impl BufferPointer {
    pub fn add(&self, offset: usize) -> BufferPointer {
        unsafe { BufferPointer(self.0.add(offset)) }
    }
}

/// Raw slice pointer that implements the `Send` trait since it's only acting on static memory
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

    pub fn as_slice(&self) -> *const [f32] {
        core::ptr::slice_from_raw_parts(self.ptr.0, self.length as usize)
    }
}
