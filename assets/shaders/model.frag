#version 330

in vec3 v_Norm;
in float occl;
in vec3 v_Rgb;

out vec4 ColorBuffer;

const vec3 SUN_DIRECTION = normalize(vec3(0, 1, 0.5));
const float SUN_FRACTION = 0.3;

void main() {
    ColorBuffer = occl * vec4(v_Rgb, 1.0) * (1.0 - SUN_FRACTION + SUN_FRACTION * abs(dot(v_Norm, SUN_DIRECTION)));
}
