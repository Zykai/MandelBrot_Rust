use gl;
use gl::types::*;
use glutin::{Api, GlProfile, GlRequest};
use std::ffi::{CStr, CString};
use std::fs;
use std::time::Instant;
use winit::{
    DeviceEvent, ElementState, Event, EventsLoop, MouseScrollDelta, VirtualKeyCode, WindowBuilder,
    WindowEvent,
};

fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let mut matrix = [[0.0; 4]; 4];
    matrix[0][0] = 2.0 / (right - left);
    matrix[3][0] = -((right + left) / (right - left));
    matrix[1][1] = 2.0 / (top - bottom);
    matrix[3][1] = -((top + bottom) / (top - bottom));
    matrix[2][2] = -2.0 / (far - near);
    matrix[3][2] = -((far + near) / (far - near));
    matrix[3][3] = 1.0;
    matrix
}

fn scale(factor: f32) -> [[f32; 4]; 4] {
    let mut matrix = [[0.0; 4]; 4];
    matrix[0][0] = factor;
    matrix[1][1] = factor;
    matrix[2][2] = factor;
    matrix[3][3] = 1.0;
    matrix
}

fn load_cstring(name: &str) -> Result<CString, String> {
    let content = fs::read_to_string(name);
    match content {
        Ok(s) => {
            let cstring = CString::new(s);
            match cstring {
                Ok(c) => Ok(c),
                Err(e) => Err(format!("Unable to convert source to CString: {}", e)),
            }
        }
        Err(e) => Err(format!("Unable to read from file: {}", e)),
    }
}

fn load_single_shader(source: &CStr, shader_type: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut success: GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
            buffer.extend([b' '].iter().cycle().take(len as usize));
            let error: CString = CString::from_vec_unchecked(buffer);
            gl::GetShaderInfoLog(shader, len, &mut len, error.as_ptr() as *mut GLchar);
            return Err(error
                .into_string()
                .expect("Error during conversion from shader error message to rust string"));
        }
        Ok(shader)
    }
}

fn load_shader() -> Result<GLuint, String> {
    unsafe {
        let vertex = load_single_shader(
            &load_cstring("shaders/mandelbrot_vertex.glsl")?,
            gl::VERTEX_SHADER,
        )?;
        let fragment = load_single_shader(
            &load_cstring("shaders/mandelbrot_fragment.glsl")?,
            gl::FRAGMENT_SHADER,
        )?;

        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
            buffer.extend([b' '].iter().cycle().take(len as usize));
            let error: CString = CString::from_vec_unchecked(buffer);
            gl::GetProgramInfoLog(program, len, &mut len, error.as_ptr() as *mut GLchar);
            return Err(error
                .into_string()
                .expect("Error during conversion from shader error message to rust string"));
        } else {
            Ok(program)
        }
    }
}

fn create_vao(width: f32, height: f32) -> (GLuint, GLuint) {
    let vertices: [f32; 12] = [
        -width / 2.0,
        height / 2.0,
        -width / 2.0,
        -height / 2.0,
        width / 2.0,
        -height / 2.0,
        -width / 2.0,
        height / 2.0,
        width / 2.0,
        -height / 2.0,
        width / 2.0,
        height / 2.0,
    ];
    let mut vbo: GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * std::mem::size_of::<f32>() as i32,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        (vbo, vao)
    }
}

fn get_uniform_location(program: GLuint, name: String) -> GLint {
    unsafe {
        gl::GetUniformLocation(
            program,
            CString::new(name)
                .expect("Variable name is not a valid CString")
                .as_ptr(),
        )
    }
}

fn set_uniform_int(location: GLint, value: GLint) {
    unsafe {
        gl::Uniform1i(location, value);
    }
}

fn set_uniform_mat4(location: GLint, value: [[f32; 4]; 4]) {
    unsafe {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr() as *const GLfloat);
    }
}

fn set_uniform_float2(location: GLint, value1: f32, value2: f32) {
    unsafe {
        gl::Uniform2f(location, value1, value2);
    }
}

fn render() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }
}

