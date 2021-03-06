//! Tessellation gates.
//!
//! A tessellation gate is a _pipeline node_ that allows to share [`Tess`] for deeper nodes.
//!
//! [`Tess`]: crate::tess::Tess

use crate::backend::tess_gate::TessGate as TessGateBackend;
use crate::context::GraphicsContext;
use crate::tess::TessView;

/// Tessellation gates.
pub struct TessGate<'a, C>
where
  C: ?Sized,
{
  pub(crate) ctx: &'a mut C,
}

impl<'a, C> TessGate<'a, C>
where
  C: ?Sized + GraphicsContext,
  C::Backend: TessGateBackend,
{
  /// Enter the [`TessGate`] by sharing a [`TessView`].
  pub fn render<'b, T>(&'b mut self, tess_view: T)
  where
    T: Into<TessView<'b, C::Backend>>,
  {
    let tess_view = tess_view.into();

    unsafe {
      self.ctx.backend().render(
        &tess_view.tess.repr,
        tess_view.start_index,
        tess_view.vert_nb,
        tess_view.inst_nb,
      )
    }
  }
}
