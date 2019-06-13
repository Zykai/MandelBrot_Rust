#version 330 core
layout (location = 0) in vec3 aPos;

uniform vec2 pos;
uniform mat4 ortho;

out vec2 position;

void main(){
    vec3 temp = vec3(aPos.x + pos.x, aPos.y + pos.y, aPos.z);
    position = aPos.xy;
    gl_Position = ortho * vec4(temp, 1.0);
}