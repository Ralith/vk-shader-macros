use vk_shader_macros::*;

#[allow(dead_code)]
const TEST: &[u32] = glsl! {
    version: 450, kind: vert, optimize: size, target: vulkan1_1,
    r#"
#version 450

// Standard include (resolved from crate root)
#include <tests/test.glsl>

void main() {
    gl_Position = test(gl_Position);
}
"#
};

#[allow(dead_code)]
const NO_OPTIONS: &[u32] = glsl! {
    r#"
#version 450
#pragma shader_stage(vertex)

void main() {
    gl_Position = vec4(0);
}
"#
};
