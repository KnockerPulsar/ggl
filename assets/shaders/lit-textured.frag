#version 330 core

// So, the GLSL compiler seems to optimize uniforms automatically.
// Trying to set the uniform later won't error out, but will have no effect.
// Effectively, a silent failure.
//
// For example. If there are no lights, then most of this shader is useless.
// All textures except the emissive texture are optimized out.
// So when you try to set the first diffuse texture, it renderdoc will show it 
// as the first emissive texture. 
// Querying the position of `texture_diffuse1` will not error out. Instead, it
// will return the position of `texture_emissive1`.

// ! TODO: Use multiple textures for lighting
// ! Currently, only the first of each type is used.
struct Material {
    sampler2D texture_diffuse1;
    sampler2D texture_diffuse2;
    sampler2D texture_diffuse3;

    sampler2D texture_specular1;
    sampler2D texture_specular2;
    sampler2D texture_specular3;
    
    sampler2D texture_emissive1;
    vec3 emissive_factor;
    float shininess;
}; 

struct DirectionalLight {
    vec3 direction;
  
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    bool is_enabled;
};


struct PointLight {
    vec3 position;
    
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    
    vec3 attenuation_constants; // quadratic, linear, constant  
    bool is_enabled;
};

struct SpotLight {
    vec3 position;
    vec3 direction;
    vec2 cutoff_cos; // inner and outer cutoff cosine of angles  

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    vec3 attenuation_constants; // quadratic, linear, constant  
    bool is_enabled;
};


in vec3 normal;
in vec3 frag_pos;
in vec2 tex_coord;

out vec4 frag_color;

uniform vec3 u_view_pos;


#define MAX_POINT_LIGHTS 16
uniform int u_num_point_lights;
uniform PointLight u_point_lights[MAX_POINT_LIGHTS];

#define MAX_SPOT_LIGHTS 16
uniform int u_num_spot_lights;
uniform SpotLight u_spot_lights[MAX_SPOT_LIGHTS];

#define MAX_DIRECTIONAL_LIGHTS 4
uniform int u_num_directional_lights;
uniform DirectionalLight u_directional_lights[MAX_DIRECTIONAL_LIGHTS];

// The only actual per-instance data
uniform Material u_material;

vec3 computeDirectionalLight(DirectionalLight light, vec3 normal, vec3 view_direction);
vec3 computePointLight(PointLight light, vec3 normal, vec3 frag_pos, vec3 view_direction);
vec3 computeSpotLight(SpotLight light, vec3 normal, vec3 frag_pos, vec3 view_direction);

void main() {
    
    vec3 norm = normalize(normal);
    vec3 view_direction = normalize(u_view_pos - frag_pos);

    vec3 result = vec3(0);
    
    for (int i = 0; i < u_num_point_lights; i++){
        result += 
            computePointLight(u_point_lights[i], norm, frag_pos, view_direction);
    }    
    
    for (int i = 0; i < u_num_spot_lights; i++){
        result += 
            computeSpotLight(u_spot_lights[i], norm, frag_pos, view_direction);
    }    


    for (int i = 0; i < u_num_directional_lights; i++){
        result += computeDirectionalLight(u_directional_lights[i], norm, view_direction);
    }
    
    result += texture(u_material.texture_emissive1, tex_coord).rrr * u_material.emissive_factor;
    frag_color = vec4(result, 1.0);
}

vec3 computeDirectionalLight(DirectionalLight light, vec3 normal, vec3 view_direction) {
    if(!light.is_enabled) return vec3(0.0);

    vec3 light_dir = normalize(-light.direction);
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_direction, reflect_dir), 0.0), u_material.shininess);
    
    vec3 ambient = light.ambient * texture(u_material.texture_diffuse1, tex_coord).rgb;
    vec3 diffuse = light.diffuse * diff * texture(u_material.texture_diffuse1, tex_coord).rgb;
    vec3 specular = light.specular * spec * texture(u_material.texture_specular1, tex_coord).rgb;
    
    return ambient + diffuse + specular;
}

vec3 computePointLight(PointLight light, vec3 normal, vec3 frag_pos, vec3 view_direction) {
    if(!light.is_enabled) return vec3(0.0);

    vec3 light_dir = normalize(light.position - frag_pos);    
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_direction, reflect_dir), 0.0), u_material.shininess);
    
    float dist = length(light.position - frag_pos);
    float attenuation = 1.0 / (
        light.attenuation_constants.x             // Constant anttenuation
        + light.attenuation_constants.y  * dist   // Linear attenuation 
        + light.attenuation_constants.z  * dist * dist // Quadratic attenuation
    );
    
    vec3 ambient = light.ambient * texture(u_material.texture_diffuse1, tex_coord).rgb;
    vec3 diffuse = light.diffuse * diff * texture(u_material.texture_diffuse1, tex_coord).rgb;
    vec3 specular = light.specular * spec * texture(u_material.texture_specular1, tex_coord).rgb;

    
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    
    
    return ambient + diffuse + specular;
    
}

vec3 computeSpotLight(SpotLight light, vec3 normal, vec3 frag_pos, vec3 view_direction) {
    if(!light.is_enabled) return vec3(0.0);

    vec3 light_dir = normalize(light.position - frag_pos);    
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_direction, reflect_dir), 0.0), u_material.shininess);
    
    float dist = length(light.position - frag_pos);
    float attenuation = 1.0 / (
        light.attenuation_constants.x             // Constant anttenuation
        + light.attenuation_constants.y  * dist   // Linear attenuation 
        + light.attenuation_constants.z  * dist * dist // Quadratic attenuation
    );
    
    vec3 ambient = light.ambient * texture(u_material.texture_diffuse1, tex_coord).rgb;
    vec3 diffuse = light.diffuse * diff * texture(u_material.texture_diffuse1, tex_coord).rgb;
    vec3 specular = light.specular * spec * texture(u_material.texture_specular1, tex_coord).rgb;
    
    float theta = dot(light_dir, normalize(-light.direction));

    // cos is inversely prop. with the angle
    // cos(0) =    pi/2
    // cos(pi/2) = 0

    // cos (15 deg) > cos(20 degsj)
    float intensity = smoothstep(light.cutoff_cos.y, light.cutoff_cos.x, theta);

    ambient *= attenuation ;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;
    
    
    return ambient + diffuse + specular;
    
}
