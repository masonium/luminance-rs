//! WebGL2 buffer implementation.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use std::slice;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::webgl2::state::{Bind, WebGL2State};
use crate::webgl2::WebGL2;
use luminance::backend::buffer::{
  Buffer as BufferBackend, BufferBase, BufferSlice as BufferSliceBackend,
};
use luminance::buffer::BufferError;

/// WebGL buffer.
#[derive(Clone)]
pub struct Buffer {
  pub(crate) buf: WebGlBuffer,
  pub(crate) bytes: usize,
  pub(crate) len: usize,
  state: Rc<RefCell<WebGL2State>>,
}

unsafe impl BufferBase for WebGL2 {
  type BufferRepr = Buffer;
}

unsafe impl<T> BufferBackend<T> for WebGL2 {
  unsafe fn new_buffer(&mut self, len: usize) -> Result<Self::BufferRepr, BufferError> {
    let bytes = mem::size_of::<T>() * len;
    let mut state = self.state.borrow_mut();

    // generate a buffer and force binding the handle; this prevent side-effects from previous bound
    // resources to prevent binding the buffer
    let buf = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;
    state.bind_array_buffer(Some(&buf), Bind::Forced);

    state.ctx.buffer_data_with_i32(
      WebGl2RenderingContext::ARRAY_BUFFER,
      bytes as i32,
      WebGl2RenderingContext::STREAM_DRAW,
    );

    Ok(Buffer {
      buf,
      bytes,
      len,
      state: self.state.clone(),
    })
  }

  unsafe fn destroy_buffer(buffer: &mut Self::BufferRepr) {
    let mut state = buffer.state.borrow_mut();

    state.unbind_buffer(&buffer.buf);
    state.ctx.delete_buffer(Some(&buffer.buf));
  }

  unsafe fn len(buffer: &Self::BufferRepr) -> usize {
    buffer.len
  }

  unsafe fn from_slice<S>(&mut self, slice: S) -> Result<Self::BufferRepr, BufferError>
  where
    S: AsRef<[T]>,
  {
    let mut state = self.state.borrow_mut();
    let slice = slice.as_ref();
    let len = slice.len();
    let bytes = mem::size_of::<T>() * len;

    let buf = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;
    state.bind_array_buffer(Some(&buf), Bind::Forced);

    let data = std::slice::from_raw_parts(slice.as_ptr() as *const _, bytes);
    state.ctx.buffer_data_with_u8_array(
      WebGl2RenderingContext::ARRAY_BUFFER,
      data,
      WebGl2RenderingContext::STREAM_DRAW,
    );

    Ok(Buffer {
      buf,
      bytes,
      len,
      state: self.state.clone(),
    })
  }

  //unsafe fn repeat(&mut self, len: usize, value: T) -> Result<Self::BufferRepr, BufferError>
  //where
  //  T: Copy,
  //{
  //  let mut buf = <Self as Buffer<T>>::new_buffer(self, len)?;
  //  Self::clear(&mut buf, value)?;
  //  Ok(buf)
  //}

  unsafe fn at(buffer: &Self::BufferRepr, i: usize) -> Option<T>
  where
    T: Copy,
  {
    if i >= buffer.len {
      None
    } else {
      buffer
        .state
        .borrow_mut()
        .bind_array_buffer(Some(&buffer.buf), Bind::Cached);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;
      let x = *ptr.add(i);
      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

      Some(x)
    }
  }

  //unsafe fn whole(buffer: &Self::BufferRepr) -> Vec<T>
  //where
  //  T: Copy,
  //{
  //  buffer
  //    .state
  //    .borrow_mut()
  //    .bind_array_buffer(buffer.handle, Bind::Cached);
  //  let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *mut T;
  //  let values = Vec::from_raw_parts(ptr, buffer.len, buffer.len);
  //  let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

  //  values
  //}

  //unsafe fn set(buffer: &mut Self::BufferRepr, i: usize, x: T) -> Result<(), BufferError>
  //where
  //  T: Copy,
  //{
  //  if i >= buffer.len {
  //    Err(BufferError::Overflow {
  //      index: i,
  //      buffer_len: buffer.len,
  //    })
  //  } else {
  //    buffer
  //      .state
  //      .borrow_mut()
  //      .bind_array_buffer(buffer.handle, Bind::Cached);
  //    let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T;
  //    *ptr.add(i) = x;
  //    let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

