use core::ops::Deref;
use num_traits::AsPrimitive;

/// Raw pointer that implements the `Send` trait since it's only acting on static memory
///
/// Should always point at the beginning of your audio buffer in use
#[derive(Debug)]
pub struct BufferPointer<T: AsPrimitive<f32>>(pub *const T);
unsafe impl<T: AsPrimitive<f32>> Send for BufferPointer<T> {}

impl<T: AsPrimitive<f32>> BufferPointer<T> {
    pub fn add(&self, offset: usize) -> BufferPointer<T> {
        unsafe { BufferPointer(self.0.add(offset)) }
    }
}

/// Since we know that our pointer is always pointing at some buffer in memory, it can
/// never be dangling. Thats's why it is safe to dereference it with `unsafe`.
impl<T: AsPrimitive<f32>> Deref for BufferPointer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

#[derive(Debug)]
pub struct BufferSlice<T: AsPrimitive<f32>> {
    pub ptr: BufferPointer<T>,
    pub length: usize,
}

impl<T: AsPrimitive<f32>> BufferSlice<T> {
    pub fn from_slice(slice: &[T]) -> BufferSlice<T> {
        BufferSlice {
            ptr: BufferPointer(slice.as_ptr()),
            length: slice.len(),
        }
    }

    pub fn get_sub_slice(&self, offset: &mut usize, length: &mut usize) -> BufferSlice<T> {
        // truncate offset if too far
        if *offset >= self.length {
            *offset = self.length - 1;
        }

        // afterwards truncate length if too long
        if *offset + *length >= self.length {
            *length = self.length - *offset;
        }

        BufferSlice {
            ptr: self.ptr.add(*offset),
            length: *length,
        }
    }

    pub fn get_f32_value_at(&self, position: &mut usize) -> f32 {
        self.get_sub_slice(position, &mut 1).ptr.as_()
    }
}
