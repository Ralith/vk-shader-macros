# vk-shader-macros

[![Documentation](https://docs.rs/vk-shader-macros/badge.svg)](https://docs.rs/vk-shader-macros/)
[![Crates.io](https://img.shields.io/crates/v/vk-shader-macros.svg)](https://crates.io/crates/vk-shader-macros)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

A procedural macro for compiling GLSL into SPIR-V constants.

Unlike the standard `include_bytes` macro, paths are currently
resolved relative to crate root. This is due to a temporary limitation
in the procedural macro API.

## Examples

```rust
const VERT: &[u32] = include_glsl!("shaders/example.vert");
const FRAG: &[u32] = include_glsl!("shaders/example.glsl", kind: frag, debug);
```

## Dependencies

This crate currently depends on the foreign
[shaderc](https://github.com/google/shaderc/) library. By default, it
will be compiled automatically, requiring git, cmake, python 3, and a
supported C++ compiler to be available in the build environment. A
pre-compiled shaderc can be used by disabling the crate's default
features, but care must be taken to use a version that is
binary-compatible with the one checked out by [the shaderc
crate](https://github.com/google/shaderc-rs).
