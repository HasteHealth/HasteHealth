use axum::http::Method;
use json_patch::Patch;
use oxidized_fhir_client::request::{
    FHIRBatchRequest, FHIRConditionalUpdateRequest, FHIRCreateRequest, FHIRDeleteInstanceRequest,
    FHIRDeleteSystemRequest, FHIRDeleteTypeRequest, FHIRHistoryInstanceRequest,
    FHIRHistorySystemRequest, FHIRHistoryTypeRequest, FHIRInvokeInstanceRequest,
    FHIRInvokeSystemRequest, FHIRInvokeTypeRequest, FHIRPatchRequest, FHIRReadRequest, FHIRRequest,
    FHIRSearchSystemRequest, FHIRSearchTypeRequest, FHIRTransactionRequest,
    FHIRUpdateInstanceRequest, FHIRVersionReadRequest, Operation, OperationParseError,
};
use oxidized_fhir_client::url::{ParseError, parse_query};
use oxidized_fhir_model::r4::types::{
    Bundle, Parameters, Resource, ResourceType, ResourceTypeError,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use oxidized_fhir_serialization_json::errors::DeserializeError;
use oxidized_repository::types::SupportedFHIRVersions;

#[derive(Debug)]
pub enum HTTPBody {
    String(String),
    Resource(Resource),
}

#[derive(Debug)]
pub struct HTTPRequest {
    method: Method,
    path: String,
    body: HTTPBody,
    query: String,
}
impl HTTPRequest {
    pub fn new(method: Method, path: String, body: HTTPBody, query: String) -> Self {
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
    DeserializeError(#[from] DeserializeError),
    #[error(code = "invalid", diagnostic = "Failed to deserialize patch")]
    PatchDeserializeError(#[from] serde_json::Error),
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
        HTTPBody::String(body) => resource_type.deserialize(&body)?,
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
        HTTPBody::String(body) => oxidized_fhir_serialization_json::from_str::<Parameters>(&body),
    }?;
    Ok(params)
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
        HTTPBody::String(body) => oxidized_fhir_serialization_json::from_str::<Bundle>(&body),
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
(operation)	        /$[name]                            POST	R	Parameters	N/A	N/A
                                                        GET	N/A	N/A	N/A	N/A
                                                        POST	application/x-www-form-urlencoded	form data	N/A	N/A
*/
fn parse_request_1_non_empty(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    if url_chunks[0].starts_with("$") {
        match req.method {
            Method::POST => {
                // Handle operation request
                Ok(FHIRRequest::InvokeSystem(FHIRInvokeSystemRequest {
                    operation: Operation::new(&url_chunks[0])?,
                    parameters: get_parameters(req)?,
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
                match url_chunks[0].as_str() {
                    "_search" => Err(FHIRRequestParsingError::Unsupported(
                        "POST search requests are not supported".to_string(),
                    )
                    .into()),
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
                let parameters = parse_query(&req.query)?;
                let resource = get_resource(&resource_type, req)?;
                Ok(FHIRRequest::ConditionalUpdate(
                    FHIRConditionalUpdateRequest {
                        parameters,
                        resource_type,
                        resource,
                    },
                ))
            }
            Method::DELETE => Ok(FHIRRequest::DeleteType(FHIRDeleteTypeRequest {
                parameters: parse_query(&req.query)?,
                resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
            })),
            Method::GET => {
                match url_chunks[0].as_str() {
                    "metadata" => {
                        // Handle capabilities request
                        Ok(FHIRRequest::Capabilities)
                    }
                    "_history" => Ok(FHIRRequest::HistorySystem(FHIRHistorySystemRequest {
                        parameters: parse_query(&req.query)?,
                    })),
                    _ => {
                        // Handle search request
                        Ok(FHIRRequest::SearchType(FHIRSearchTypeRequest {
                            resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                            parameters: parse_query(&req.query)?,
                        }))
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
fn parse_request_1_empty(
    _fhir_version: SupportedFHIRVersions,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    match req.method {
        Method::POST => {
            let bundle = get_bundle(req)?;

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
                parameters: parse_query(&req.query)?,
            }))
        }
        Method::DELETE => Ok(FHIRRequest::DeleteSystem(FHIRDeleteSystemRequest {
            parameters: parse_query(&req.query)?,
        })),
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request".to_string(),
        )
        .into()),
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
fn parse_request_2(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    if url_chunks[1].starts_with("$") {
        match req.method {
            Method::POST => {
                // Handle operation request
                Ok(FHIRRequest::InvokeType(FHIRInvokeTypeRequest {
                    resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                    operation: Operation::new(&url_chunks[1])?,
                    parameters: get_parameters(req)?,
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
                match url_chunks[1].as_str() {
                    "_search" => {
                        // Handle search request
                        Err(FHIRRequestParsingError::Unsupported(
                            "POST search requests are not supported".to_string(),
                        )
                        .into())
                    }
                    _ => Err(FHIRRequestParsingError::Unsupported(
                        "To create new resources run post at resource root.".to_string(),
                    )
                    .into()),
                }
            }
            Method::GET => {
                if url_chunks[1] == "_history" {
                    Ok(FHIRRequest::HistoryType(FHIRHistoryTypeRequest {
                        resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                        parameters: parse_query(&req.query)?,
                    }))
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
                Ok(FHIRRequest::UpdateInstance(FHIRUpdateInstanceRequest {
                    resource_type,
                    id: url_chunks[1].to_string(),
                    resource,
                }))
            }
            Method::PATCH => Ok(FHIRRequest::Patch(FHIRPatchRequest {
                resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                id: url_chunks[1].to_string(),
                patch: match req.body {
                    HTTPBody::String(body) => serde_json::from_str::<Patch>(&body)?,
                    _ => Err(FHIRRequestParsingError::Unsupported(
                        "PATCH requests must have a JSON body".to_string(),
                    ))?,
                },
            })),
            Method::DELETE => Ok(FHIRRequest::DeleteInstance(FHIRDeleteInstanceRequest {
                resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                id: url_chunks[1].to_string(),
            })),
            _ => Err(FHIRRequestParsingError::Unsupported(
                "Unsupported method for FHIR request.".to_string(),
            )
            .into()),
        }
    }
}

/*
(operation)         /[type]/[id]/$[name]                POST	R	Parameters	N/A	N/A
                                                        GET	N/A	N/A	N/A	N/A
                                                        POST	application/x-www-form-urlencoded	form data	N/A	N/A
history-instance	  /[type]/[id]/_history	              GET	N/A	N/A	N/A	N/A
*/
fn parse_request_3(
    _fhir_version: SupportedFHIRVersions,
    url_chunks: Vec<String>,
    req: HTTPRequest,
) -> Result<FHIRRequest, FHIRRequestParsingError> {
    if url_chunks[2].starts_with("$") {
        match req.method {
            Method::POST => {
                // Handle operation request
                Ok(FHIRRequest::InvokeInstance(FHIRInvokeInstanceRequest {
                    resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                    id: url_chunks[1].to_string(),
                    operation: Operation::new(&url_chunks[2])?,
                    parameters: get_parameters(req)?,
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
            Method::GET => {
                if url_chunks[2] == "_history" {
                    Ok(FHIRRequest::HistoryInstance(FHIRHistoryInstanceRequest {
                        resource_type: ResourceType::try_from(url_chunks[0].as_str())?,
                        id: url_chunks[1].to_string(),
                        parameters: parse_query(&req.query)?,
                    }))
                } else {
                    // Handle read request
                    Err(FHIRRequestParsingError::Unsupported(
                        "Unsupported GET request.".to_string(),
                    )
                    .into())
                }
            }
            _ => Err(FHIRRequestParsingError::Unsupported(
                "Unsupported method for FHIR request.".to_string(),
            )
            .into()),
        }
    }
}

/*
vread            	  /[type]/[id]/_history/[vid]	        GET‡	N/A	N/A	N/A	N/A
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
            version_id: url_chunks[3].to_string(),
        }))
    } else {
        Err(FHIRRequestParsingError::Unsupported(
            "Unsupported method for FHIR request.".to_string(),
        )
        .into())
    }
}

pub fn http_request_to_fhir_request(
    fhir_version: SupportedFHIRVersions,
    req: HTTPRequest,
) -> Result<FHIRRequest, OperationOutcomeError> {
    let url_pieces = req
        .path
        .split_terminator('/')
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    match url_pieces.len() {
        0 | 1 => parse_request_1(fhir_version, url_pieces, req),
        2 => parse_request_2(fhir_version, url_pieces, req),
        3 => parse_request_3(fhir_version, url_pieces, req),
        4 => parse_request_4(fhir_version, url_pieces, req),
        _ => Err(FHIRRequestParsingError::InvalidPath.into()),
    }
    .map_err(|e| e.into())
}
