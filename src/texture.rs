//! This module provides texture features.

use core::marker::PhantomData;
use pixel::Pixel;

/// How to wrap texture coordinates while sampling textures?
#[derive(Clone, Copy, Debug)]
pub enum Wrap {
  /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
  /// every components.
  ClampToEdge,
  /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
  ///
  /// ```
  /// // given the frac function returning the fractional part of a floating number:
  /// coord_ith = frac(coord_ith); // always between [0;1]
  /// ```
  Repeat,
  /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
  MirroredRepeat
}

/// Minification and magnification filter.
#[derive(Clone, Copy, Debug)]
pub enum Filter {
  /// Clamp to nearest pixel.
  Nearest,
  /// Linear interpolation with surrounding pixels.
  Linear
}

/// Depth comparison to perform while depth test. `a` is the incoming fragment’s depth and b is the
/// fragment’s depth that is already stored.
#[derive(Clone, Copy, Debug)]
pub enum DepthComparison {
  /// Depth test never succeeds.
  Never,
  /// Depth test always succeeds.
  Always,
  /// Depth test succeeds if `a == b`.
  Equal,
  /// Depth test succeeds if `a != b`.
  NotEqual,
  /// Depth test succeeds if `a < b`.
  Less,
  /// Depth test succeeds if `a <= b`.
  LessOrEqual,
  /// Depth test succeeds if `a > b`.
  Greater,
  /// Depth test succeeds if `a >= b`.
  GreaterOrEqual
}

/// Reify a type into a `Dim`.
pub trait Dimensionable {
  type Size;

  /// Dimension.
  fn dim() -> Dim;
  /// Width of the associated `Size`.
  fn width(size: &Self::Size) -> u32;
  /// Height of the associated `Size`. If it doesn’t have one, set it to 1.
  fn height(_: &Self::Size) -> u32 { 1 }
  /// Depth of the associated `Size`. If it doesn’t have one, set it to 1.
  fn depth(_: &Self::Size) -> u32 { 1 }
}

pub fn dim_capacity<T>(size: &T::Size) -> u32 where T: Dimensionable {
  T::width(size) * T::height(size) * T::depth(size)
}

/// Dimension of a texture.
#[derive(Clone, Copy, Debug)]
pub enum Dim {
  DIM1,
  DIM2,
  DIM3,
  Cubemap
}

#[derive(Clone, Copy, Debug)]
pub struct DIM1;

impl Dimensionable for DIM1 {
  type Size = u32;

  fn dim() -> Dim { Dim::DIM1 }

  fn width(w: &u32) -> u32 { *w }
}

#[derive(Clone, Copy, Debug)]
pub struct DIM2;

impl Dimensionable for DIM2 {
  type Size = (u32, u32);

  fn dim() -> Dim { Dim::DIM2 }

  fn width(&(w, _): &(u32, u32)) -> u32 { w }

  fn height(&(_, h): &(u32, u32)) -> u32 { h }
}

#[derive(Clone, Copy, Debug)]
pub struct DIM3;

impl Dimensionable for DIM3 {
  type Size = (u32, u32, u32);

  fn dim() -> Dim { Dim::DIM3 }

  fn width(&(w, _, _): &(u32, u32, u32)) -> u32 { w }

  fn height(&(_, h, _): &(u32, u32, u32)) -> u32 { h }

  fn depth(&(_, _, d): &(u32, u32, u32)) -> u32 { d }
}

#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Size = u32;

  fn dim() -> Dim { Dim::Cubemap }

  fn width(s: &u32) -> u32 { *s }

  fn height(s: &u32) -> u32 { *s }

  fn depth(s: &u32) -> u32 { *s }
}

/// Reify a type into a `Layering`.
pub trait Layerable {
  fn layering() -> Layering;
}

/// Texture layering. If a texture is layered, it has an extra coordinates to access the layer.
#[derive(Clone, Copy, Debug)]
pub enum Layering {
  /// Non-layered.
  Flat,
  /// Layered.
  Layered
}

