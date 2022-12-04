#version 450

layout (location=0) in vec4 point;
layout (location=1) in vec4 color_input;

layout (location=0) out vec4 color;

void main() {
    gl_PointSize=5.0;

    gl_Position = point;
    color=color_input;
}
