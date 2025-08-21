#![allow(unused)]
use std::any::type_name;

use lambda_http::{Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

trait Resource {
    type Value;
    fn resource_type(&self) -> &str;
    fn concrete(&self) -> &Self::Value;
}

#[derive(Serialize, Deserialize, Debug)]
struct Test {
    name: String,
    age: u64,
}

impl Resource for Test {
    type Value = Test;
    fn resource_type(&self) -> &str {
        "Test"
    }
    fn concrete(&self) -> &Self::Value {
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum ResourceEnum {
    Test(Test),
}

// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }

fn resource_checker(resource: &ResourceEnum) -> Result<(), String> {
    match resource {
        ResourceEnum::Test(test) => {
            println!("Resource is of type Test with name: {}", test.name);
            Ok(())
        }
        _ => Err("Invalid resource type".to_string()),
    }
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("world");

    let body = event.body();
    let body_string = "{\"name\": \"bob\", \"age\": 45}"; // String::from_utf8_lossy(body.as_ref());

    let test: ResourceEnum = serde_json::from_str(&body_string).unwrap();
    // .unwrap_or(ResourceEnum::Test(Box::new(Test {
    //     name: "default_user".to_string(),
    //     age: 0,
    // })));

    let z = resource_checker(&test);
    println!("Resource check result: {:?}", z);

    match test {
        ResourceEnum::Test(z) => {
            println!("Resource is of type Test with name: {}", z.name);
        }
    }

    let message = format!("Hello {who}, this is an AWS Lambda HTTP request.");

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use lambda_http::{Request, RequestExt};
//     use std::collections::HashMap;

//     #[tokio::test]
//     async fn test_generic_http_handler() {
//         let request = Request::default();

//         let response = function_handler(request).await.unwrap();
//         assert_eq!(response.status(), 200);

//         let body_bytes = response.body().to_vec();
//         let body_string = String::from_utf8(body_bytes).unwrap();

//         assert_eq!(
//             body_string,
//             "Hello world, this is an AWS Lambda HTTP request"
//         );
//     }

//     #[tokio::test]
//     async fn test_http_handler_with_query_string() {
//         let mut query_string_parameters: HashMap<String, String> = HashMap::new();
//         query_string_parameters.insert("name".into(), "lambda-test".into());

//         let request = Request::default().with_query_string_parameters(query_string_parameters);

//         let response = function_handler(request).await.unwrap();
//         assert_eq!(response.status(), 200);

//         let body_bytes = response.body().to_vec();
//         let body_string = String::from_utf8(body_bytes).unwrap();

//         assert_eq!(
//             body_string,
//             "Hello lambda-test, this is an AWS Lambda HTTP request"
//         );
//     }
// }
