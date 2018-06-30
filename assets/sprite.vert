#version 330 core

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 uv;

out vec2 TexCoords;

uniform mat4 model;
uniform mat4 projection;

void main() {
    TexCoords = uv;
    gl_Position = projection * model * vec4(pos, 0.0, 1.0);
}
