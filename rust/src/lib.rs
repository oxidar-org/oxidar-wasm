use log::{Level, info};
use std::{cell::RefCell, rc::Rc};

// Macros y atributos necesarios para interoperabilidad con JavaScript.
use wasm_bindgen::prelude::*;

mod error;
mod graphics;

pub use error::{Result, WebError};
use graphics::{Graphics, RenderMode};

/// Función pública expuesta a JavaScript que configura los logs de consola para debugging.
#[wasm_bindgen]
pub fn setup_logs() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).expect("Could not setup log level.");

    info!("WASM initialized!");
}

/// Objeto principal que maneja la lógica de renderizado WebGL.
#[wasm_bindgen]
#[derive(Default)]
pub struct RustCanvas {
    graphics: Rc<RefCell<Option<Graphics>>>,
    mode: RenderMode,
}

#[wasm_bindgen]
impl RustCanvas {
    /// Crea una nueva instancia por defecto. Esta metodo se invoca desde JS.
    pub fn create() -> Self {
        Self::default()
    }

    /// Inicializa el sistema gráfico con un elemento HTML dado (por ID).
    ///
    /// # Argumentos
    /// * `element_id` - ID del canvas HTML donde se hará el renderizado.
    pub fn init(&mut self, element_id: &str) -> Result<()> {
        let mut graphics = self.graphics.borrow_mut();

        if graphics.as_ref().is_none() {
            *graphics = Some(Graphics::new(element_id)?)
        }

        Ok(())
    }

    /// Cambia el modo de renderizado al siguiente modo disponible.
    /// Expone esta función a JS bajo el nombre `toggleMode`.
    #[wasm_bindgen(js_name = "toggleMode")]
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    /// Dibuja la escena.
    pub fn draw(&mut self) -> Result<()> {
        let graphics = self.graphics.borrow();

        match graphics.as_ref() {
            Some(t) => t.draw(self.mode),
            None => Err(WebError::GraphicsNotInitialized),
        }
    }
}
