use axum::http::Method;
use fhir_client::request::{FHIRInvokeSystemRequest, FHIRRequest};
use serde_json::error;
use thiserror::Error;

use crate::SupportedFHIRVersions;

pub struct HTTPRequest {
    method: Method,
    path: String,
    body: String,
}

#[derive(Error, Debug)]
pub enum ParsingErrors {
    #[error("Invalid HTTP method")]
    InvalidMethod,
    #[error("Invalid FHIR path")]
    InvalidPath,
    #[error("Invalid FHIR body")]
    InvalidBody,
    #[error("Unsupported FHIR request '{}'")]
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
    req: HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    if url_chunks[0].starts_with("$") {
        match req.method {
            Method::POST => {
                // Handle operation request
                let operation_name = url_chunks[0].trim_start_matches('$');
                Ok(FHIRRequest::InvokeSystem(FHIRInvokeSystemRequest {
                    parameters: fhir_serialization_json::from_str(&req.body)?,
                }))
            }
            Method::GET => {
                // Handle operation request
                Err(ParsingErrors::Unsupported(
                    "GET operation requests are not supported".to_string(),
                )
                .into())
            }
            _ => Err(ParsingErrors::InvalidMethod.into()),
        }
    } else {
        Err(ParsingErrors::InvalidPath.into())
    }
}

fn parse_request_1<'a>(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<&'a str>,
    req: HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    if url_chunks[0] == "" {
        parse_request_1_empty(fhir_version, url_chunks, req)
    } else {
        parse_request_1_non_empty(fhir_version, url_chunks, req)
    }
}
fn parse_request_2() -> anyhow::Result<FHIRRequest> {}
fn parse_request_3() -> anyhow::Result<FHIRRequest> {}
fn parse_request_4() -> anyhow::Result<FHIRRequest> {}

pub fn http_request_to_fhir_request(
    fhir_version: SupportedFHIRVersions,
    req: HTTPRequest,
) -> anyhow::Result<FHIRRequest> {
    let url_pieces = req.path.split('/').collect::<Vec<&str>>();

    match url_pieces.len() {
        1 => parse_request_1(fhir_version, url_pieces, req),
        _ => {
            let tenant = url_pieces[0].to_string();
            let project = url_pieces[1].to_string();
            let fhir_version = url_pieces[2].to_string();
            let fhir_location = url_pieces[3..].join("/");

            panic!();
        }
    }
}
