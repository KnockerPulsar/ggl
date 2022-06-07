#version 330 core

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    sampler2D emissive;
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
in vec2 tex_coord;

out vec4 frag_color;

uniform vec3 u_view_pos;
uniform Material u_material;
uniform Light u_light;

void main() {
    
    
    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(u_light.position- frag_pos);    

    vec3 view_dir = normalize(u_view_pos - frag_pos);
    vec3 reflect_dir = reflect(-light_dir, norm);    

    float diff = max(dot(norm, light_dir), 0);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), u_material.shininess);

    vec3 ambient =  u_light.ambient * texture(u_material.diffuse, tex_coord).rgb;
    vec3 diffuse =  diff * u_light.diffuse * texture(u_material.diffuse, tex_coord).rgb;
    vec3 specular = spec * u_light.specular * texture(u_material.specular, tex_coord).rgb ;
    vec3 emissive = texture(u_material.emissive, tex_coord).rgb;
    
    vec3 result = (ambient + diffuse + specular + emissive);
    
    frag_color = vec4(result, 1.0);
}
