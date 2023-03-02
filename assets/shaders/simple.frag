#version 330 core

in vec2 tex_coord;

uniform sampler2D texture_diffuse1;

void main(){
    gl_FragColor = texture(texture_diffuse1, tex_coord);
}

