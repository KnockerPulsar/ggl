extern crate glow;
extern crate glutin;
extern crate image;
extern crate nalgebra_glm as glm;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::env;
use std::mem::size_of;

mod camera;
mod input;
mod shader;

use camera::Camera;
use glow::*;
use input::InputSystem;

fn main() {
    let (window_width, window_height) = (1280, 720) as (i32, i32);

    let (gl, shader_version, window, event_loop) = {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("GG OpenGl")
            .with_inner_size(glutin::dpi::LogicalSize::new(window_width, window_height));

        unsafe {
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();

            let gl =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (gl, "#version 330", window, event_loop)
        }
    };

    unsafe {
        gl.viewport(0, 0, window_width, window_height);
        gl.clear_color(0.5, 0.1, 0.5, 1.0f32);
    }

    // 3 floats for vertex position
    // 3 floats for vertex normals
    // 2 floats for texture coordinates
    let verts = vec![
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 0.0, 0.5,
        0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, -0.5, 0.5,
        -0.5, 0.0, 0.0, -1.0, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, -0.5, -0.5,
        0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0,
        0.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0,
        0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0,
        0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, 1.0, 1.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0,
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0, -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, 0.0, 0.0,
        -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5,
        -0.5, 1.0, 0.0, 0.0, 1.0, 1.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, -0.5,
        1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
        0.0, 1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, -1.0, 0.0,
        1.0, 1.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0,
        0.0, -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0,
        -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, 0.5,
        0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, -0.5, 0.5, 0.5,
        0.0, 1.0, 0.0, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0f32,
    ];

    println!(
        "Current working directory: {}",
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    );

    window.window().set_cursor_grab(true).unwrap();

    let container_diff_id = load_texture(&gl, "assets/textures/container2.png");
    let container_spec_id = load_texture(&gl, "assets/textures/container2_specular.png");
    let container_emissive_id = load_texture(&gl, "assets/textures/container2_emissive.png");

    let lit_shader = shader::ShaderProgram::new(
        &gl,
        "assets/shaders/lit-untextured.vert",
        "assets/shaders/lit-untextured.frag",
    );

    let light_shader = shader::ShaderProgram::new(
        &gl,
        "assets/shaders/lit-untextured.vert",
        "assets/shaders/light.frag",
    );

    let mut cube_vbo: glow::Buffer;
    let mut lit_vao: glow::VertexArray;
    let mut light_vao: glow::VertexArray;
    unsafe {
        // Lit object setup
        lit_shader.use_program(&gl);

        lit_vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(lit_vao));

        cube_vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            std::slice::from_raw_parts(verts.as_ptr() as *const u8, size_of::<f32>() * verts.len()),
            glow::STATIC_DRAW,
        );

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 8 * size_of::<f32>() as i32, 0);
        gl.enable_vertex_attrib_array(0);

        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 8 * size_of::<f32>() as i32, 3 * size_of::<f32>() as i32);
        gl.enable_vertex_attrib_array(1);

        gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 8 * size_of::<f32>() as i32, 6 * size_of::<f32>() as i32);
        gl.enable_vertex_attrib_array(2);

        light_shader.use_program(&gl);

        // Light setup

        light_vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(light_vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(cube_vbo));

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 8 * size_of::<f32>() as i32, 0);
        gl.enable_vertex_attrib_array(0);

        gl.enable(glow::DEPTH_TEST);
        gl.enable(glow::TEXTURE_2D);
    }

    let mut last_frame = std::time::Instant::now();

    let mut input = InputSystem::new();
    let mut camera = Camera::new(
        &glm::vec3(0.0, 0.0, 2.0f32),
        &glm::vec3(0.0, 1.0, 0.0f32),
        &glm::vec2(0.0, 0.0),
    );

    let container_pos = glm::vec3(0.0, 0.0, 0.0);
    let container_model_mat = glm::translation(&container_pos);

    let light_pos = glm::vec3(3.0, 0.0, 0.0);
    let mut light_model_mat = glm::translation(&light_pos);
    light_model_mat = glm::scale(&light_model_mat, &glm::vec3(0.05, 0.05, 0.05));

    unsafe {
        event_loop.run(move |event, win, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::LoopDestroyed => {
                    return;
                }
                Event::MainEventsCleared => {
                    window.window().request_redraw();
                }

                Event::RedrawRequested(_) => {
                    let current_frame = std::time::Instant::now();

                    input.update((current_frame - last_frame).as_secs_f32());

                    draw(
                        &mut camera,
                        &input,
                        &gl,
                        window_width,
                        window_height,
                        &lit_shader,
                        container_diff_id,
                        container_spec_id,
                        container_emissive_id,
                        light_pos,
                        container_model_mat,
                        lit_vao,
                        &light_shader,
                        light_model_mat,
                        light_vao,
                        &window,
                    );

                    input.frame_end();
                    last_frame = current_frame;
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        window.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        //TODO: Clean up opengl stuff
                        *control_flow = ControlFlow::Exit
                    }

                    WindowEvent::KeyboardInput { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::MouseInput { .. } => {
                        input.handle_events(&event);
                    }
                    _ => (),
                },
                _ => (),
            }
        });
    }
}

