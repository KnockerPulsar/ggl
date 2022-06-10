#version 330 core

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    sampler2D emissive;
    float shininess;
}; 

struct DirectionalLight {
    vec3 direction;
  
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform DirectionalLight u_directional_light;

struct PointLight {
    vec3 position;
    
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    
    vec3 attenuation_constants; // quadratic, linear, constant  
};

#define MAX_POINT_LIGHTS 16
uniform int u_num_point_lights;
uniform PointLight u_point_lights[MAX_POINT_LIGHTS];

struct SpotLight {
  vec3 position;
  vec3 direction;
  vec2 cutoff_cos; // inner and outer cutoff cosine of angles  

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  
  vec3 attenuation_constants; // quadratic, linear, constant  
};

#define MAX_SPOT_LIGHTS 16
uniform int u_num_spot_lights;
uniform SpotLight u_spot_lights[MAX_SPOT_LIGHTS];

in vec3 normal;
in vec3 frag_pos;
in vec2 tex_coord;

out vec4 frag_color;

uniform vec3 u_view_pos;
uniform Material u_material;

vec3 computeDirectionalLight(DirectionalLight light, vec3 normal, vec3 view_direction);
vec3 computePointLight(PointLight light, vec3 normal, vec3 frag_pos, vec3 view_direction);
vec3 computeSpotLight(SpotLight light, vec3 normal, vec3 frag_pos, vec3 view_direction);

void main() {
    
    vec3 norm = normalize(normal);
    vec3 view_direction = normalize(u_view_pos - frag_pos);
    
    vec3 result = computeDirectionalLight(u_directional_light, norm, view_direction);    
    
    int num_pt_lights = min(MAX_POINT_LIGHTS, u_num_point_lights);
    for (int i = 0; i < num_pt_lights; i++){
        result += computePointLight(u_point_lights[i], norm, frag_pos, view_direction);
    }    
    
    int num_spt_lights = min(MAX_SPOT_LIGHTS, u_num_spot_lights);
    for (int i = 0; i < num_spt_lights; i++){
        result += computeSpotLight(u_spot_lights[i], norm, frag_pos, view_direction);
    }    
    
    frag_color = vec4(result, 1.0);
}

vec3 computeDirectionalLight(DirectionalLight light, vec3 normal, vec3 view_direction) {
    vec3 light_dir = normalize(-light.direction);
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_direction, reflect_dir), 0.0), u_material.shininess);
    
    vec3 ambient = light.ambient * texture(u_material.diffuse, tex_coord).rgb;
    vec3 diffuse = light.diffuse * diff * texture(u_material.diffuse, tex_coord).rgb;
    vec3 specular = light.specular * spec * texture(u_material.specular, tex_coord).rgb;
    vec3 emissive = texture(u_material.emissive, tex_coord).rgb;
    
    return ambient + diffuse + specular /*+ emissive*/;
}

vec3 computePointLight(PointLight light, vec3 normal, vec3 frag_pos, vec3 view_direction) {
    vec3 light_dir = normalize(light.position - frag_pos);    
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_direction, reflect_dir), 0.0), u_material.shininess);
    
    float dist = length(light.position - frag_pos);
    float attenuation = 1.0 / (
        light.attenuation_constants.z             // Constant anttenuation
        + light.attenuation_constants.y  * dist   // Linear attenuation 
        + light.attenuation_constants.x  * dist * dist // Quadratic attenuation
    );
    
    vec3 ambient = light.ambient * texture(u_material.diffuse, tex_coord).rgb;
    vec3 diffuse = light.diffuse * diff * texture(u_material.diffuse, tex_coord).rgb;
    vec3 specular = light.specular * spec * texture(u_material.specular, tex_coord).rgb;
    vec3 emissive = texture(u_material.emissive, tex_coord).rgb;

    
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    
    
    return ambient + diffuse + specular /*+ emissive*/;
    
}

vec3 computeSpotLight(SpotLight light, vec3 normal, vec3 frag_pos, vec3 view_direction) {
    vec3 light_dir = normalize(light.position - frag_pos);    
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_direction, reflect_dir), 0.0), u_material.shininess);
    
    float dist = length(light.position - frag_pos);
    float attenuation = 1.0 / (
        light.attenuation_constants.z             // Constant anttenuation
        + light.attenuation_constants.y  * dist   // Linear attenuation 
        + light.attenuation_constants.x  * dist * dist // Quadratic attenuation
    );
    
    vec3 ambient = light.ambient * texture(u_material.diffuse, tex_coord).rgb;
    vec3 diffuse = light.diffuse * diff * texture(u_material.diffuse, tex_coord).rgb;
    vec3 specular = light.specular * spec * texture(u_material.specular, tex_coord).rgb;
    vec3 emissive = texture(u_material.emissive, tex_coord).rgb;
    
    float theta = dot(light_dir, normalize(-light.direction));

    // cos is inversely prop. with the angle
    // cos(0) =    pi/2
    // cos(pi/2) = 0

    // cos (15 deg) > cos(20 degsj)
    float intensity = smoothstep(light.cutoff_cos.y, light.cutoff_cos.x, theta);

    ambient *= attenuation ;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;
    
    
    return ambient + diffuse + specular /*+ emissive*/;
    
}