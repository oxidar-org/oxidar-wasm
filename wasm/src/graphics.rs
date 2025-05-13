use image::{RgbImage, load_from_memory};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

use crate::error::{Result, WebError};

const IMAGE_VERT: &str = include_str!("shaders/vert.glsl");
const IMAGE_FRAG: &str = include_str!("shaders/frag.glsl");

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

static IMAGE: Lazy<RgbImage> = Lazy::new(|| {
    load_from_memory(include_bytes!("images/ferris.png"))
        .expect("failed to decode image")
        .to_rgb8()
});

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
    pub fn new(element_id: &str) -> Result<Self> {
        let canvas = get_canvas(element_id)?;

        let gl_object = match canvas.get_context("webgl") {
            Ok(Some(t)) => t,
            Ok(None) | Err(_) => return Err(WebError::GetCanvasWebglContext),
        };

        let gl = gl_object
            .dyn_into::<WebGlRenderingContext>()
            .map_err(WebError::GetCanvasWebglHandle)?;

        let vert_shader =
            compile_webgl_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, IMAGE_VERT)?;

        let frag_shader =
            compile_webgl_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, IMAGE_FRAG)?;

        let program = link_webgl_program(&gl, &vert_shader, &frag_shader)?;

        Ok(Self {
            gl,
            canvas,
            program,
        })
    }

    pub fn draw(&self, mode: RenderMode) -> Result<()> {
        self.gl.use_program(Some(&self.program));

        self.gl.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );

        let a_position = box_vertex_position(&self.gl, &self.program)?;

        let clip_width = IMAGE.width() as f32 / self.canvas.width() as f32 * 2.0;
        let clip_height = IMAGE.height() as f32 / self.canvas.height() as f32 * -2.0;

        let clip: [f32; 9] = [clip_width, 0.0, 0.0, 0.0, clip_height, 0.0, -1.0, 1.0, 1.0];

        let u_matrix_loc = self.gl.get_uniform_location(&self.program, "u_matrix");

        self.gl
            .uniform_matrix3fv_with_f32_array(Some(&u_matrix_loc.unwrap()), false, &clip);

        let u_now_loc = self
            .gl
            .get_uniform_location(&self.program, "u_now")
            .unwrap();

        let now = js_sys::Date::now() / 1000.0;
        let fractional = (now % 1.0) as f32;

        self.gl.uniform1f(Some(&u_now_loc), fractional);

        let u_resolution_loc = self
            .gl
            .get_uniform_location(&self.program, "u_resolution")
            .unwrap();

        let width = self.gl.drawing_buffer_width() as f32;
        let height = self.gl.drawing_buffer_height() as f32;

        self.gl.uniform2f(Some(&u_resolution_loc), width, height);

        let u_mode_loc = self
            .gl
            .get_uniform_location(&self.program, "u_mode")
            .unwrap();

        self.gl.uniform1i(Some(&u_mode_loc), mode as i32);

        // texture

        self.gl
            .pixel_storei(WebGlRenderingContext::UNPACK_ALIGNMENT, 1);

        let texture = self.gl.create_texture().ok_or(WebError::CreateTexture)?;

        self.gl.active_texture(WebGlRenderingContext::TEXTURE0);
        self.gl
            .bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

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

        // render
        self.gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);

        self.gl.disable_vertex_attrib_array(a_position);

        Ok(())
    }
}
