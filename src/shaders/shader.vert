#version 450

layout (location=0) in vec4 position;

layout (location=0) out vec4 colourdata_for_the_fragmentshader;

void main() {
    gl_PointSize=10.0;
    gl_Position = position;
    colourdata_for_the_fragmentshader=vec4(0.4, 1.0, 0.5, 1.0);
}
