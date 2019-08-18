extern crate proc_macro;

use std::{env, fs, str, mem};
use std::path::Path;
use std::cell::RefCell;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, LitStr, LitInt, Token};

struct IncludeGlsl {
    sources: Vec<String>,
    spv: Vec<u32>,
}

impl Parse for IncludeGlsl {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_lit = input.parse::<LitStr>()?;
        let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(&path_lit.value());
        let path_str = path.to_string_lossy();

        let sources = RefCell::new(vec![path_str.clone().into_owned()]);
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_include_callback(|name, ty, src, _depth| {
            let path = match ty {
                shaderc::IncludeType::Relative => Path::new(src).parent().unwrap().join(name),
                shaderc::IncludeType::Standard => Path::new(env!("CARGO_MANIFEST_DIR")).join(name),
            };
            let path_str = path.to_str().ok_or("non-unicode path")?.to_owned();
            sources.borrow_mut().push(path_str.clone());
            Ok(shaderc::ResolvedInclude {
                resolved_name: path_str,
                content: fs::read_to_string(path).map_err(|x| x.to_string())?,
            })
        });

        let mut kind = None;
        let mut debug = !cfg!(feature = "strip");
        let mut optimization = shaderc::OptimizationLevel::Performance;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let key = input.parse::<Ident>()?;
            match &key.to_string()[..] {
                "kind" => {
                    input.parse::<Token![:]>()?;
                    let value = input.parse::<Ident>()?;
                    if let Some(x) = extension_kind(&value.to_string()) {
                        kind = Some(x);
                    } else {
                        return Err(syn::Error::new(value.span(), "unknown shader kind"));
                    }
                }
                "version" => {
                    input.parse::<Token![:]>()?;
                    let x = input.parse::<LitInt>()?;
                    options.set_forced_version_profile(x.base10_parse::<u32>()?, shaderc::GlslProfile::None);
                }
                "strip" => {
                    debug = false;
                }
                "debug" => {
                    debug = true;
                }
                "define" => {
                    input.parse::<Token![:]>()?;
                    let name = input.parse::<Ident>()?;
                    let value = if input.peek(Token![,]) {
                        None
                    } else {
                        Some(input.parse::<LitStr>()?.value())
                    };
                    
                    options.add_macro_definition(&name.to_string(), value.as_ref().map(|x| &x[..]));
                }
                "optimize" => {
                    input.parse::<Token![:]>()?;
                    let value = input.parse::<Ident>()?;
                    if let Some(x) = optimization_level(&value.to_string()) {
                        optimization = x;
                    } else {
                        return Err(syn::Error::new(value.span(), "unknown optimization level"));
                    }
                }
                _ => {
                    return Err(syn::Error::new(key.span(), "unknown shader compile option"));
                }
            }
        }

        if debug {
            options.set_generate_debug_info();
        }

        options.set_optimization_level(optimization);

        let kind = kind
            .or_else(|| {
                path.extension()
                    .and_then(|x| x.to_str().and_then(|x| extension_kind(x)))
            })
            .unwrap_or(shaderc::ShaderKind::InferFromSource);
        let src = fs::read_to_string(&path).map_err(|e| syn::Error::new(path_lit.span(), e))?;

        let mut compiler = shaderc::Compiler::new().unwrap();
        let out = compiler
            .compile_into_spirv(&src, kind, &path_str, "main", Some(&options))
            .map_err(|e| syn::Error::new(path_lit.span(), e))?;
        if out.get_num_warnings() != 0 {
            return Err(syn::Error::new(path_lit.span(), out.get_warning_messages()));
        }
        mem::drop(options);

        Ok(Self {
            sources: sources.into_inner(),
            spv: out.as_binary().into(),
        })
    }
}

#[proc_macro_hack]
pub fn include_glsl(tokens: TokenStream) -> TokenStream {
    let IncludeGlsl { sources, spv } = parse_macro_input!(tokens as IncludeGlsl);
    let expanded = quote! {
        {
            #({ const _FORCE_DEP: &[u8] = include_bytes!(#sources); })*
            &[#(#spv),*]
        }
    };
    TokenStream::from(expanded)
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
