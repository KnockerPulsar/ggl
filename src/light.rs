use std::format;
use std::sync::Arc;

use crate::shader::ShaderProgram;
use nalgebra_glm::*;

pub trait Light {
    fn upload_data(
        &self,
        gl: &Arc<glow::Context>, // OpenGL context

        //* String containing uniform name into light array
        //* Example: u_point_lights[0]
        uniform_name: &str,
        shader: &ShaderProgram,
    );

    fn is_enabled(&self) -> bool;
}

pub struct LightColors {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}

pub struct DirectionalLight {
    pub enabled: bool,

    pub direction: Vec3,
    pub colors: LightColors,
}

pub struct PointLight {
    pub enabled: bool,
    pub position: Vec3,
    pub colors: LightColors,
    pub attenuation_constants: Vec3,
}

pub struct SpotLight {
    pub enabled: bool,
    pub position: Vec3,
    pub direction: Vec3,

    pub cutoff_cosines: Vec2, // ! ANGLES IN RADIANS

    pub colors: LightColors,
    pub attenuation_constants: Vec3,
}

impl Light for DirectionalLight {
    fn upload_data(
        &self,
        gl: &Arc<glow::Context>, // OpenGL context
        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        shader.set_vec3(&gl, &format!("{}.direction", uniform_name), self.direction);
        shader.set_vec3(
            &gl,
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Light for PointLight {
    fn upload_data(&self, gl: &Arc<glow::Context>, uniform_name: &str, shader: &ShaderProgram) {
        shader.set_vec3(&gl, &format!("{}.position", uniform_name), self.position);
        shader.set_vec3(
            &gl,
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.attenuation_constants", uniform_name),
            self.attenuation_constants,
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Light for SpotLight {
    fn upload_data(
        &self,
        gl: &Arc<glow::Context>, // OpenGL context
        uniform_name: &str,
        shader: &ShaderProgram,
    ) {
        shader.set_vec3(&gl, &format!("{}.position", uniform_name), self.position);
        shader.set_vec3(&gl, &format!("{}.direction", uniform_name), self.direction);
        shader.set_vec3(
            &gl,
            &format!("{}.ambient", uniform_name),
            self.colors.ambient,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.diffuse", uniform_name),
            self.colors.diffuse,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.specular", uniform_name),
            self.colors.specular,
        );
        shader.set_vec3(
            &gl,
            &format!("{}.attenuation_constants", uniform_name),
            self.attenuation_constants,
        );
        shader.set_vec2(
            &gl,
            &format!("{}.cutoff_cos", uniform_name),
            self.cutoff_cosines,
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
