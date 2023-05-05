use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

const VERTEX_SHADER: &str =
r#"
attribute vec3 position;
attribute vec3 color;
uniform vec2 scale;
uniform vec2 translation;
varying vec4 vertexColor;
void main() {
    gl_Position = vec4((position.x - translation.x) * scale.x - 1.0, (position.y - translation.y) * scale.y - 1.0, position.z, 1.0);
    vertexColor = vec4(color, 1.0);
}
"#;

const FRAGMENT_SHADER: &str =
r#"
varying lowp vec4 vertexColor;
void main() {
    gl_FragColor = vertexColor;
}
"#;


pub fn make_shader_program(
    context: &WebGlRenderingContext,
) -> Result<WebGlProgram, String> {
    let vert_shader = compile_shader(context, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER)?;
    let frag_shader = compile_shader(context, WebGlRenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
    link_program(context, &vert_shader, &frag_shader)
}

fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
