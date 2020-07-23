#version 450
#define PI 3.1415926535897932384626433832795

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0, std140) uniform Globals {
    uint width;
    uint height;
};

void main() {
    vec2 uv = (gl_FragCoord.xy + 0.5) / vec2(width,height);
    outColor = vec4(uv, 0, 1.0);
}