unsafe fn draw(
    camera: &mut Camera,
    input: &InputSystem,
    gl: &Context,
    window_width: i32,
    window_height: i32,
    lit_shader: &shader::ShaderProgram,
    container_diff_id: NativeTexture,
    container_spec_id: NativeTexture,
    container_emissive_id: NativeTexture,
    light_pos: glm::Vec3,
    container_model_mat: glm::Mat4,
    lit_vao: NativeVertexArray,
    light_shader: &shader::ShaderProgram,
    light_model_mat: glm::Mat4,
    light_vao: NativeVertexArray,
    window: &glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
) {
    camera.update(input);
    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

    let view = camera.get_view_matrix();
    let projection = glm::perspective_fov(
        camera.get_fov().to_radians(),
        window_width as f32,
        window_height as f32,
        0.1f32,
        100.0f32,
    );

    lit_shader.use_program(&gl);

    lit_shader.set_vec3(&gl, "u_view_pos", camera.get_pos());
    lit_shader.set_vec3(&gl, "u_material.specular", glm::vec3(0.5, 0.5, 0.5));
    lit_shader.set_float(&gl, "u_material.shininess", 32.0);
    lit_shader.set_int(&gl, "u_material.diffuse", 0);

    gl.active_texture(glow::TEXTURE0);
    gl.bind_texture(glow::TEXTURE_2D, Some(container_diff_id));
    lit_shader.set_int(&gl, "u_material.specular", 1);

    gl.active_texture(glow::TEXTURE1);
    gl.bind_texture(glow::TEXTURE_2D, Some(container_spec_id));
    lit_shader.set_int(&gl, "u_material.emissive", 2);

    gl.active_texture(glow::TEXTURE2);
    gl.bind_texture(glow::TEXTURE_2D, Some(container_emissive_id));

    lit_shader.set_vec3(
        &gl,
        "u_directional_light.direction",
        glm::vec3(1.0, -0.3, 1.0),
    );
    lit_shader.set_vec3(&gl, "u_directional_light.ambient", glm::vec3(0.5, 0.5, 0.1));
    lit_shader.set_vec3(&gl, "u_directional_light.diffuse", glm::vec3(0.9, 0.9, 0.2));
    lit_shader.set_vec3(
        &gl,
        "u_directional_light.specular",
        glm::vec3(1.0, 1.0, 1.0),
    );

    // lit_shader.set_int("&gl, u_num_point_lights", 2);
    // lit_shader.set_vec3(&gl, "u_point_lights[0].position", light_pos);
    // lit_shader.set_vec3(&gl, "u_point_lights[0].ambient", glm::vec3(0.1, 0.0, 0.0));
    // lit_shader.set_vec3(&gl, "u_point_lights[0].diffuse", glm::vec3(1.0, 0.0, 0.0));
    // lit_shader.set_vec3(&gl, "u_point_lights[0].specular", glm::vec3(0.0, 1.0, 1.0));
    // lit_shader.set_vec3(&gl,
    //     "u_point_lights[0].attenuation_constants",
    //     glm::vec3(1.0, 0.0, 1.0),
    // );
    // lit_shader.set_vec3(&gl,
    //     "u_point_lights[1].position",
    //     light_pos + glm::vec3(-2.0, -2.0, -2.0),
    // );
    // lit_shader.set_vec3(&gl, "u_point_lights[1].ambient", glm::vec3(0.0, 0.0, 0.1));
    // lit_shader.set_vec3(&gl, "u_point_lights[1].diffuse", glm::vec3(0.0, 0.0, 0.9));
    // lit_shader.set_vec3(&gl, "u_point_lights[1].specular", glm::vec3(0.0, 1.0, 0.0));
    // lit_shader.set_vec3(&gl,
    //     "u_point_lights[1].attenuation_constants",
    //     glm::vec3(0.1, 0.0, 1.0),
    // );

    // lit_shader.set_int(&gl, "u_num_spot_lights", 2);

    // lit_shader.set_vec3(&gl, "u_spot_lights[0].position", light_pos);
    // lit_shader.set_vec3(&gl, "u_spot_lights[0].direction", -light_pos);
    // lit_shader.set_vec3(&gl, "u_spot_lights[0].ambient", glm::vec3(0.1f32, 0.0, 0.0));
    // lit_shader.set_vec3(&gl, "u_spot_lights[0].diffuse", glm::vec3(10.0, 0.0, 0.0));
    // lit_shader.set_vec3(&gl, "u_spot_lights[0].specular", glm::vec3(0.0, 10.0, 10.0));
    // lit_shader.set_vec3(
    //     &gl,
    //     "u_spot_lights[0].attenuation_constants",
    //     glm::vec3(1.0, 0.0, 1.0),
    // );
    // lit_shader.set_vec2(
    //     &gl,
    //     "u_spot_lights[0].cutoff_cos",
    //     glm::vec2(5f32.cos(), 4f32.cos()),
    // );
    // lit_shader.set_vec3(
    //     &gl,
    //     "u_spot_lights[1].position",
    //     light_pos + glm::vec3(-2.0, -2.0, -2.0),
    // );
    // lit_shader.set_vec3(
    //     &gl,
    //     "u_spot_lights[0].direction",
    //     -(light_pos + glm::vec3(-2.0, -2.0, -2.0)),
    // );

    // lit_shader.set_vec3(&gl, "u_spot_lights[1].ambient", glm::vec3(0.0, 0.0, 0.1f32));
    // lit_shader.set_vec3(&gl, "u_spot_lights[1].diffuse", glm::vec3(0.0, 0.0, 9f32));
    // lit_shader.set_vec3(&gl, "u_spot_lights[1].specular", glm::vec3(0.0, 10.0, 0.0));
    // lit_shader.set_vec3(
    //     &gl,
    //     "u_spot_lights[1].attenuation_constants",
    //     glm::vec3(0.1, 0.0, 1.0),
    // );
    // lit_shader.set_vec2(
    //     &gl,
    //     "u_spot_lights[1].cutoff_cos",
    //     glm::vec2(4f32.cos(), 10f32.cos()),
    // );

    lit_shader.set_mat4(&gl, "projection", projection);
    lit_shader.set_mat4(&gl, "view", view);
    lit_shader.set_mat4(&gl, "model", container_model_mat);

    gl.bind_vertex_array(Some(lit_vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 36);

    light_shader.use_program(&gl);
    light_shader.set_mat4(&gl, "projection", projection);
    light_shader.set_mat4(&gl, "view", view);
    light_shader.set_mat4(&gl, "model", light_model_mat);

    gl.bind_vertex_array(Some(light_vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 36);

    window.swap_buffers().unwrap();
}
fn load_texture(gl: &glow::Context, path: &str) -> glow::NativeTexture {
    let texture = image::io::Reader::open(path).unwrap().decode().unwrap();

    let texture_w = texture.width() as i32;
    let texture_h = texture.height() as i32;

    let texture_handle: glow::NativeTexture;

    unsafe {
        let format = match texture.color() {
            image::ColorType::L8 => glow::RGB,
            image::ColorType::Rgb8 => glow::RGB,
            image::ColorType::Rgba8 => glow::RGBA,
            _ => {
                panic!("Unsupported color type {:?}", texture.color());
            }
        };

        println!("Loaded texture of format {:#?}", format);
        println!(
            "GL_RED = {:?}, GL_RGB = {:?}, GL_RGBA = {:?}",
            glow::RED,
            glow::RGB,
            glow::RGBA
        );

        texture_handle = gl.create_texture().unwrap();

        gl.bind_texture(glow::TEXTURE_2D, Some(texture_handle));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            format as i32,
            texture_w,
            texture_h,
            0,
            format as u32,
            glow::UNSIGNED_BYTE,
            Some(texture.as_bytes()),
        );
        gl.generate_mipmap(glow::TEXTURE_2D);

        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
    }

    texture_handle
}
