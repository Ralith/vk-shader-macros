extern crate proc_macro;

mod build;

use std::path::Path;
use std::{env, fs};

use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, LitStr, Token};

use self::build::{BuildOptions, Builder, Output};

struct IncludeGlsl(Output);

impl Parse for IncludeGlsl {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_lit = input.parse::<LitStr>()?;
        let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(&path_lit.value());
        let path_str = path.to_string_lossy();

        let src = fs::read_to_string(&path).map_err(|e| syn::Error::new(path_lit.span(), e))?;

        let options = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse::<BuildOptions>()?
        } else {
            BuildOptions::default()
        };

        let builder = Builder {
            src,
            name: path_str.into_owned(),
            path: Some(path),
            span: path_lit.span(),
            options,
        };
        builder.build().map(Self)
    }
}

struct Glsl(Output);

impl Parse for Glsl {
    fn parse(input: ParseStream) -> Result<Self> {
        let options = input.parse::<BuildOptions>()?;

        if options.unterminated {
            input.parse::<Token![,]>()?;
        }

        let src_lit = input.parse::<LitStr>()?;
        let src = src_lit.value();

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let builder = Builder {
            src,
            name: "inline".to_owned(),
            path: None,
            span: src_lit.span(),
            options,
        };
        builder.build().map(Self)
    }
}

/// Compile a GLSL source file into a binary SPIR-V constant
///
/// ```
/// use vk_shader_macros::include_glsl;
/// const VERT: &[u32] = include_glsl!("example.vert");
/// ```
///
/// Due to limitations of proc macros, paths are resolved relative to the crate root.
///
/// # Options
///
/// Compile options may be specified as additional arguments. Supported options include:
/// - `kind: <kind>` - Specify shader kind. Valid kinds are the same as the recognized file
///    extensions: `vert`, `frag`, `comp`, `geom`, `tesc`, `tese`, `spvasm`, `rgen`, `rahit`,
///    `rchit`, `rmiss`, `rint`, `rcall`, `task`, and `mesh`. If omitted, kind is inferred from the
///    file's extension, or a pragma in the source.
/// - `version: <version>` - Specify GLSL version. If omitted, version must be specified in the
///    source with `#version`
/// - `strip` - Omit debug info (set as default by enabling the `strip` feature)
/// - `debug` - Force debug info, even with the `strip` feature enabled
/// - `define: <name> ["value"]` - Define the preprocessor macro `<name>` as `value`
/// - `optimize: <level>` - Specify optimization level. Supported values are: `zero`, `size`, and
///   `performance`.  If omitted, will default to `performance`.
/// - `target: <target>` - Specify target environment. Supported values: `vulkan1_0`, `vulkan1_1`,
///   `vulkan1_2`. Defaults to `vulkan1_0`.
#[proc_macro]
pub fn include_glsl(tokens: TokenStream) -> TokenStream {
    let IncludeGlsl(output) = parse_macro_input!(tokens as IncludeGlsl);
    output.expand()
}

/// Compile inline GLSL source
///
/// ```
/// use vk_shader_macros::glsl;
/// const VERT: &[u32] = glsl! {
///     version: 450, kind: vert, optimize: size, target: vulkan1_1,
///     r#"
/// #version 450
///
/// void main() {
///     gl_Position = vec4(0);
/// }
/// "#
/// };
/// ```
///
/// Because the shader kind cannot be inferred from a file extension,
/// you may need to specify it manually as the above example does or
/// add it to the source code, e.g. `#pragma shader_stage(vertex)`.
///
/// Due to limitations of proc macros, includes are resolved relative to the crate root.
///
/// # Options
///
/// See the [`include_glsl!`] macro for a list of compile options.
#[proc_macro]
pub fn glsl(tokens: TokenStream) -> TokenStream {
    let Glsl(output) = parse_macro_input!(tokens as Glsl);
    output.expand()
}
