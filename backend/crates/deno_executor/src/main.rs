use deno_core::error::ModuleLoaderError;
use deno_core::{
    ModuleLoadOptions, ModuleLoadReferrer, ModuleLoader, ModuleSource, ModuleType, resolve_import,
};
// main.rs
use deno_core::{error::AnyError, extension};
use std::rc::Rc;

use deno_ast::{MediaType, ModuleSpecifier};
use deno_ast::{ParseParams, SourceMapOption};
use deno_core::ModuleLoadResponse;
use deno_core::ModuleSourceCode;
// use deno_core::op2;
use deno_error::JsErrorBox;

// #[op2]
// #[string]
// async fn op_read_file(#[string] path: String) -> Option<String> {
//     let contents = tokio::fs::read_to_string(path).await.ok()?;
//     Some(contents)
// }

// #[op2]
// #[string]
// async fn op_write_file(#[string] path: String, #[string] contents: String) -> Option<String> {
//     tokio::fs::write(path, contents).await.ok()?;
//     Some("".to_string())
// }

struct TsModuleLoader;
impl ModuleLoader for TsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<deno_core::ModuleSpecifier, ModuleLoaderError> {
        resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        fn load(module_specifier: &ModuleSpecifier) -> Result<ModuleSource, ModuleLoaderError> {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| JsErrorBox::generic("Only file:// URLs are supported."))?;

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match MediaType::from_path(&path) {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (ModuleType::JavaScript, true),
                MediaType::Json => (ModuleType::Json, false),
                _ => {
                    return Err(JsErrorBox::generic(format!(
                        "Unknown extension {:?}",
                        path.extension()
                    )));
                }
            };

            let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;
            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.clone(),
                    text: code.into(),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })
                .map_err(JsErrorBox::from_err)?;
                let res = parsed
                    .transpile(
                        &deno_ast::TranspileOptions {
                            imports_not_used_as_values: deno_ast::ImportsNotUsedAsValues::Remove,
                            decorators: deno_ast::DecoratorsTranspileOption::Ecma,
                            ..Default::default()
                        },
                        &deno_ast::TranspileModuleOptions { module_kind: None },
                        &deno_ast::EmitOptions {
                            source_map: SourceMapOption::Separate,
                            inline_sources: true,
                            ..Default::default()
                        },
                    )
                    .map_err(JsErrorBox::from_err)?;
                let res = res.into_source();
                // let source_map = res.source_map.unwrap().into_bytes();

                res.text
            } else {
                code
            };
            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                module_specifier,
                None,
            ))
        }

        ModuleLoadResponse::Sync(load(module_specifier))
    }
}

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    extension!(
        runjs,
        // ops = [
        // ],
        esm_entry_point = "ext:runjs/runtime.js",
        esm = [dir "src", "runtime.js"]
    );

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(TsModuleLoader)),
        extensions: vec![runjs::init()],
        ..Default::default()
    });

    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;

    result.await?;

    Ok(())
}

// main.rs
fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    if let Err(error) = runtime.block_on(run_js("./example.ts")) {
        eprintln!("error: {}", error);
    }
}
