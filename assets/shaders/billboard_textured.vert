#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoords;

uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;

out vec2 tex_coord;

void main() {
    tex_coord = aTexCoords;

    // sx    x   x   tx
    // x    sy   x   ty
    // x    x   sz   tz
    // x    x   x    x
    // Where tx, ty, and tz are the x, y, and z translations repsectively.
    //       sx, sy, and sz are the x, y, and z scales repsectively.
    vec3 billboard_center = vec3(model[0][3], model[1][3], model[2][3]);
    vec2 scale = vec2(model[0][0], model[1][1]);

    gl_Position = projection * view * model * vec4( billboard_center, 1 );
    gl_Position /= abs(gl_Position.w);

    gl_Position.xy += aPos.xy * scale;
    gl_Position.z = 0.0; // Draw above anything
} 
