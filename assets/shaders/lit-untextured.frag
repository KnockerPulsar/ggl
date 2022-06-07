#version 330 core

in vec3 normal;
in vec3 frag_pos;

out vec4 frag_color;

uniform vec3 u_object_color;
uniform vec3 u_light_color;
uniform vec3 u_light_pos;
uniform vec3 u_view_pos;

void main() {
    
    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * u_light_color;
    
    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(u_light_pos- frag_pos);    

    float diff = max(dot(norm, light_dir), 0);
    vec3 diffuse = diff * u_light_color;
    
    float specular_strength = 0.5;
    vec3 view_dir = normalize(u_view_pos - frag_pos);
    vec3 reflect_dir = reflect(-light_dir, norm);    
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
    vec3 specular = specular_strength * spec * u_light_color;
    
    vec3 result = (ambient + diffuse + specular) * u_object_color;
    
    frag_color = vec4(result, 1.0);
}
