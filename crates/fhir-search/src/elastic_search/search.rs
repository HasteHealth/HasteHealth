use oxidized_fhir_client::url::ParsedParameter;
use oxidized_fhir_repository::VersionIdRef;

pub enum QueryBuildError {}

fn build_elastic_search(
    parameters: Vec<ParsedParameter>,
) -> Result<Vec<VersionIdRef>, QueryBuildError> {
}
