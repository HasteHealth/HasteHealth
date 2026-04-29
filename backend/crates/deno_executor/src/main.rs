// main.rs
use deno_core::{error::AnyError, extension};
use std::rc::Rc;

use deno_core::PollEventLoopOptions;
use deno_core::op2;

#[op2]
#[string]
async fn op_read_file(#[string] path: String) -> Option<String> {
    let contents = tokio::fs::read_to_string(path).await.ok()?;
    Some(contents)
}

#[op2]
#[string]
async fn op_write_file(#[string] path: String, #[string] contents: String) -> Option<String> {
    tokio::fs::write(path, contents).await.ok()?;
    Some("".to_string())
}

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    extension!(
        runjs,
        ops = [
            op_read_file,
            op_write_file,
        ],
        esm_entry_point = "ext:runjs/runtime.js",
        esm = [dir "src", "runtime.js"]
    );

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
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
    if let Err(error) = runtime.block_on(run_js("./example.js")) {
        eprintln!("error: {}", error);
    }
}
