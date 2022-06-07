#version 330 core

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
}; 
  
struct Light {
    vec3 position;
  
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

in vec3 normal;
in vec3 frag_pos;

out vec4 frag_color;

uniform vec3 u_view_pos;
uniform Material u_material;
uniform Light u_light;

void main() {
    
    vec3 ambient = u_material.ambient * u_light.ambient;
    
    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(u_light.position- frag_pos);    

    float diff = max(dot(norm, light_dir), 0);
    vec3 diffuse = (diff * u_material.diffuse) * u_light.diffuse;
    
    vec3 view_dir = normalize(u_view_pos - frag_pos);
    vec3 reflect_dir = reflect(-light_dir, norm);    

    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), u_material.shininess);
    vec3 specular =(u_material.specular * spec) * u_light.specular;
    
    vec3 result = (ambient + diffuse + specular);
    
    frag_color = vec4(result, 1.0);
}
