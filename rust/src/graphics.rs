use image::{RgbImage, load_from_memory};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

use crate::error::{Result, WebError};

const IMAGE_VERT: &str = include_str!("shaders/vert.glsl");
const IMAGE_FRAG: &str = include_str!("shaders/frag.glsl");

// Enum para definir modos de renderizado, con valor por defecto None.
// Aca podes agregar mas modos y luego agregarlos en el fragment shader (frag.glsl).
#[derive(Clone, Copy, Default)]
pub enum RenderMode {
    #[default]
    None = 0,
    Crt = 1,
}

impl RenderMode {
    pub fn next(self) -> Self {
        match self {
            RenderMode::None => RenderMode::Crt,
            RenderMode::Crt => RenderMode::None,
        }
    }
}

// Imagen cargada una sola vez al inicio.
static IMAGE: Lazy<RgbImage> = Lazy::new(|| {
    load_from_memory(include_bytes!("images/ferris.png"))
        .expect("failed to decode image")
        .to_rgb8()
});

// Posiciones de vértices para un rectángulo que cubre la pantalla (dos triángulos).
static BOX_POSITIONS: &[f32; 12] = &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0];

fn compile_webgl_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or(WebError::UnableCreateShader)?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(WebError::CreateShader(
            gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")),
        ))
    }
}

fn link_webgl_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram> {
    let program = gl.create_program().ok_or(WebError::UnableCreateProgram)?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(WebError::CreateProgram(
            gl.get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program")),
        ))
    }
}

// Configura el buffer de posición de vértices para el rectángulo.
fn box_vertex_position(gl: &WebGlRenderingContext, program: &WebGlProgram) -> Result<u32> {
    let position_buffer = gl.create_buffer().ok_or(WebError::CreateBuffer)?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(BOX_POSITIONS);

        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    let a_position = gl.get_attrib_location(program, "a_position") as u32;
    gl.enable_vertex_attrib_array(a_position);

    gl.vertex_attrib_pointer_with_i32(a_position, 2, WebGlRenderingContext::FLOAT, false, 0, 0);

    Ok(a_position)
}

// Obtiene el elemento canvas HTML por su ID.
fn get_canvas(element_id: &str) -> Result<HtmlCanvasElement> {
    let document = web_sys::window()
        .expect("Could not get Window object.")
        .document()
        .unwrap();

    let canvas = document
        .get_element_by_id(element_id)
        .ok_or_else(|| WebError::MissingCanvasElement(element_id.into()))?;

    let canvas = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(WebError::GetCanvasHandle)?;

    Ok(canvas)
}

#[wasm_bindgen]
pub struct Graphics {
    gl: WebGlRenderingContext,
    canvas: HtmlCanvasElement,
    program: WebGlProgram,
}

impl Graphics {
    // Crea una instancia de Graphics obteniendo contexto WebGL, compilando shaders y creando programa.
    pub fn new(element_id: &str) -> Result<Self> {
        let canvas = get_canvas(element_id)?;

        let gl_object = match canvas.get_context("webgl") {
            Ok(Some(t)) => t,
            Ok(None) | Err(_) => return Err(WebError::GetCanvasWebglContext),
        };

        let gl = gl_object
            .dyn_into::<WebGlRenderingContext>()
            .map_err(WebError::GetCanvasWebglHandle)?;

        // El vertex shader es un programa que se ejecuta para cada vértice del modelo 3D o geometría.
        // Su función principal es calcular la posición final de cada vértice en coordenadas de pantalla (clip space),
        // y puede pasar datos a los fragment shaders, como colores o coordenadas de textura.
        let vert_shader =
            compile_webgl_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, IMAGE_VERT)?;

