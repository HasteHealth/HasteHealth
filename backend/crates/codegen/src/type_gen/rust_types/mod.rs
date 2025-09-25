use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use proc_macro2::TokenStream;

mod data_types;
mod terminology;

pub struct GeneratedTypes {
    pub terminology: TokenStream,
    pub resources: TokenStream,
    pub types: TokenStream,
}

pub async fn generate(
    file_paths: &Vec<String>,
    level: Option<&'static str>,
) -> Result<GeneratedTypes, OperationOutcomeError> {
    let terminology_types = terminology::generate(file_paths).await?;

    let data_types =
        data_types::generate(file_paths, level, &terminology_types.inlined_terminologies)
            .map_err(|d| OperationOutcomeError::error(OperationOutcomeCodes::Exception, d))?;

    Ok(GeneratedTypes {
        terminology: terminology_types.tokens,
        resources: data_types.resources,
        types: data_types.types,
    })
}
