use std::{env, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, Item, Pat, ReturnType, Type, Visibility};

const JAVA_CLASS: &str = "com.rucraft.RustBridge";

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");

    let source_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src/lib.rs")
        .canonicalize()
        .unwrap();
    let source = fs::read_to_string(source_path).unwrap();
    let source_file = syn::parse_file(&source).unwrap();

    let wrappers = source_file.items.iter().filter_map(|item| match item {
        Item::Fn(function) if matches!(function.vis, Visibility::Public(_)) => {
            Some(generate_wrapper(function))
        }
        _ => None,
    });

    let generated = quote! {
        #(#wrappers)*
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("java_bridge.rs");
    fs::write(out_path, generated.to_string()).unwrap();
}

fn generate_wrapper(function: &syn::ItemFn) -> TokenStream {
    let function_name = &function.sig.ident;
    let module_name = format_ident!("__java_bridge_{}", function_name);
    let export_name = format!("Java_{}_{}", mangle_class(JAVA_CLASS), function_name);
    let mut arg_names = Vec::new();
    let mut arg_types = Vec::new();

    for input in &function.sig.inputs {
        let FnArg::Typed(argument) = input else {
            panic!("Java bridge does not support methods with self: {function_name}");
        };

        let Pat::Ident(name) = argument.pat.as_ref() else {
            panic!("Java bridge arguments must be simple names: {function_name}");
        };

        arg_names.push(name.ident.clone());
        arg_types.push(argument.ty.clone());
    }

    let body = match &function.sig.output {
        ReturnType::Default => quote! {
            crate::#function_name(#(#arg_names),*)
        },
        ReturnType::Type(_, return_type) if is_java_string_return(return_type) => quote! {
            let value = crate::#function_name(#(#arg_names),*);
            let string = env.with_env(|env| -> ::jni::errors::Result<_> {
                ::jni::objects::JString::from_str(env, ::core::convert::AsRef::<str>::as_ref(&value))
                    .map(::jni::objects::JString::into_raw)
            });

            string.resolve::<::jni::errors::ThrowRuntimeExAndDefault>()
        },
        ReturnType::Type(_, _) => quote! {
            crate::#function_name(#(#arg_names),*)
        },
    };

    let return_type = match &function.sig.output {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, return_type) if is_java_string_return(return_type) => {
            quote! { -> ::jni::sys::jstring }
        }
        ReturnType::Type(_, return_type) => quote! { -> #return_type },
    };

    let env_name = if returns_java_string(&function.sig.output) {
        quote! { mut env }
    } else {
        quote! { _env }
    };

    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module_name {
            use ::jni::EnvUnowned;
            use ::jni::objects::JClass;
            #[unsafe(export_name = #export_name)]
            pub extern "system" fn jni_export<'caller>(
                #env_name: EnvUnowned<'caller>,
                _class: JClass<'caller>,
                #(#arg_names: #arg_types),*
            ) #return_type {
                #body
            }
        }
    }
}

fn returns_java_string(output: &ReturnType) -> bool {
    match output {
        ReturnType::Type(_, return_type) => is_java_string_return(return_type),
        ReturnType::Default => false,
    }
}

fn is_java_string_return(return_type: &Type) -> bool {
    match return_type {
        Type::Path(path) => path.path.is_ident("String"),
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Path(path) => path.path.is_ident("str"),
            _ => false,
        },
        _ => false,
    }
}

fn mangle_class(class: &str) -> String {
    class
        .chars()
        .flat_map(|character| match character {
            '.' | '/' => "_".chars().collect::<Vec<_>>(),
            '_' => "_1".chars().collect(),
            ';' => "_2".chars().collect(),
            '[' => "_3".chars().collect(),
            character => vec![character],
        })
        .collect()
}