        // El fragment shader es un programa que se ejecuta para cada fragmento (potencial píxel) generado por la rasterización de los triángulos.
        // Su función es calcular el color final de cada píxel, incluyendo efectos de luz, textura, transparencia, etc.
        let frag_shader =
            compile_webgl_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, IMAGE_FRAG)?;

        let program = link_webgl_program(&gl, &vert_shader, &frag_shader)?;

        Ok(Self {
            gl,
            canvas,
            program,
        })
    }

    // Método para dibujar a Ferris usando WebGL.
    pub fn draw(&self, mode: RenderMode) -> Result<()> {
        self.gl.use_program(Some(&self.program));

        // Define el viewport para cubrir todo el canvas.
        self.gl.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );

        // Un *uniform* es una variable global definida en el shader GLSL que el programa WebGL puede asignar desde la CPU.
        // A diferencia de los atributos que cambian por cada vértice, los uniforms permanecen constantes para todos los vértices o fragmentos
        // de una misma llamada de dibujo ("draw call"). Esto permite enviar datos que no varían dentro de un frame,
        // como matrices de transformación, tiempos, resoluciones, o parámetros de efectos.

        // En este código, los uniforms se usan para:
        // - u_matrix: para transformar las posiciones de los vértices al espacio de recorte (clip space).
        // - u_now: para pasar el tiempo actual, usado para animaciones en el shader.
        // - u_resolution: para informar al shader sobre el tamaño del área de dibujo, útil para efectos dependientes de resolución.
        // - u_mode: para controlar modos de renderizado (por ejemplo, efecto CRT).

        // Configura los vértices para dibujar el rectángulo.
        let a_position = box_vertex_position(&self.gl, &self.program)?;

        let clip_width = IMAGE.width() as f32 / self.canvas.width() as f32 * 2.0;
        let clip_height = IMAGE.height() as f32 / self.canvas.height() as f32 * -2.0;

        let clip: [f32; 9] = [clip_width, 0.0, 0.0, 0.0, clip_height, 0.0, -1.0, 1.0, 1.0];

        let u_matrix_loc = self.gl.get_uniform_location(&self.program, "u_matrix");

        self.gl
            .uniform_matrix3fv_with_f32_array(Some(&u_matrix_loc.unwrap()), false, &clip);

        // Envía el tiempo actual fraccional para animaciones en el shader.
        let u_now_loc = self
            .gl
            .get_uniform_location(&self.program, "u_now")
            .unwrap();

        let now = js_sys::Date::now() / 1000.0;
        let fractional = (now % 1.0) as f32;

        self.gl.uniform1f(Some(&u_now_loc), fractional);

        // Envía la resolución del buffer de dibujo para cálculos en el shader.
        let u_resolution_loc = self
            .gl
            .get_uniform_location(&self.program, "u_resolution")
            .unwrap();

        let width = self.gl.drawing_buffer_width() as f32;
        let height = self.gl.drawing_buffer_height() as f32;

        self.gl.uniform2f(Some(&u_resolution_loc), width, height);

        // Envía el modo de renderizado.
        let u_mode_loc = self
            .gl
            .get_uniform_location(&self.program, "u_mode")
            .unwrap();

        self.gl.uniform1i(Some(&u_mode_loc), mode as i32);

        // Configuración de textura

        self.gl
            .pixel_storei(WebGlRenderingContext::UNPACK_ALIGNMENT, 1);

        let texture = self.gl.create_texture().ok_or(WebError::CreateTexture)?;

        self.gl.active_texture(WebGlRenderingContext::TEXTURE0);
        self.gl
            .bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

        // Carga la imagen cargada en memoria como textura WebGL
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGlRenderingContext::TEXTURE_2D,
                0,
                WebGlRenderingContext::RGB as i32,
                IMAGE.width() as i32,
                IMAGE.height() as i32,
                0,
                WebGlRenderingContext::RGB,
                WebGlRenderingContext::UNSIGNED_BYTE,
                Some(IMAGE.as_raw()),
            )
            .map_err(WebError::LoadTexture)?;

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::LINEAR as i32,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::LINEAR as i32,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_S,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_T,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32,
        );

        let u_image = self.gl.get_uniform_location(&self.program, "u_image");

        self.gl.uniform1i(Some(&u_image.unwrap()), 0);

        // Dibuja el rectángulo (dos triángulos) usando los vértices y textura.
        self.gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);

        self.gl.disable_vertex_attrib_array(a_position);

        Ok(())
    }
}
