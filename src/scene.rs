extern crate nalgebra_glm as glm;



use egui_gizmo::GizmoMode;
use glm::{vec3, vec2, Vec3};
use glutin::dpi::PhysicalSize;

use crate::{
    camera::Camera,
    ecs::Ecs,
    transform::{Transform, Degree3},
    light::*,
    loaders::{*, utils::Handle}, renderer::{Material}, texture::{Texture2D, TextureType}, model::Model,
};

pub struct Scene {
    pub selected_entity: Option<usize>,
    pub camera: Camera,
    pub gizmo_mode: GizmoMode,
    pub ecs: Ecs
}

impl Scene {
    pub fn empty(window_width: i32, window_height: i32) -> Self {
        let aspect = window_width as f32 / window_height as f32;
        Scene {
            selected_entity: None,
            camera: Camera::new(Vec3::zeros(), vec3(0., 1., 0.), vec2(0., 0.), aspect),
            gizmo_mode: GizmoMode::Translate,
            ecs: Ecs::new()
        }
    }

    pub fn light_test(
        window_width: i32, 
        window_height: i32,
        texture_loader: &mut TextureLoader, 
        shader_loader: &mut ShaderLoader, 
        object_loader: &mut ObjLoader
    ) -> Self {

        let mut ecs = Ecs::new();
        let _up_two = vec3(0., 2., 0.);

        // let _spot0 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             positions[2] + up_two,
        //             Degree3::xyz(0.0, 0.0, 0.),
        //             "Spotlight 0",
        //     ))
        //     .with(SpotLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(0., 3., 3.), 0.1),
        //         attenuation_constants: vec3(0.1, 0.3, 0.),
        //         cutoff_angles: vec2(10f32, 15f32),
        //     });
        //
        // let _spot1 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             vec3(-3.4, 1.7, 3.7),
        //             Degree3::xyz(-70., -25., -50.),
        //             "Spotlight 1",
        //     ))
        //     .with(SpotLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(1., 1., 1.), 0.7),
        //         attenuation_constants: vec3(0.1, 0.0, 1.0),
        //         cutoff_angles: vec2(20., 30.),
        //     });
        //
        
        let pl_tex = Texture2D::from_native_handle(texture_loader.point_light_texture(), TextureType::Diffuse, 1);
        let pl_mat = Material::billboard(shader_loader, pl_tex);
        let pl_name = "Point light billboard";
        let pl_model = object_loader.clone(DEFAULT_PLANE_NAME, pl_name);

        for mr in &mut pl_model.borrow_mut().mesh_renderers {
           mr.set_material(pl_mat.clone());
        }
        
        let _point0 = ecs
            .add_entity()
            .with(Transform::with_scale(
                    vec3(5.0, 5.0, 0.0),
                    Degree3::default(),
                    vec3(0.1, 0.1, 0.1),
                    "Point light 0",
            ))
            .with(PointLight {
                enabled: true,
                colors:  LightColors::no_ambient(vec3(2., 0., 0.), 0.1),
                attenuation_constants: vec3(0.2, 0.0, 0.5),
            })
        .with(pl_model);

        //
        // let _point1 = ecs
        //     .add_entity()
        //     .with(Transform::new(
        //             positions[1] + up_two,
        //             Degree3::default(),
        //             "Point light 1",
        //     ))
        //     .with(PointLight {
        //         enabled: true,
        //         colors: LightColors::no_ambient(vec3(0., 1., 0.), 0.07),
        //         attenuation_constants: vec3(0.1, 0.0, 1.0),
        //     });
        
        {
            let default_cube = object_loader.clone_handle(DEFAULT_CUBE_NAME);
            let _ground = ecs
                .add_entity()
                .with(
                    Transform::with_scale(
                        vec3(0., -2., 0.),
                        Degree3::xyz(0., 0., 0.), 
                        vec3(10., 1., 10.), 
                        "ground"
                    )
                ).with(Handle::clone(&default_cube));


            let positions = [
                vec3(5., 0., 5.),
                vec3(-5., 0., -5.),
                vec3(5., 0., -5.),
                vec3(-5., 0., 5.),
            ];

            positions
                .iter()
                .enumerate()
                .for_each(|(index, pos)| {
                    let transform = Transform::new(*pos, Degree3::default(), &format!("cube {}", index));
                    let _model = ecs
                        .add_entity()
                        .with(transform)
                        .with(Handle::clone(&default_cube));
                    });
        }

        let cam = Camera::new(
            glm::vec3(0.0, 1.0, 5.0f32),
            glm::vec3(0.0, 1.0, 0.0f32),
            glm::vec2(0.0, 0.0),
            window_width as f32 / window_height as f32
        );

        {
            let default_plane = object_loader.clone_handle(DEFAULT_PLANE_NAME);
            let _directional = ecs
                .add_entity()
                .with(Transform::with_scale(
                        vec3(0.0, 0.0, 0.0),
                        Degree3(vec3(0., 0., 0.)),
                        vec3(0.1, 0.1, 0.1),
                        "Directional Light",
                ))
                .with(DirectionalLight {
                    enabled: true,
                    colors: LightColors::default().ambient(vec3(0.1, 0.04, 0.1)).diffuse(vec3(0.5, 0.2, 0.5)),
                })
            .with::<Handle<Model>>(Handle::clone(&default_plane));
        }
        
        // let bp_path = "assets/obj/backpack.obj";
        // let bp = object_loader.load_model("Backpack", bp_path, texture_loader, shader_loader).unwrap();
        //
        // let _backpack = ecs
        //     .add_entity()
        //     .with(Transform::default().set_name("Backpack").clone())
        //     .with::<Handle<Model>>(Handle::clone(&bp));

        Scene {
            selected_entity: None,
            camera: cam,
            gizmo_mode: GizmoMode::Translate,
            ecs
        }

    }

    pub fn window_size_changed(&mut self, inner_size: &PhysicalSize<u32>) {
        self.camera.update_aspect_ratio(inner_size.width as f32, inner_size.height as f32);
    }
}

