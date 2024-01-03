extern crate nalgebra_glm as glm;

use crate::renderer::Renderer;
use crate::{
    ecs::Ecs,
    light::{CommonLightData, LightInterface},
    light::{DirectionalLight, PointLight, SpotLight},
    loaders::{ShaderLoader, DEFAULT_LIT_SHADER},
    shader::ProgramHandle,
    transform::Transform,
};

pub fn light_subsystem<T: LightInterface>(
    lit_shader: &ProgramHandle,
    ecs: &mut Ecs,
    u_light_array: &str,
    global_enable: &bool,
) where
    T: 'static,
{
    let mut enabled_light_index = 0;
    ecs.query3::<Transform, PointLight, CommonLightData>()
        .iter_mut()
        .filter_map(|(x, y, z)| {
            if let (Some(xx), Some(yy), Some(zz)) = (x, y, z) {
                Some((xx, yy, zz))
            } else {
                None
            }
        })
        .for_each(|(transform, light, cld)| {
            light.upload_data(
                &cld,
                transform,
                &format!("{}[{}]", u_light_array, enabled_light_index),
                lit_shader,
                global_enable,
            );
            enabled_light_index += 1;
        });
}

pub fn light_system(ecs: &mut Ecs, shader_loader: &mut ShaderLoader, r: &mut Renderer) {
    let lit_shader = &shader_loader.get_shader_rc(DEFAULT_LIT_SHADER);
    lit_shader.use_program();

    light_subsystem::<SpotLight>(lit_shader, ecs, "u_spot_lights", &r.lights_on);
    light_subsystem::<PointLight>(lit_shader, ecs, "u_point_lights", &r.lights_on);
    light_subsystem::<DirectionalLight>(lit_shader, ecs, "u_directional_ligths", &r.lights_on);
}

// let rot = transform.get_rot().0;
// let (theta, phi) = (rot.x.to_radians(), rot.y.to_radians());
//
// // x = r * sin(theta) * cos(phi)
// // y = r * cos(theta)
// // z = r * sin(theta) * sin(phi)
//
// // TODO: fix this
// let dx = theta.sin() * phi.cos();
// let dy = theta.cos();
// let dz = theta.sin() * phi.sin();
//
// let bul = shader_loader.get_shader_rc(DEFAULT_UNLIT_SHADER);
// bul.use_program();
//
// bul.upload_uniforms(
//     &map! {
//         "projection" => Uniform::Mat4(camera.get_proj_matrix()),
//         "view"       => Uniform::Mat4(camera.get_view_matrix()),
//         "model"      => Uniform::Mat4(transform.get_model_matrix())
//     },
//     "",
// );
//
// r.draw_line(
//     *transform.get_pos(),
//     transform.get_pos() + vec3(-dx, -dy, -dz) * 5.0,
// );
// None
