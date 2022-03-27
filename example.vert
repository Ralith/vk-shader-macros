#version 450

// Relative include (resolved from current file)
#include "tests/test.glsl"

void main() {
    gl_Position = test(gl_Position);
}
