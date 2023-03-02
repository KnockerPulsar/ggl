#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoords;

uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;

uniform vec3 billboard_center;
uniform float billboard_size;

out vec2 tex_coord;

void main() {
    tex_coord = aTexCoords;
    vec3 vertex_pos = vec3( model * vec4(billboard_center, 1) );

    gl_Position = projection * view * vec4( vertex_pos , 1 );
    gl_Position /= (gl_Position.w * billboard_size);

    gl_Position.xy += aPos.xy;
    gl_Position.z = 0.0; // Draw above anything
} 