  //    Ok(())
  //  }
  //}

  //unsafe fn write_whole(buffer: &mut Self::BufferRepr, values: &[T]) -> Result<(), BufferError> {
  //  let len = values.len();
  //  let in_bytes = len * mem::size_of::<T>();

  //  // generate warning and recompute the proper number of bytes to copy
  //  let real_bytes = match in_bytes.cmp(&buffer.bytes) {
  //    Ordering::Less => {
  //      return Err(BufferError::TooFewValues {
  //        provided_len: len,
  //        buffer_len: buffer.len,
  //      })
  //    }

  //    Ordering::Greater => {
  //      return Err(BufferError::TooManyValues {
  //        provided_len: len,
  //        buffer_len: buffer.len,
  //      })
  //    }

  //    _ => in_bytes,
  //  };

  //  buffer
  //    .state
  //    .borrow_mut()
  //    .bind_array_buffer(buffer.handle, Bind::Cached);
  //  let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
  //  ptr::copy_nonoverlapping(values.as_ptr() as *const c_void, ptr, real_bytes);
  //  let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);

  //  Ok(())
  //}

  //unsafe fn clear(buffer: &mut Self::BufferRepr, x: T) -> Result<(), BufferError>
  //where
  //  T: Copy,
  //{
  //  Self::write_whole(buffer, &vec![x; buffer.len])
  //}
  //}

  //pub struct BufferSlice<T> {
  //buffer: RawBuffer,
  //ptr: *const T,
  //}

  //pub struct BufferSliceMut<T> {
  //buffer: RawBuffer,
  //ptr: *mut T,
  //}

  //unsafe impl<T> BufferSliceBackend<T> for WebGL2 {
  //type SliceRepr = BufferSlice<T>;

  //type SliceMutRepr = BufferSliceMut<T>;

  //unsafe fn slice_buffer(buffer: &Self::BufferRepr) -> Result<Self::SliceRepr, BufferError> {
  //  buffer
  //    .state
  //    .borrow_mut()
  //    .bind_array_buffer(buffer.handle, Bind::Cached);

  //  let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;
  //  let buffer = buffer.clone();

  //  if ptr.is_null() {
  //    Err(BufferError::MapFailed)
  //  } else {
  //    Ok(BufferSlice { buffer, ptr })
  //  }
  //}

  //unsafe fn slice_buffer_mut(
  //  buffer: &mut Self::BufferRepr,
  //) -> Result<Self::SliceMutRepr, BufferError> {
  //  buffer
  //    .state
  //    .borrow_mut()
  //    .bind_array_buffer(buffer.handle, Bind::Cached);

  //  let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE) as *mut T;
  //  let buffer = buffer.clone();

  //  if ptr.is_null() {
  //    Err(BufferError::MapFailed)
  //  } else {
  //    Ok(BufferSliceMut { buffer, ptr })
  //  }
  //}

  //unsafe fn destroy_buffer_slice(slice: &mut Self::SliceRepr) {
  //  slice
  //    .buffer
  //    .state
  //    .borrow_mut()
  //    .bind_array_buffer(slice.buffer.handle, Bind::Cached);
  //  gl::UnmapBuffer(gl::ARRAY_BUFFER);
  //}

  //unsafe fn destroy_buffer_slice_mut(slice: &mut Self::SliceMutRepr) {
  //  slice
  //    .buffer
  //    .state
  //    .borrow_mut()
  //    .bind_array_buffer(slice.buffer.handle, Bind::Cached);
  //  gl::UnmapBuffer(gl::ARRAY_BUFFER);
  //}

  //unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], BufferError> {
  //  Ok(slice::from_raw_parts(slice.ptr, slice.buffer.len))
  //}

  //unsafe fn obtain_slice_mut(slice: &mut Self::SliceMutRepr) -> Result<&mut [T], BufferError> {
  //  Ok(slice::from_raw_parts_mut(slice.ptr, slice.buffer.len))
  //}
}
