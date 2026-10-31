use crate::fhir_client::{Backend, ServerCTX};
use axum::body::Bytes;
use axum::http::Method;
use haste_fhir_client::request::{
    CompartmentRequest, DeleteRequest, FHIRBatchRequest, FHIRConditionalUpdateRequest,
    FHIRCreateRequest, FHIRDeleteInstanceRequest, FHIRDeleteSystemRequest, FHIRDeleteTypeRequest,
    FHIRHistoryInstanceRequest, FHIRHistorySystemRequest, FHIRHistoryTypeRequest, FHIRPatchRequest,
    FHIRReadRequest, FHIRRequest, FHIRSearchSystemRequest, FHIRSearchTypeRequest,
    FHIRTransactionRequest, FHIRUpdateInstanceRequest, FHIRVersionReadRequest, HistoryRequest,
    OperationParseError, SearchRequest, UpdateRequest,
};
use haste_fhir_client::url::{ParseError, ParsedParameters};
use haste_fhir_model::r4::generated::resources::{
    Bundle, Parameters, Resource, ResourceType, ResourceTypeError,
};
use haste_fhir_model::r4::generated::terminology::BundleType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_operation_error::derive::OperationOutcomeError;
use haste_fhir_serialization_json::errors::DeserializeError;
use haste_jwt::VersionId;
use haste_repository::types::SupportedFHIRVersions;
use json_patch::Patch;
use std::sync::Arc;

mod invocation;

#[derive(Debug)]
pub enum HTTPBody {
    Bytes(Bytes),
    Resource(Resource),
}

#[derive(Debug)]
pub struct HTTPRequest<'a> {
    method: Method,
    path: String,
    body: HTTPBody,
    /// The query string, without its `?`.
    query: &'a str,
}
impl<'a> HTTPRequest<'a> {
    pub fn new(method: Method, path: String, body: HTTPBody, query: &'a str) -> Self {
        HTTPRequest {
            method,
            path,
            body,
            query,
        }
    }
}

#[derive(OperationOutcomeError, Debug)]
pub enum FHIRRequestParsingError {
    #[error(code = "invalid", diagnostic = "Invalid HTTP Method")]
    InvalidMethod,
    #[error(code = "invalid", diagnostic = "Invalid Path location")]
    InvalidPath,
    #[error(code = "invalid", diagnostic = "Invalid FHIR body")]
    InvalidBody,
    #[error(
        code = "not-supported",
        diagnostic = "Unsupported FHIR request '{arg0}'"
    )]
    Unsupported(String),
    #[error(code = "invalid", diagnostic = "Invalid Resource Type '{arg0}'")]
    ResourceTypeError(#[from] ResourceTypeError),
    #[error(code = "invalid", diagnostic = "Invalid Operation '{arg0}'")]
    InvalidOperation(#[from] OperationParseError),
    #[error(code = "invalid", diagnostic = "Deserialization error: {arg0}")]
    SerdeDeserializeError(#[from] DeserializeError),
    #[error(
        code = "invalid",
        diagnostic = "Failed to deserialize resource: {arg0}"
    )]
    SerdeError(#[from] serde_json::Error),
    #[error(
        code = "invalid",
        diagnostic = "Error parsing query parameters: {arg0}"
    )]
    InvalidQueryParameters(#[from] ParseError),
}

fn get_resource(
    resource_type: &ResourceType,
    req: HTTPRequest,
) -> Result<Resource, FHIRRequestParsingError> {
    let resource = match req.body {
        HTTPBody::Resource(resource) => resource,
        HTTPBody::Bytes(body) => resource_type.deserialize(&body)?,
    };
    Ok(resource)
}

fn get_parameters(req: HTTPRequest) -> Result<Parameters, FHIRRequestParsingError> {
    let params = match req.body {
        HTTPBody::Resource(resource) => {
            if let Resource::Parameters(params) = resource {
                Ok(params)
            } else {
                return Err(FHIRRequestParsingError::InvalidBody);
            }
        }
        HTTPBody::Bytes(body) => serde_json::from_slice::<Parameters>(&body),
    }?;
    Ok(params)
}

