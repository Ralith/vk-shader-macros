use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::{env, fs, mem, str};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, LitInt, LitStr, Token};

pub struct Output {
    pub sources: Vec<String>,
    pub spv: Vec<u32>,
}

impl Output {
    pub fn expand(self) -> TokenStream {
        let Self { sources, spv } = self;

        let expanded = quote! {
            {
                #({ const _FORCE_DEP: &[u8] = include_bytes!(#sources); })*
                &[#(#spv),*]
            }
        };
        TokenStream::from(expanded)
    }
}

pub struct BuildOptions {
    pub kind: Option<shaderc::ShaderKind>,
    pub version: Option<u32>,
    pub debug: bool,
    pub definitions: Vec<(String, Option<String>)>,
    pub optimization: shaderc::OptimizationLevel,
    pub target_version: u32,

    pub unterminated: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            kind: None,
            version: None,
            debug: !cfg!(feature = "strip"),
            definitions: Vec::new(),
            optimization: if cfg!(feature = "default-optimize-zero") {
                shaderc::OptimizationLevel::Zero
            } else {
                shaderc::OptimizationLevel::Performance
            },
            target_version: 1 << 22,
            unterminated: false,
        }
    }
}

impl Parse for BuildOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut out = Self::default();

        while input.peek(Ident) {
            let key = input.parse::<Ident>()?;
            match key.to_string().as_str() {
                "kind" => {
                    input.parse::<Token![:]>()?;

                    let value = input.parse::<Ident>()?;
                    if let Some(kind) = extension_kind(&value.to_string()) {
                        out.kind = Some(kind);
                    } else {
                        return Err(syn::Error::new(value.span(), "unknown shader kind"));
                    }
                }
                "version" => {
                    input.parse::<Token![:]>()?;

                    let value = input.parse::<LitInt>()?;
                    out.version = Some(value.base10_parse()?);
                }
                "strip" => {
                    out.debug = false;
                }
                "debug" => {
                    out.debug = true;
                }
                "define" => {
                    input.parse::<Token![:]>()?;

                    let name = input.parse::<Ident>()?;
                    let value = if input.peek(Token![,]) {
                        None
                    } else {
                        Some(input.parse::<LitStr>()?.value())
                    };
                    out.definitions.push((name.to_string(), value));
                }
                "optimize" => {
                    input.parse::<Token![:]>()?;

                    let value = input.parse::<Ident>()?;
                    if let Some(level) = optimization_level(&value.to_string()) {
                        out.optimization = level;
                    } else {
                        return Err(syn::Error::new(value.span(), "unknown optimization level"));
                    }
                }
                "target" => {
                    input.parse::<Token![:]>()?;

                    let value = input.parse::<Ident>()?;
                    if let Some(version) = target(&value.to_string()) {
                        out.target_version = version;
                    } else {
                        return Err(syn::Error::new(value.span(), "unknown target"));
                    }
                }
                _ => {
                    return Err(syn::Error::new(key.span(), "unknown shader compile option"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                out.unterminated = true;
                break;
            }
        }

        Ok(out)
    }
}

pub struct Builder {
    pub src: String,
    pub name: String,
    pub path: Option<PathBuf>,
    pub span: Span,
    pub options: BuildOptions,
}

impl Builder {
    pub fn build(self) -> Result<Output> {
        let Self {
            src,
            name: src_name,
            path: src_path,
            span: src_span,
            options: build_options,
        } = self;

        let path_str = src_path.clone().map(|x| x.to_string_lossy().into_owned());
        let sources = RefCell::new(path_str.map(|x| vec![x]).unwrap_or_else(Vec::new));

        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_include_callback(|name, ty, src, _depth| {
            let path = match ty {
                shaderc::IncludeType::Relative => Path::new(src).parent().unwrap().join(name),
                shaderc::IncludeType::Standard => {
                    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(name)
                }
            };
            let path_str = path.to_str().ok_or("non-unicode path")?.to_owned();
            sources.borrow_mut().push(path_str.clone());
            Ok(shaderc::ResolvedInclude {
                resolved_name: path_str,
                content: fs::read_to_string(path).map_err(|x| x.to_string())?,
            })
        });
        if let Some(version) = build_options.version {
            options.set_forced_version_profile(version, shaderc::GlslProfile::None);
        }
        for (name, value) in build_options.definitions {
            options.add_macro_definition(&name, value.as_ref().map(|x| &x[..]));
        }
        if build_options.debug {
            options.set_generate_debug_info();
        }
        options.set_optimization_level(build_options.optimization);
        options.set_target_env(shaderc::TargetEnv::Vulkan, build_options.target_version);

        let kind = build_options
            .kind
            .or_else(|| {
                src_path.and_then(|x| {
                    x.extension()
                        .and_then(|x| x.to_str().and_then(extension_kind))
                })
            })
            .unwrap_or(shaderc::ShaderKind::InferFromSource);

        let compiler = shaderc::Compiler::new().unwrap();
        let out = compiler
            .compile_into_spirv(&src, kind, &src_name, "main", Some(&options))
            .map_err(|e| syn::Error::new(src_span, e))?;
        if out.get_num_warnings() != 0 {
            return Err(syn::Error::new(src_span, out.get_warning_messages()));
        }
        mem::drop(options);

        Ok(Output {
            sources: sources.into_inner(),
            spv: out.as_binary().into(),
        })
    }
}

fn extension_kind(ext: &str) -> Option<shaderc::ShaderKind> {
    use shaderc::ShaderKind::*;
    Some(match ext {
        "vert" => Vertex,
        "frag" => Fragment,
        "comp" => Compute,
        "geom" => Geometry,
        "tesc" => TessControl,
        "tese" => TessEvaluation,
        "spvasm" => SpirvAssembly,
        "rgen" => RayGeneration,
        "rahit" => AnyHit,
        "rchit" => ClosestHit,
        "rmiss" => Miss,
        "rint" => Intersection,
        "rcall" => Callable,
        "task" => Task,
        "mesh" => Mesh,
        _ => {
            return None;
        }
    })
}

fn optimization_level(level: &str) -> Option<shaderc::OptimizationLevel> {
    match level {
        "zero" => Some(shaderc::OptimizationLevel::Zero),
        "size" => Some(shaderc::OptimizationLevel::Size),
        "performance" => Some(shaderc::OptimizationLevel::Performance),
        _ => None,
    }
}

fn target(s: &str) -> Option<u32> {
    Some(match s {
        "vulkan" | "vulkan1_0" => 1 << 22,
        "vulkan1_1" => 1 << 22 | 1 << 12,
        "vulkan1_2" => 1 << 22 | 2 << 12,
        "vulkan1_3" => 1 << 22 | 3 << 12,
        _ => return None,
    })
}
