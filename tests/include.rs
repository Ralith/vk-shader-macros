use vk_shader_macros::*;

#[allow(dead_code)]
const TEST: &[u32] = include_glsl!("example.vert", version: 450, optimize: size, target: vulkan1_1);