/// Decodes the query string preserving parameter order and duplicates.
fn parse_query(query: &str) -> Result<ParsedParameters, FHIRRequestParsingError> {
    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    Ok(ParsedParameters::try_from(pairs.as_slice())?)
}

/// Search parameters for a POST `_search`, from the form-encoded body combined
/// with the query string.
///
/// Per the spec a parameter means the same thing in either place, and one that
/// appears in both is the same as repeating it.
///
/// The body is read as `application/x-www-form-urlencoded` whenever there is
/// one. An empty body leaves the query string as the whole search, which is
/// what a client that puts its parameters in the URL expects.
fn get_search_parameters_body(
    req: HTTPRequest,
) -> Result<ParsedParameters, FHIRRequestParsingError> {
    let mut pairs: Vec<(String, String)> = url::form_urlencoded::parse(req.query.as_bytes())
        .into_owned()
        .collect();

    let body = match &req.body {
        HTTPBody::Bytes(body) => body.as_ref(),
        // A search carries form data, never a resource.
        HTTPBody::Resource(_) => return Err(FHIRRequestParsingError::InvalidBody),
    };

    pairs.extend(url::form_urlencoded::parse(body).into_owned());

    Ok(ParsedParameters::try_from(pairs.as_slice())?)
}

fn get_bundle(req: HTTPRequest) -> Result<Bundle, FHIRRequestParsingError> {
    let bundle = match req.body {
        HTTPBody::Resource(resource) => {
            if let Resource::Bundle(bundle) = resource {
                Ok(bundle)
            } else {
                return Err(FHIRRequestParsingError::InvalidBody);
            }
        }
        HTTPBody::Bytes(body) => serde_json::from_slice::<Bundle>(&body),
    }?;
    Ok(bundle)
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
*/
fn parse_request_1_non_empty(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    match req.method {
        Method::POST => {
            match url_chunks[0].as_str() {
                "_search" => Ok(FHIRRequest::Search(SearchRequest::System(
                    FHIRSearchSystemRequest {
                        parameters: get_search_parameters_body(req)?,
                    },
                ))),
                _ => {
                    let resource_type = ResourceType::try_from(url_chunks[0].as_str())?;
                    let resource = get_resource(&resource_type, req)?;
                    // Handle create request
                    Ok(FHIRRequest::Create(FHIRCreateRequest {
                        resource_type,
                        resource,
                    }))
                }
            }
        }
        Method::PUT => {
            let resource_type = ResourceType::try_from(url_chunks[0].as_str())?;
            let parameters = parse_query(req.query)?;
            let resource = get_resource(&resource_type, req)?;
            Ok(FHIRRequest::Update(UpdateRequest::Conditional(
                FHIRConditionalUpdateRequest {
                    parameters,
                    resource_type,
                    resource,
                },
            )))
        }
        Method::DELETE => Ok(FHIRRequest::Delete(DeleteRequest::Type(
            FHIRDeleteTypeRequest {
                parameters: parse_query(req.query)?,
                resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
            },
        ))),
        Method::GET => {
            match url_chunks[0].as_str() {
                "metadata" => {
                    // Handle capabilities request
                    Ok(FHIRRequest::Capabilities)
                }
                "_history" => Ok(FHIRRequest::History(HistoryRequest::System(
                    FHIRHistorySystemRequest {
                        parameters: parse_query(req.query)?,
                    },
                ))),
                _ => {
                    // Handle search request
                    Ok(FHIRRequest::Search(SearchRequest::Type(
                        FHIRSearchTypeRequest {
                            resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                            parameters: parse_query(req.query)?,
                        },
                    )))
                }
            }
        }
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request".to_string(),
        )),
    }
}

