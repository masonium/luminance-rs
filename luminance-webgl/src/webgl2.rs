//! WebGL 2.0 backend support.

mod buffer;
pub(crate) mod state;

use crate::webgl2::state::{StateQueryError, WebGL2State};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::WebGl2RenderingContext;

/// The WebGL2 backend.
pub struct WebGL2 {
  pub(crate) state: Rc<RefCell<WebGL2State>>,
}

impl WebGL2 {
  pub fn new(ctx: WebGl2RenderingContext) -> Result<Self, StateQueryError> {
    WebGL2State::new(ctx).map(|state| WebGL2 {
      state: Rc::new(RefCell::new(state)),
    })
  }
}
