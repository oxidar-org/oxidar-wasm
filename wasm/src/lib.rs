use log::{Level, info};
use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;

mod error;
mod graphics;

pub use error::{Result, WebError};
use graphics::{Graphics, RenderMode};

#[wasm_bindgen]
pub fn setup_logs() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).expect("Could not setup log level.");

    info!("WASM initialized!");
}

#[wasm_bindgen]
#[derive(Default)]
pub struct RustCanvas {
    graphics: Rc<RefCell<Option<Graphics>>>,
    mode: RenderMode,
}

#[wasm_bindgen]
impl RustCanvas {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn init(&mut self, element_id: &str) -> Result<()> {
        let mut graphics = self.graphics.borrow_mut();

        if graphics.as_ref().is_none() {
            *graphics = Some(Graphics::new(element_id)?)
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "toggleMode")]
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    pub fn draw(&mut self) -> Result<()> {
        let graphics = self.graphics.borrow();

        match graphics.as_ref() {
            Some(t) => t.draw(self.mode),
            None => Err(WebError::GraphicsNotInitialized),
        }
    }
}
