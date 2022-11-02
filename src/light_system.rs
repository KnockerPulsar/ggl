extern crate nalgebra_glm as glm;

use crate::{
    ecs::Ecs,
    light::Light,
    light::{DirectionalLight, LightColors, PointLight, SpotLight},
    shader::ShaderProgram,
    transform::Transform
};

use std::cell::RefMut;

pub fn light_subsystem<T: Light>(
    lit_shader: &ShaderProgram,
    transforms: &mut RefMut<Vec<Option<Transform>>>,
    spot_lights: &mut RefMut<Vec<Option<T>>>,
    u_name_light_num: &str,
    u_light_array: &str,
) {
    let enabled_count = spot_lights
        .iter()
        // Filter out None lights or disabled lights
        .filter(|l| {
            if let Some(light) = *l {
                light.is_enabled()
            } else {
                false
            }
        })
        .count() as i32;

    lit_shader.set_int(u_name_light_num, enabled_count);

    let zip = spot_lights.iter_mut().zip(transforms.iter_mut());
    let mut enabled_light_index = 0;

    // Loop over all light and transform components
    // Note that some entities might have one or none. In this case light/transform
    // Will be None
    for (light, transform) in zip {
        // If an entity has both, draw egui and upload its data
        if let (Some(l), Some(t)) = (light, transform) {
            l.upload_data(
                t,
                &format!("{}[{}]", u_light_array, enabled_light_index),
                lit_shader,
            );

            enabled_light_index += 1;
        }
    }
}

pub fn light_system(ecs: &mut Ecs, lit_shader: &ShaderProgram) {
    if let Some(mut transforms) = ecs.borrow_comp_vec::<Transform>() {
        if let Some(mut spot_lights) = ecs.borrow_comp_vec::<SpotLight>() {
            light_subsystem::<SpotLight>(
                lit_shader,
                &mut transforms,
                &mut spot_lights,
                "u_num_spot_lights",
                "u_spot_lights",
            );
        }

        if let Some(mut point_lights) = ecs.borrow_comp_vec::<PointLight>() {
            light_subsystem::<PointLight>(
                lit_shader,
                &mut transforms,
                &mut point_lights,
                "u_num_point_lights",
                "u_point_lights",
            );
        }

        if let Some(mut directional_lights) = ecs.borrow_comp_vec::<DirectionalLight>() {
            let zip = directional_lights.iter_mut().zip(transforms.iter_mut());

            // Loop over all light and transform components
            // Note that some entities might have one or none. In this case light/transform
            // Will be None
            for (light, transform) in zip {
                // If an entity has both, draw egui and upload its data
                if let (Some(l), Some(t)) = (light, transform) {
                    if l.is_enabled() {
                        l.upload_data(t, "u_directional_light", lit_shader);
                    } else {
                        DirectionalLight {
                            enabled: false,
                            colors: LightColors {
                                ambient: glm::Vec3::zeros(),
                                diffuse: glm::Vec3::zeros(),
                                specular: glm::Vec3::zeros(),
                            },
                        }
                        .upload_data(
                            t,
                            "u_directional_light",
                            lit_shader,
                        );
                    }
                    break;
                }
            }
        }
    }
}
