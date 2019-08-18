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
const FRAG: &[u32] = include_glsl!("shaders/example.glsl", kind: frag);
```

Debug info is generated by default; pass `strip` to the macro to omit
it, or build the create with the `strip` feature enabled.

## Why `[u32]`?

SPIR-V is a stream of 32-bit words, not bytes, and this is reflected
in APIs that consume it. In particular, passing a `[u8]` of SPIR-V
that is not 4-byte-aligned to Vulkan is undefined behavior. Storing
SPIR-V in its native format guarantees that this will never occur,
without requiring copying or unsafe code.

## Dependencies

This crate currently depends on the foreign
[shaderc](https://github.com/google/shaderc/) library. By default, it
will attempt to use an installed shaderc library. However if it does
not exist, it will fall back to building from source, requiring git,
cmake, python 3, and a supported C++ compiler to be available in the
build environment. When using a pre-compiled shaderc, care must be
taken to use a version that is binary-compatible with the one checked
out by [the shaderc crate](https://github.com/google/shaderc-rs).
You can force shaderc to be built from source by enabling the
`build-from-source` feature on vk-shader-macros.