#[derive(Clone, Copy, Debug)]
pub struct Flat;

impl Layerable for Flat { fn layering() -> Layering { Layering::Flat } }

#[derive(Clone, Copy, Debug)]
pub struct Layered;

impl Layerable for Layered { fn layering() -> Layering { Layering::Layered } }

/// Trait to implement to provide texture features.
pub trait HasTexture {
  type ATex;

  /// Create a new texture.
  fn new<L, D, P>(size: D::Size, mipmaps: u32, sampler: &Sampler) -> Self::ATex
    where L: Layerable,
          D: Dimensionable,
          P: Pixel;
  /// Destroy a texture.
  fn free(tex: &mut Self::ATex);
  /// Clear the texture’s texels by setting them all to the same value.
  fn clear<P>(tex: &Self::ATex, pixel: &P::Encoding) where P: Pixel;
  /// Upload texels to the texture’s memory.
  fn upload<P>(tex: &Self::ATex, texels: &Vec<P::Encoding>) where P: Pixel;
}

/// Texture.
///
/// `L` refers to the layering type; `D` refers to the dimension; `P` is the pixel format for the
/// texels.
#[derive(Debug)]
pub struct Tex<C, L, D, P> where C: HasTexture, L: Layerable, D: Dimensionable, P: Pixel {
  pub repr: C::ATex,
  pub size: D::Size,
  pub mipmaps: u32,
  pub texels: Vec<P::Encoding>,
  _l: PhantomData<L>,
  _c: PhantomData<C>,
}

impl<C, L, D, P> Drop for Tex<C, L, D, P> where C: HasTexture, L: Layerable, D: Dimensionable, P: Pixel {
  fn drop(&mut self) {
    C::free(&mut self.repr)
  }
}

impl<C, L, D, P> Tex<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel {
  pub fn new(size: D::Size, mipmaps: u32, sampler: &Sampler) -> Self {
    let tex = C::new::<L, D, P>(size, mipmaps, sampler);

    Tex {
      repr: tex,
      size: size,
      mipmaps: mipmaps,
      texels: Vec::with_capacity(dim_capacity::<D>(&size) as usize),
      _c: PhantomData,
      _l: PhantomData,
    }
  }

  pub fn clear(&self, pixel: &P::Encoding) {
    C::clear::<P>(&self.repr, pixel)
  }

  pub fn upload(&self, texels: &Vec<P::Encoding>) {
    C::upload::<P>(&self.repr, texels)
  }
}

/// A `Sampler` object gives hint on how a `Tex` should be sampled.
#[derive(Clone, Copy, Debug)]
pub struct Sampler {
  /// How should we wrap around the *r* sampling coordinate?
  pub wrap_r: Wrap,
  /// How should we wrap around the *s* sampling coordinate?
  pub wrap_s: Wrap,
  /// How should we wrap around the *t* sampling coordinate?
  pub wrap_t: Wrap,
  /// Minification filter.
  pub minification: Filter,
  /// Magnification filter.
  pub magnification: Filter,
  /// For depth textures, should we perform depth comparison and if so, how?
  pub depth_comparison: Option<DepthComparison>
}

/// Default value is as following:
///
/// ```
/// Sampler {
///   wrap_r: Wrap::ClampToEdge,
///   wrap_s: Wrap::ClampToEdge,
///   wrap_t: Wrap::ClampToEdge,
///   minification: Filter::Linear,
///   magnification: Filter::Linear,
///   depth_comparison: None
/// }
/// ```
impl Default for Sampler {
  fn default() -> Self {
    Sampler {
      wrap_r: Wrap::ClampToEdge,
      wrap_s: Wrap::ClampToEdge,
      wrap_t: Wrap::ClampToEdge,
      minification: Filter::Linear,
      magnification: Filter::Linear,
      depth_comparison: None
    }
  }
}
