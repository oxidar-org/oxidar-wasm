use thiserror::Error;
use wasm_bindgen::JsValue;

pub type Result<T> = std::result::Result<T, WebError>;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("Missing canvas element id {0}.")]
    MissingCanvasElement(String),

    #[error("Can't get canvas element handle. {0:?}")]
    GetCanvasHandle(web_sys::Element),

    #[error("Can't get canvas webgl context handle. {0:?}")]
    GetCanvasWebglHandle(js_sys::Object),

    #[error("Error creating shader: {0}")]
    CreateShader(String),

    #[error("Can't get canvas WebGL context. You browser don't supports WebGL.")]
    GetCanvasWebglContext,

    #[error("Unable to create shader object.")]
    UnableCreateShader,

    #[error("Error creating program: {0}")]
    CreateProgram(String),

    #[error("Unable to create program object.")]
    UnableCreateProgram,

    #[error("Graphics not initialized.")]
    GraphicsNotInitialized,

    #[error("Failed to create buffer.")]
    CreateBuffer,

    #[error("Failed to create texture.")]
    CreateTexture,

    #[error("Failed to load texture. {0:?}")]
    LoadTexture(JsValue),
}

// Implementa la conversión automática del tipo `WebError` a `JsValue`.
// Esto permite retornar errores Rust a JavaScript.
impl From<WebError> for JsValue {
    fn from(err: WebError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}
