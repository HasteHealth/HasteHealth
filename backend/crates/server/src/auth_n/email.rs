use crate::ServerEnvironmentVariables;
use haste_config::Config;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};

pub async fn send_email(
    config: &dyn Config<ServerEnvironmentVariables>,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), OperationOutcomeError> {
    let from_address = config.get(ServerEnvironmentVariables::EmailFromAddress)?;
    let api_key = config.get(ServerEnvironmentVariables::SendGridAPIKey)?;
    let sender = Sender::new(api_key, None);

    let m = Message::new(Email::new(&from_address))
        .set_subject(subject)
        .add_content(Content::new().set_content_type("text/html").set_value(body))
        .add_personalization(Personalization::new(Email::new(to)));

    let resp = sender.send(&m).await.map_err(|e| {
        tracing::error!("Failed to send email '{}'", e);
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Failed to send email".to_string(),
        )
    })?;

    tracing::info!("Email sent status: '{}'", resp.status());

    Ok(())
}
