#version 450
#define PI 3.1415926535897932384626433832795

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0, std140) uniform Globals {
    uint width;
    uint height;
};

layout(set = 0, binding = 1, std140) readonly buffer PixelBuffer {
    vec4 img[];
};

void main() {
    int x = int(gl_FragCoord.x);
    int y = int(height) - int(gl_FragCoord.y);
    vec4 p = img[x + y * width];
    outColor = p;
}
