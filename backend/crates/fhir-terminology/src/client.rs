use crate::FHIRTerminology;
use oxidized_fhir_client::{
    FHIRClient, // url::{Parameter, ParsedParameter},
    url::{Parameter, ParsedParameter},
};
use oxidized_fhir_generated_ops::generated::{
    CodeSystemLookup, ValueSetExpand, ValueSetValidateCode,
};
use oxidized_fhir_model::r4::types::{Resource, ResourceType, ValueSet};
// use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use std::{marker::PhantomData, sync::Arc};

pub struct FHIRClientTerminology<CTX, Error, Client: FHIRClient<CTX, Error>> {
    _ctx: PhantomData<CTX>,
    _error: PhantomData<Error>,
    client: Arc<Box<Client>>,
}

async fn resolve_valueset<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: Arc<Box<Client>>,
    ctx: CTX,
    input: &ValueSetExpand::Input,
) -> Result<Option<ValueSet>, OperationOutcomeError> {
    if let Some(valueset) = input.valueSet.as_ref() {
        return Ok(Some(valueset.clone()));
    } else if let Some(url) = &input.url.as_ref().and_then(|u| u.value.as_ref()) {
        let mut result = client
            .search_type(
                ctx,
                ResourceType::ValueSet,
                vec![ParsedParameter::Resource(Parameter {
                    name: "url".to_string(),
                    value: vec![url.to_string()],
                    modifier: None,
                    chains: None,
                })],
            )
            .await?;
        if result.len() > 1 {
            return Err(OperationOutcomeError::error(
                OperationOutcomeCodes::Duplicate,
                format!("Multiple ValueSet resources found for url {}", url),
            ));
        } else if let Some(resource) = result.pop() {
            return match resource {
                Resource::ValueSet(vs) => Ok(Some(vs)),
                _ => Ok(None),
            };
        }
    }

    Ok(None)
}

impl<CTX: Send + Sync, Client: FHIRClient<CTX, OperationOutcomeError>> FHIRTerminology<CTX>
    for FHIRClientTerminology<CTX, OperationOutcomeError, Client>
{
    async fn expand(
        &self,
        ctx: CTX,
        input: &ValueSetExpand::Input,
    ) -> Result<ValueSetExpand::Output, OperationOutcomeError> {
        // Implementation would go here
        let _result = self
            .client
            .search_type(
                ctx,
                ResourceType::ValueSet,
                vec![ParsedParameter::Resource(Parameter {
                    name: "url".to_string(),
                    value: vec!["http://example.org/fhir/ValueSet/example".to_string()],
                    modifier: None,
                    chains: None,
                })],
            )
            .await;

        unimplemented!();
    }
    async fn validate(
        &self,
        _ctx: CTX,
        _input: &ValueSetValidateCode::Input,
    ) -> Result<ValueSetValidateCode::Output, OperationOutcomeError> {
        // Implementation would go here
        unimplemented!()
    }
    async fn lookup(
        &self,
        _ctx: CTX,
        _input: &CodeSystemLookup::Input,
    ) -> Result<CodeSystemLookup::Output, OperationOutcomeError> {
        // Implementation would go here
        unimplemented!()
    }
}
