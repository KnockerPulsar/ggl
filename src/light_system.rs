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
    light_uniform_suffix: &str,
    global_enable: &bool,
) where
    T: 'static,
{
    let Some(mut q) = ecs.query3::<Transform, T, CommonLightData>() else {
        eprintln!(
            "Attempt to query nonexistent compoenent {}",
            std::any::type_name::<T>()
        );

        lit_shader.set_int(&format!("u_num_{}", light_uniform_suffix), 0);

        return;
    };

    let num_enabled_lights = q
        .iter_mut()
        .filter_map(|(x, y, z)| {
            if let (Some(xx), Some(yy), Some(zz)) = (x, y, z) {
                Some((xx, yy, zz))
            } else {
                None
            }
        })
        .enumerate()
        .map(|(i, (transform, light, cld))| {
            light.upload_data(
                &cld,
                transform,
                &format!("u_{}[{}]", light_uniform_suffix, i),
                lit_shader,
                global_enable,
            );

            cld.enabled as i32
        })
        .sum();

    lit_shader.set_int(
        &format!("u_num_{}", light_uniform_suffix),
        num_enabled_lights,
    );
}

pub fn light_system(ecs: &mut Ecs, shader_loader: &mut ShaderLoader, r: &mut Renderer) {
    let lit_shader = &shader_loader.get_shader_rc(DEFAULT_LIT_SHADER);
    lit_shader.use_program();

    light_subsystem::<SpotLight>(lit_shader, ecs, "spot_lights", &r.lights_on);
    light_subsystem::<PointLight>(lit_shader, ecs, "point_lights", &r.lights_on);
    light_subsystem::<DirectionalLight>(lit_shader, ecs, "directional_lights", &r.lights_on);
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
