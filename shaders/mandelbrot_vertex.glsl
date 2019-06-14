#version 330 core
layout (location = 0) in vec2 vertex_pos;

uniform vec2 pos;
uniform mat4 ortho;
uniform mat4 scale;

out vec2 position;

void main(){
    position.x = vertex_pos.x / 400 - 0.5;
    position.y = vertex_pos.y / 400;
    gl_Position = ortho * scale *  vec4(vertex_pos.x + pos.x, vertex_pos.y + pos.y, 0.0, 1.0);
}