/*
transaction	        /	                                  POST	R	Bundle	O	N/A
batch	              /	                                  POST	R	Bundle	O	N/A
search-system	      ?	                                  GET	N/A	N/A	N/A	N/A
delete-conditional  ?                                   DELETE N/A N/A N/A O: If-Match
*/
fn parse_request_1_empty(
    _fhir_version: SupportedFHIRVersions,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    match req.method {
        Method::POST => {
            let bundle = get_bundle(req)?;

            match &bundle.type_ {
                b if b == &BundleType::transaction() => {
                    // Handle transaction request
                    Ok(FHIRRequest::Transaction(FHIRTransactionRequest {
                        resource: bundle,
                    }))
                }
                b if b == &BundleType::batch() => {
                    // Handle batch request
                    Ok(FHIRRequest::Batch(FHIRBatchRequest { resource: bundle }))
                }
                _ => Err(FHIRRequestParsingError::Unsupported(
                    "Unsupported bundle type".to_string(),
                )),
            }
        }
        Method::GET => {
            // Handle search system request
            Ok(FHIRRequest::Search(SearchRequest::System(
                FHIRSearchSystemRequest {
                    parameters: parse_query(req.query)?,
                },
            )))
        }
        Method::DELETE => Ok(FHIRRequest::Delete(DeleteRequest::System(
            FHIRDeleteSystemRequest {
                parameters: parse_query(req.query)?,
            },
        ))),
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request".to_string(),
        )),
    }
}

fn parse_request_1(
    fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    if url_chunks.is_empty() {
        parse_request_1_empty(fhir_version, req)
    } else {
        parse_request_1_non_empty(fhir_version, url_chunks, req)
    }
}

/*
search-type         /[type]/_search?	                POST	application/x-www-form-urlencoded	form data	N/A	N/A
read            	/[type]/[id]	                    GET‡	N/A	N/A	N/A	O: If-Modified-Since, If-None-Match
update             	/[type]/[id]                      	PUT	R	Resource	O	O: If-Match
patch        	    /[type]/[id]                      	PATCH	R (may be a patch type)	Patch	O	O: If-Match
delete	            /[type]/[id]	                    DELETE	N/A	N/A	N/A	N/A
history-type	    /[type]/_history	                GET	N/A	N/A	N/A	N/A
*/
fn parse_request_2(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    match req.method {
        Method::POST => {
            match url_chunks[1].as_str() {
                "_search" => {
                    // Handle search request
                    let resource_type = ResourceType::try_from(url_chunks[0].as_str())?;
                    Ok(FHIRRequest::Search(SearchRequest::Type(
                        FHIRSearchTypeRequest {
                            resource_type,
                            parameters: get_search_parameters_body(req)?,
                        },
                    )))
                }
                _ => Err(FHIRRequestParsingError::Unsupported(
                    "To create new resources run post at resource root.".to_string(),
                )),
            }
        }
        Method::GET => {
            if url_chunks[1] == "_history" {
                Ok(FHIRRequest::History(HistoryRequest::Type(
                    FHIRHistoryTypeRequest {
                        resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                        parameters: parse_query(req.query)?,
                    },
                )))
            } else {
                // Handle read request
                Ok(FHIRRequest::Read(FHIRReadRequest {
                    resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                    id: url_chunks[1].to_string(),
                }))
            }
        }
        Method::PUT => {
            let resource_type = ResourceType::try_from(url_chunks[0].as_str())?;
            let resource = get_resource(&resource_type, req)?;
            Ok(FHIRRequest::Update(UpdateRequest::Instance(
                FHIRUpdateInstanceRequest {
                    resource_type,
                    id: url_chunks[1].to_string(),
                    resource,
                },
            )))
        }
        Method::PATCH => Ok(FHIRRequest::Patch(FHIRPatchRequest {
            resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
            id: url_chunks[1].to_string(),
            patch: match req.body {
                HTTPBody::Bytes(body) => serde_json::from_slice::<Patch>(&body)?,
                _ => Err(FHIRRequestParsingError::Unsupported(
                    "PATCH requests must have a JSON body".to_string(),
                ))?,
            },
        })),
        Method::DELETE => Ok(FHIRRequest::Delete(DeleteRequest::Instance(
            FHIRDeleteInstanceRequest {
                resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                id: url_chunks[1].to_string(),
            },
        ))),
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request.".to_string(),
        )),
    }
}

