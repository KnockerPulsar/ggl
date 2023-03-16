extern crate nalgebra_glm as glm;

use glm::{Vec3, vec3};

use std::collections::HashMap;
use crate::{
    ecs::Ecs,
    light::Light,
    light::{DirectionalLight, PointLight, SpotLight},
    shader::{ShaderProgram, Uniform},
    transform::{Transform, Degree3}, renderer::Renderer, loaders::{ShaderLoader, DEFAULT_LIT_SHADER, DEFAULT_UNLIT_SHADER}, map, camera::Camera
};

pub fn light_subsystem<T: Light>(
    lit_shader: &ShaderProgram,
    ecs: &mut Ecs,
    u_light_array: &str,
    global_enable: &bool
) 
    where T: 'static
{
    let mut enabled_light_index = 0;
    ecs.do_all_mut::<Transform, T, ()>(|transform, light| {
        light.upload_data(
            transform,
            &format!("{}[{}]", u_light_array, enabled_light_index),
            lit_shader,
            global_enable
        );
        enabled_light_index += 1;

        None
    });
}

pub fn light_system(ecs: &mut Ecs, shader_loader: &mut ShaderLoader, r: &mut Renderer, camera: &Camera) {
    let lit_shader = &shader_loader.get_shader_rc(DEFAULT_LIT_SHADER);
    lit_shader.use_program();

    light_subsystem::<SpotLight>(
        lit_shader,
        ecs,
        "u_spot_lights",
        &r.lights_on
    );

    light_subsystem::<PointLight>(
        lit_shader,
        ecs,
        "u_point_lights",
        &r.lights_on
    );


    ecs.do_one::<Transform, DirectionalLight, ()>(|transform, directional_light| {
        directional_light.upload_data(
            transform, 
            "u_directional_light", 
            lit_shader, 
            &(r.lights_on && directional_light.is_enabled())
        );

        let rot = transform.get_rot().0;
        let (theta, phi) = (rot.x.to_radians(), rot.y.to_radians());

        // x = r * sin(theta) * cos(phi)
        // y = r * cos(theta)
        // z = r * sin(theta) * sin(phi)
        
        // TODO: fix this
        let dx = theta.sin() * phi.cos();
        let dy = theta.cos();
        let dz = theta.sin() * phi.sin();

        let bul = shader_loader.get_shader_rc(DEFAULT_UNLIT_SHADER);
        bul.use_program();

        bul.upload_uniforms(&map! {
            "projection" => Uniform::Mat4(camera.get_proj_matrix()),
            "view"       => Uniform::Mat4(camera.get_view_matrix()),
            "model"      => Uniform::Mat4(transform.get_model_matrix())
        }, "");

        r.draw_line(*transform.get_pos(), transform.get_pos() + vec3(-dx, -dy, -dz) * 5.0);
        None
    });
}