fn main() -> Result<(), String> {
    let (width, height): (f32, f32) = (1200.0, 800.0);
    let mut events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Mandelbrot Renderer")
        .with_dimensions(glutin::dpi::LogicalSize::from_physical(
            glutin::dpi::PhysicalSize::new(width as f64, height as f64),
            1.0,
        ));
    let winodw_context = glutin::ContextBuilder::new()
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(wb, &events_loop)
        .ok()
        .unwrap();
    let (context, _window) = unsafe { winodw_context.split() };
    let context = unsafe { context.make_current().ok().unwrap() };
    gl::load_with(|ptr| context.get_proc_address(ptr) as *const _);

    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    };
    let shader_program = load_shader()?;
    let (vbo, vao) = create_vao(width, height);
    unsafe {
        gl::UseProgram(shader_program);
        gl::BindVertexArray(vao);
    }
    let iteration_location = get_uniform_location(shader_program, "maxIterations".to_string());
    let mut current_iterations = 200;
    set_uniform_int(iteration_location, current_iterations);

    let ortho_location = get_uniform_location(shader_program, "ortho".to_string());
    set_uniform_mat4(
        ortho_location,
        ortho(
            -width / 2.0,
            width / 2.0,
            height / 2.0,
            -height / 2.0,
            1.0,
            -1.0,
        ),
    );

    let mut left_click = false;
    let (mut x_position, mut y_position): (f32, f32) = (0.0, 0.0);
    let pos_position = get_uniform_location(shader_program, "pos".to_string());
    let mut zoomed_last_frame = false;
    set_uniform_float2(pos_position, x_position, y_position);

    let mut scale_factor = 1.0;
    let scale_position = get_uniform_location(shader_program, "scale".to_string());
    set_uniform_mat4(scale_position, scale(scale_factor));

    // Draw the first frame (because of optimizations with changed)
    render();
    context.swap_buffers().unwrap();

    let mut shoud_not_close = true;
    let mut focused = true;
    while shoud_not_close {
        let mut changed = false;
        let start = Instant::now();
        if focused {
            events_loop.poll_events(|event| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    shoud_not_close = false;
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(is_focused),
                    ..
                } => {
                    focused = is_focused;
                }
                Event::DeviceEvent {
                    event: DeviceEvent::Key(key),
                    ..
                } => {
                    if key.state == ElementState::Pressed {
                        match key.virtual_keycode {
                            Some(VirtualKeyCode::Up) => {
                                current_iterations += 10;
                                set_uniform_int(iteration_location, current_iterations);
                                changed = true;
                                println!("Current iterations: {}", current_iterations);
                            }
                            Some(VirtualKeyCode::Down) => {
                                current_iterations -= 10;
                                set_uniform_int(iteration_location, current_iterations);
                                changed = true;
                                println!("Current iterations: {}", current_iterations);
                            }
                            Some(VirtualKeyCode::Escape) => {
                                shoud_not_close = false;
                            }
                            _ => {}
                        }
                    }
                }
                Event::DeviceEvent {
                    event:
                        DeviceEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(_, y),
                        },
                    ..
                } => {
                    if y > 0.0 {
                        scale_factor *= 1.01;
                        set_uniform_mat4(scale_position, scale(scale_factor));
                    } else {
                        scale_factor /= 1.01;
                        set_uniform_mat4(scale_position, scale(scale_factor));
                    }
                    changed = true;
                    zoomed_last_frame = true;
                }
                Event::DeviceEvent {
                    event: DeviceEvent::Button { button: 1, state },
                    ..
                } => {
                    if state == ElementState::Pressed {
                        left_click = true;
                    } else {
                        left_click = false;
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if left_click {
                        x_position += delta.0 as f32 / scale_factor;
                        y_position += delta.1 as f32 / scale_factor;
                        set_uniform_float2(pos_position, x_position, y_position);
                        changed = true;
                    }
                }
                _ => {
                    zoomed_last_frame = false;
                }
            });
        } else {
            events_loop.poll_events(|event| {
                if let Event::WindowEvent {
                    event: window_event,
                    ..
                } = event
                {
                    if let WindowEvent::Focused(new_focused) = window_event {
                        focused = new_focused;
                    }
                }
            })
        }

        let elapsed = start.elapsed();
        if changed {
            render();
            context.swap_buffers().unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(
            16 - elapsed.as_millis() as u64,
        ));
    }
    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    }
    Ok(())
}
