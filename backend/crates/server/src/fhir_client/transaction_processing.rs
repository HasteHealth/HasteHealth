use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::resources::{Bundle, BundleEntry};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_reflect::MetaValue;
use oxidized_repository::Repository;

use crate::fhir_client::{FHIRServerClient, ServerCTX};

pub async fn transaction_processing<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    client: &FHIRServerClient<Repo, Search, Terminology>,
    ctx: ServerCTX,
    bundle_entries: Vec<BundleEntry>,
) -> Result<Bundle, OperationOutcomeError> {
    let fp_engine = oxidized_fhirpath::FPEngine::new();
    let fp_result = fp_engine
        .evaluate(
            "$this.descendants().ofType(Reference)",
            bundle_entries
                .iter()
                .map(|be| be as &dyn MetaValue)
                .collect(),
        )
        .unwrap();

    Ok(response_bundle)
}
