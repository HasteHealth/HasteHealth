use deno_ast::MediaType;
use haste_deno_executor;
use haste_fhir_client::http::{FHIRHttpClient, FHIRHttpState};

// main.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let api_url = args.get(1).expect("API URL argument is required");

    let http_fhir_client = FHIRHttpClient::<Option<String>>::new(
        FHIRHttpState::new(api_url, None).expect("Failed to create FHIR client"),
    );

    if let Err(error) = runtime.block_on(haste_deno_executor::run_code(
        None,
        http_fhir_client,
        MediaType::TypeScript,
        r#"
const patient = await readResource("Patient", "90277570");

console.log(patient.id);
console.log(patient.name);

export {};

interface Person {
  name: string;
  age: number;
}

function hello(t: Person) {
  console.log(`Hello, ${t.name}! You are ${t.age} years old.`);
}

hello({ name: "Alice", age: 30 });
        "#
        .to_string(),
    )) {
        eprintln!("error: {}", error);
    }
}
