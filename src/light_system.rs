extern crate nalgebra_glm as glm;

use crate::{
    ecs::Ecs,
    light::Light,
    light::{DirectionalLight, PointLight, SpotLight},
    shader::ShaderProgram,
    transform::Transform
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
    ecs.do_all::<Transform, T, ()>(|transform, light| {
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

pub fn light_system(ecs: &mut Ecs, lit_shader: &ShaderProgram, global_enable: &bool) {

    light_subsystem::<SpotLight>(
        lit_shader,
        ecs,
        "u_spot_lights",
        global_enable
    );

    light_subsystem::<PointLight>(
        lit_shader,
        ecs,
        "u_point_lights",
        global_enable
    );


    ecs.do_one::<Transform, DirectionalLight, ()>(|transform, directional_light| {
        directional_light.upload_data(
            transform, 
            "u_directional_light", 
            lit_shader, 
            &(*global_enable && directional_light.is_enabled())
        );
        None
    });
}
