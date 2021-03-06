//! Tessellation backend interface.
//!
//! This interface defines the low-level API tessellations must implement to be usable.

use crate::backend::buffer::Buffer;
use crate::tess::{Mode, TessError, TessIndex, TessMapError};
use crate::vertex::Vertex;

pub unsafe trait TessBuilder {
  type TessBuilderRepr;

  unsafe fn new_tess_builder(&mut self) -> Result<Self::TessBuilderRepr, TessError>;

  unsafe fn add_vertices<V, W>(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    vertices: W,
  ) -> Result<(), TessError>
  where
    W: AsRef<[V]>,
    V: Copy + Vertex;

  unsafe fn add_instances<V, W>(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    instances: W,
  ) -> Result<(), TessError>
  where
    W: AsRef<[V]>,
    V: Copy + Vertex;

  unsafe fn set_indices<T, I>(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    indices: T,
  ) -> Result<(), TessError>
  where
    T: AsRef<[I]>,
    I: Copy + TessIndex;

  unsafe fn set_mode(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    mode: Mode,
  ) -> Result<(), TessError>;

  unsafe fn set_vertex_nb(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    nb: usize,
  ) -> Result<(), TessError>;

  unsafe fn set_instance_nb(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    nb: usize,
  ) -> Result<(), TessError>;

  unsafe fn set_primitive_restart_index(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    index: Option<u32>,
  ) -> Result<(), TessError>;
}

pub unsafe trait TessBuilderBuffer<T>: TessBuilder + Buffer<T>
where
  T: Copy,
{
  unsafe fn add_vertex_buffer(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    buf: Self::BufferRepr,
  ) -> Result<(), TessError>
  where
    T: Vertex;

  unsafe fn add_instance_buffer(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    buf: Self::BufferRepr,
  ) -> Result<(), TessError>
  where
    T: Vertex;

  unsafe fn set_index_buffer(
    &mut self,
    tess_builder: &mut Self::TessBuilderRepr,
    buf: Self::BufferRepr,
  ) -> Result<(), TessError>
  where
    T: TessIndex;
}

pub unsafe trait Tess: TessBuilder {
  type TessRepr;

  unsafe fn build(
    &mut self,
    tess_builder: Self::TessBuilderRepr,
  ) -> Result<Self::TessRepr, TessError>;

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError>;
}

pub unsafe trait TessSlice<T>: Tess {
  type SliceRepr;

  type SliceMutRepr;

  unsafe fn slice_vertices(tess: &Self::TessRepr) -> Result<Self::SliceRepr, TessMapError>
  where
    T: Vertex;

  unsafe fn slice_vertices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::SliceMutRepr, TessMapError>
  where
    T: Vertex;

  unsafe fn slice_indices(tess: &Self::TessRepr) -> Result<Self::SliceRepr, TessMapError>
  where
    T: TessIndex;

  unsafe fn slice_indices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::SliceMutRepr, TessMapError>
  where
    T: TessIndex;

  unsafe fn slice_instances(tess: &Self::TessRepr) -> Result<Self::SliceRepr, TessMapError>
  where
    T: Vertex;

  unsafe fn slice_instances_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::SliceMutRepr, TessMapError>
  where
    T: Vertex;

  unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], TessMapError>;

  unsafe fn obtain_slice_mut(slice: &mut Self::SliceMutRepr) -> Result<&mut [T], TessMapError>;
}