/*
history-instance	  /[type]/[id]/_history	            GET	N/A	N/A	N/A	N/A
compartment-request   /[type]/[id]/[compartment-type]   GET N/A N/A N/A N/A
*/
fn parse_request_3(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    match req.method {
        Method::GET => {
            if url_chunks[2] == "_history" {
                Ok(FHIRRequest::History(HistoryRequest::Instance(
                    FHIRHistoryInstanceRequest {
                        resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                        id: url_chunks[1].to_string(),
                        parameters: parse_query(req.query)?,
                    },
                )))
            }
            // Process Compartment request
            else {
                Ok(FHIRRequest::Compartment(CompartmentRequest {
                    resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                    id: url_chunks[1].to_string(),
                    request: Box::new(FHIRRequest::Search(SearchRequest::Type(
                        FHIRSearchTypeRequest {
                            resource_type: ResourceType::try_from(url_chunks[2].as_str())?,
                            parameters: parse_query(req.query)?,
                        },
                    ))),
                }))
            }
        }
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request.".to_string(),
        )),
    }
}

/*
vread            	  /[type]/[id]/_history/[vid]	        GET‡	N/A	N/A	N/A	N/A
compartment-request   /[type]/[id]/[compartment-type]/[additional]  GET N/A N/A N/A N/A
*/
fn parse_request_4(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    if req.method == Method::GET && url_chunks[2] == "_history" {
        Ok(FHIRRequest::VersionRead(FHIRVersionReadRequest {
            resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
            id: url_chunks[1].to_string(),
            version_id: VersionId::new(url_chunks[3].to_string()),
        }))
    } else if req.method == Method::GET {
        Ok(FHIRRequest::Compartment(CompartmentRequest {
            resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
            id: url_chunks[1].to_string(),
            request: Box::new(FHIRRequest::Read(FHIRReadRequest {
                resource_type: ResourceType::try_from(url_chunks[2].as_str())?,
                id: url_chunks[3].to_string(),
            })),
        }))
    } else {
        Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request.".to_string(),
        ))
    }
}

/// The FHIR request `req` makes in `ctx`. An operation invoked with GET is
/// typed against its definition.
pub async fn http_request_to_fhir_request<B: Backend>(
    fhir_version: SupportedFHIRVersions,
    req: HTTPRequest<'_>,
    ctx: &Arc<ServerCTX<B>>,
) -> Result<FHIRRequest, OperationOutcomeError> {
    let url_pieces = req
        .path
        .split_terminator('/')
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    if let Some(target) = invocation::target(&url_pieces)? {
        return invocation::invocation(target, req, ctx).await;
    }

    match url_pieces.len() {
        0 | 1 => parse_request_1(fhir_version, url_pieces, req),
        2 => parse_request_2(fhir_version, url_pieces, req),
        3 => parse_request_3(fhir_version, url_pieces, req),
        4 => parse_request_4(fhir_version, url_pieces, req),
        _ => Err(FHIRRequestParsingError::InvalidPath),
    }
    .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_client::url::ParsedParameter;

    #[test]
    fn parses_repeated_revinclude_and_include_parameters() {
        let query = "_id=256897&_revinclude=Condition:encounter&_revinclude=Observation:encounter&_include=Encounter:patient";
        let req = HTTPRequest::new(
            Method::GET,
            "Encounter".to_string(),
            HTTPBody::Bytes(Bytes::new()),
            query,
        );
        let fhir_req = parse_request_1(
            SupportedFHIRVersions::R4,
            vec!["Encounter".to_string()],
            req,
        )
        .unwrap();
        match fhir_req {
            FHIRRequest::Search(SearchRequest::Type(search)) => {
                let revincludes: Vec<_> = search
                    .parameters
                    .parameters()
                    .iter()
                    .filter_map(|p| match p {
                        ParsedParameter::Result(p) if p.name == "_revinclude" => {
                            Some(p.value.clone())
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(revincludes.len(), 2);
                assert_eq!(revincludes[0], vec!["Condition:encounter"]);
                assert_eq!(revincludes[1], vec!["Observation:encounter"]);

                let includes: Vec<_> = search
                    .parameters
                    .parameters()
                    .iter()
                    .filter_map(|p| match p {
                        ParsedParameter::Result(p) if p.name == "_include" => Some(p.value.clone()),
                        _ => None,
                    })
                    .collect();
                assert_eq!(includes.len(), 1);
                assert_eq!(includes[0], vec!["Encounter:patient"]);
            }
            _ => panic!("Expected Search::Type request"),
        }
    }
}
