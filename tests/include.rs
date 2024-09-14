use vk_shader_macros::*;

#[allow(dead_code)]
const TEST: &[u32] = include_glsl!("example.vert", version: 450, optimize: size, target: vulkan1_1);

#[allow(dead_code)]
const COMPOUND_EXTENSION: &[u32] = include_glsl!("tests/compound_extension.vert.glsl");
