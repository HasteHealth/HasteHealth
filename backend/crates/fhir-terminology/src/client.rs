use crate::FHIRTerminology;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_generated_ops::{CodeSystemLookup, ValueSetExpand, ValueSetValidateCode};
use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::marker::PhantomData;

pub struct FHIRClientTerminology<CTX, Error, Client: FHIRClient<CTX, Error>> {
    _ctx: PhantomData<CTX>,
    _error: PhantomData<Error>,
    client: Box<Client>,
}

impl<CTX: Send + Sync, Client: FHIRClient<CTX, OperationOutcomeError>> FHIRTerminology<CTX>
    for FHIRClientTerminology<CTX, OperationOutcomeError, Client>
{
    async fn expand(
        &self,
        ctx: CTX,
        _input: &ValueSetExpand::Input,
    ) -> Result<ValueSetExpand::Output, OperationOutcomeError> {
        // Implementation would go here
        let valueset = unsafe { ResourceType::unchecked("ValueSet".to_string()) };
        let _result = self.client.search_type(ctx, valueset, vec![]).await;

        panic!();
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
