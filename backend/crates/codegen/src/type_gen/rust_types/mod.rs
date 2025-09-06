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
    let data_types = data_types::generate(file_paths, level)
        .map_err(|d| OperationOutcomeError::error(OperationOutcomeCodes::Exception, d))?;
    let terminology_types = terminology::generate(file_paths).await?;

    Ok(GeneratedTypes {
        terminology: terminology_types,
        resources: data_types.resources,
        types: data_types.types,
    })
}
