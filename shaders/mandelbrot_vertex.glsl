#version 330 core
layout (location = 0) in vec2 vertex_pos;

uniform vec2 pos;
uniform mat4 ortho;

out vec2 position;

void main(){
    position = vertex_pos.xy;
    gl_Position = ortho * vec4(vertex_pos.x + pos.x, vertex_pos.y + pos.y, 0.0, 1.0);
}