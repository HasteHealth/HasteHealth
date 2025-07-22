use axum::http::Method;
use fhir_client::request::{
    FHIRBatchRequest, FHIRConditionalUpdateRequest, FHIRCreateRequest, FHIRDeleteSystemRequest,
    FHIRDeleteTypeRequest, FHIRHistorySystemRequest, FHIRInvokeSystemRequest, FHIRRequest,
    FHIRSearchSystemRequest, FHIRTransactionRequest, Operation,
};
use fhir_model::r4::types::{Bundle, Resource, ResourceType};
use serde_json::error;
use thiserror::Error;

use crate::SupportedFHIRVersions;

pub struct HTTPRequest {
    method: Method,
    path: String,
    body: String,
}
impl HTTPRequest {
    pub fn new(method: Method, path: String, body: String) -> Self {
        HTTPRequest { method, path, body }
    }
}

#[derive(Error, Debug)]
pub enum FHIRRequestParsingError {
    #[error("Invalid HTTP method")]
    InvalidMethod,
    #[error("Invalid FHIR path")]
    InvalidPath,
    #[error("Invalid FHIR body")]
    InvalidBody,
    #[error("Unsupported FHIR request '{0}'")]
    Unsupported(String),
}

/*
search-system	      ?	                                  GET	N/A	N/A	N/A	N/A

capabilities	      /metadata	                          GET‡	N/A	N/A	N/A	N/A
create         	    /[type]                           	POST	R	Resource	O	O: If-None-Exist
search-type	        /[type]?                           	GET	N/A	N/A	N/A	N/A
search-system       /_search	                          POST	application/x-www-form-urlencoded	form data	N/A	N/A
delete-conditional	/[type]?	                          DELETE	N/A	N/A	N/A	O: If-Match
update-conditional  /[type]?                            PUT	R	Resource	O	O: If-Match
history-system	    /_history	                          GET	N/A	N/A	N/A	N/A
(operation)	        /$[name]                            POST	R	Parameters	N/A	N/A
                                                        GET	N/A	N/A	N/A	N/A
                                                        POST	application/x-www-form-urlencoded	form data	N/A	N/A
*/
fn parse_request_1_non_empty<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    if url_chunks[0].starts_with("$") {
        match req.method {
            Method::POST => {
                // Handle operation request
                Ok(FHIRRequest::InvokeSystem(FHIRInvokeSystemRequest {
                    operation: Operation::new(url_chunks[0])?,
                    parameters: fhir_serialization_json::from_str(&req.body)?,
                }))
            }
            Method::GET => {
                // Handle operation request
                Err(FHIRRequestParsingError::Unsupported(
                    "GET operation requests are not supported".to_string(),
                )
                .into())
            }
            _ => Err(FHIRRequestParsingError::Unsupported(
                "Invalid method for invocation".to_string(),
            )
            .into()),
        }
    } else {
        match req.method {
            Method::POST => {
                match url_chunks[0] {
                    "_search" => Err(FHIRRequestParsingError::Unsupported(
                        "POST search requests are not supported".to_string(),
                    )
                    .into()),
                    _ => {
                        // Handle create request
                        Ok(FHIRRequest::Create(FHIRCreateRequest {
                            resource_type: ResourceType::new(url_chunks[0].to_string())?,
                            resource: fhir_serialization_json::from_str::<Resource>(&req.body)?,
                        }))
                    }
                }
            }
            Method::PUT => Ok(FHIRRequest::ConditionalUpdate(
                FHIRConditionalUpdateRequest {
                    parameters: vec![],
                    resource_type: ResourceType::new(url_chunks[0].to_string())?,
                    resource: fhir_serialization_json::from_str::<Resource>(&req.body)?,
                },
            )),
            Method::DELETE => Ok(FHIRRequest::DeleteType(FHIRDeleteTypeRequest {
                parameters: vec![],
                resource_type: ResourceType::new(url_chunks[0].to_string())?,
            })),
            Method::GET => {
                match url_chunks[0] {
                    "capabilities" => {
                        // Handle capabilities request
                        Ok(FHIRRequest::Capabilities)
                    }
                    "_history" => Ok(FHIRRequest::HistorySystem(FHIRHistorySystemRequest {
                        parameters: vec![],
                    })),
                    _ => {
                        // Handle search request
                        Ok(FHIRRequest::SearchType(
                            fhir_client::request::FHIRSearchTypeRequest {
                                resource_type: ResourceType::new(url_chunks[0].to_string())?,
                                parameters: vec![],
                            },
                        ))
                    }
                }
            }
            _ => Err(FHIRRequestParsingError::Unsupported(
                "Unsupported method for FHIR request".to_string(),
            )
            .into()),
        }
    }
}

