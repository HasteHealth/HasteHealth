use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use base64::{
    Engine as _,
    engine::general_purpose::{self, STANDARD},
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::{
    HasteHealthTenantBranding, HasteHealthTenantCustomization,
};
use haste_fhir_model::r4::generated::types::{Attachment, FHIRCode, FHIRString};
use haste_fhir_model::r4::generated::{terminology::IssueType, types::FHIRBase64Binary};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, UserRole};
use haste_repository::{Repository, admin::TenantModelAdmin, types::tenant::CreateTenant};
use std::io::Cursor;
use std::sync::Arc;

const LOGO_SIZE: u32 = 150;

fn invalid(message: impl Into<String>) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::invalid(), message.into())
}

/// Decodes a PNG logo, requires it be square, and resizes it to `LOGO_SIZE`x`LOGO_SIZE`.
fn process_png_logo(decoded: &[u8]) -> Result<Vec<u8>, OperationOutcomeError> {
    let decoded_image = image::load_from_memory_with_format(decoded, image::ImageFormat::Png)
        .map_err(|_| invalid("Logo attachment must be a valid PNG image."))?;

    if decoded_image.width() != decoded_image.height() {
        return Err(invalid(format!(
            "Logo dimensions must be square (equal width and height). The uploaded image is \
             {}x{} pixels.",
            decoded_image.width(),
            decoded_image.height()
        )));
    }

    let resized =
        decoded_image.resize_exact(LOGO_SIZE, LOGO_SIZE, image::imageops::FilterType::Lanczos3);

    let mut png_bytes = Cursor::new(Vec::new());
    resized
        .write_to(&mut png_bytes, image::ImageFormat::Png)
        .map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                "Failed to encode the resized logo image.".to_string(),
            )
        })?;

    Ok(png_bytes.into_inner())
}

pub fn tenant_customization_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    HasteHealthTenantCustomization::Input,
    HasteHealthTenantCustomization::Output,
> {
    OperationExecutor::new(
        HasteHealthTenantCustomization::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: HasteHealthTenantCustomization::Input| {
                Box::pin(async move {
                    // Allow only owners to customize the tenant.
                    if context.ctx.user.claims.user_role != UserRole::Owner {
                        return Err(OperationOutcomeError::error(
                            IssueType::forbidden(),
                            "Only owners can customize the tenant.".to_string(),
                        ));
                    }

                    let Some(mut current) = TenantModelAdmin::<CreateTenant, _, _, _, _>::read(
                        context.state.repo.as_ref(),
                        &TenantId::System,
                        &tenant.as_ref().to_string(),
                    )
                    .await?
                    else {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::not_found(),
                            "Tenant not found.".to_string(),
                        ));
                    };

                    // Absence of `name`/`logo` clears the existing value; callers that want to
                    // preserve the current value must resend it.
                    current.display_name = input.name.and_then(|n| n.value);

                    match input.logo {
                        Some(logo) => {
                            let encoded =
                                logo.data.and_then(|data| data.value).ok_or_else(|| {
                                    invalid("Logo attachment must include base64-encoded data.")
                                })?;

                            logo.contentType
                                .and_then(|c| c.value)
                                .filter(|c| c.eq_ignore_ascii_case("image/png"))
                                .ok_or_else(|| {
                                    invalid(
                                        "Logo attachment must be a PNG image (content type \
                                         image/png).",
                                    )
                                })?;

                            let decoded = STANDARD.decode(encoded.as_bytes()).map_err(|_| {
                                invalid("Logo attachment data is not valid base64.")
                            })?;

                            current.logo_data = Some(process_png_logo(&decoded)?);
                            current.logo_content_type = Some("image/png".to_string());
                        }
                        None => {
                            current.logo_data = None;
                            current.logo_content_type = None;
                        }
                    }

                    TenantModelAdmin::update(
                        context.state.repo.as_ref(),
                        &TenantId::System,
                        current,
                    )
                    .await?;

                    Ok(HasteHealthTenantCustomization::Output {})
                })
            },
        ),
    )
}

pub fn tenant_branding_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    HasteHealthTenantBranding::Input,
    HasteHealthTenantBranding::Output,
> {
    OperationExecutor::new(
        HasteHealthTenantBranding::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             _input: HasteHealthTenantBranding::Input| {
                Box::pin(async move {
                    let Some(current) = TenantModelAdmin::<CreateTenant, _, _, _, _>::read(
                        context.state.repo.as_ref(),
                        &TenantId::System,
                        &tenant.as_ref().to_string(),
                    )
                    .await?
                    else {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::not_found(),
                            "Tenant not found.".to_string(),
                        ));
                    };

                    let name = current.display_name.map(|value| FHIRString {
                        value: Some(value),
                        ..Default::default()
                    });

                    let logo = current.logo_content_type.map(|content_type| Attachment {
                        contentType: Some(Box::new(FHIRCode {
                            value: Some(content_type),
                            ..Default::default()
                        })),
                        data: Some(Box::new(FHIRBase64Binary {
                            value: current
                                .logo_data
                                .as_ref()
                                .map(|d| general_purpose::STANDARD.encode(d)),
                            ..Default::default()
                        })),
                        ..Default::default()
                    });

                    Ok(HasteHealthTenantBranding::Output { name, logo })
                })
            },
        ),
    )
}
