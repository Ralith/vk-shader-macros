use vk_shader_macros::*;

#[allow(dead_code)]
const TEST: &[u32] = glsl! {
    version: 450, kind: vert, optimize: size, target: vulkan1_1;
    r#"
#version 450

// Standard include (resolved from crate root)
#include <tests/test.glsl>

void main() {
    gl_Position = test(gl_Position);
}
"#
};