/*
transaction	        /	                                  POST	R	Bundle	O	N/A
batch	              /	                                  POST	R	Bundle	O	N/A
search-system	      ?	                                  GET	N/A	N/A	N/A	N/A
delete-conditional  ?                                   DELETE N/A N/A N/A O: If-Match
*/
fn parse_request_1_empty<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    match req.method {
        Method::POST => {
            let bundle = fhir_serialization_json::from_str::<Bundle>(&req.body)?;

            match bundle.type_.value.as_ref().map(|s| s.as_str()) {
                Some("transaction") => {
                    // Handle transaction request
                    Ok(FHIRRequest::Transaction(FHIRTransactionRequest {
                        resource: bundle,
                    }))
                }
                Some("batch") => {
                    // Handle batch request
                    Ok(FHIRRequest::Batch(FHIRBatchRequest { resource: bundle }))
                }
                _ => Err(FHIRRequestParsingError::Unsupported(
                    "Unsupported bundle type".to_string(),
                )
                .into()),
            }
        }
        Method::GET => {
            // Handle search system request
            Ok(FHIRRequest::SearchSystem(FHIRSearchSystemRequest {
                parameters: vec![],
            }))
        }
        Method::DELETE => Ok(FHIRRequest::DeleteSystem(FHIRDeleteSystemRequest {
            parameters: vec![],
        })),
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request".to_string(),
        )
        .into()),
    }
}

fn parse_request_1<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    if url_chunks[0] == "" {
        parse_request_1_empty(fhir_version, url_chunks, req)
    } else {
        parse_request_1_non_empty(fhir_version, url_chunks, req)
    }
}

/*
(operation)	        /[type]/$[name]                     POST	R	Parameters	N/A	N/A
                                                        GET	N/A	N/A	N/A	N/A
                                                        POST	application/x-www-form-urlencoded	form data	N/A	N/A
search-type         /[type]/_search?	                POST	application/x-www-form-urlencoded	form data	N/A	N/A
read            	/[type]/[id]	                    GET‡	N/A	N/A	N/A	O: If-Modified-Since, If-None-Match
update             	/[type]/[id]                      	PUT	R	Resource	O	O: If-Match
patch        	    /[type]/[id]                      	PATCH	R (may be a patch type)	Patch	O	O: If-Match
delete	            /[type]/[id]	                    DELETE	N/A	N/A	N/A	N/A
history-type	    /[type]/_history	                GET	N/A	N/A	N/A	N/A
*/
fn parse_request_2<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    todo!()
}
fn parse_request_3<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    todo!()
}
fn parse_request_4<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    todo!()
}

pub fn http_request_to_fhir_request(
    fhir_version: SupportedFHIRVersions,
    req: &HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    let url_pieces = req.path.split('/').collect::<Vec<&str>>();

    match url_pieces.len() {
        1 => parse_request_1(fhir_version, url_pieces, req),
        2 => parse_request_2(fhir_version, url_pieces, req),
        3 => parse_request_3(fhir_version, url_pieces, req),
        4 => parse_request_4(fhir_version, url_pieces, req),
        _ => Err(FHIRRequestParsingError::InvalidPath.into()),
    }
}
