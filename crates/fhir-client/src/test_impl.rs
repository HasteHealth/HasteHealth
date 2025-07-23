use crate::{FHIRClient, request::FHIRRequest};
use std::pin::Pin;

struct Test<CTX, F>
where
    F: FnMut(CTX) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync,
{
    _phantom: std::marker::PhantomData<CTX>,
    middleware: Vec<F>,
}

impl<CTX, F> Test<CTX, F>
where
    F: Fn(CTX) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync,
{
    pub fn new(middleware: Vec<F>) -> Self {
        Test {
            _phantom: std::marker::PhantomData,
            middleware,
        }
    }
    pub async fn call(&self, x: CTX) -> CTX {
        let mut ctx = x;
        for middleware in &self.middleware {
            ctx = middleware(ctx).await;
        }
        ctx
    }
}

fn what(x: usize) -> Pin<Box<dyn Future<Output = usize> + Send>> {
    Box::pin(async move {
        println!("Hello {}", x);
        x + 1
    })
}

fn string_concat(x: String) -> Pin<Box<dyn Future<Output = String> + Send>> {
    Box::pin(async move {
        println!("Hello {}", x);
        format!("{} world", x)
    })
}

fn main() {
    let test = Test::new(vec![what, what, what]);
    Test::new(vec![what]);

    let test2 = Test::new(vec![string_concat, string_concat]);

    test.call(42);
    test2.call("Hello".into());
}

// impl<CTX, Error, M> FHIRClient<CTX, Error> for Test<M>
// where
//     M: Fn(usize) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
// {
//     type Middleware = M;

//     fn request(
//         &self,
//         ctx: CTX,
//         request: crate::request::FHIRRequest,
//     ) -> impl Future<Output = Result<crate::request::FHIRResponse, Error>> + Send {
//         async { todo!() }
//     }

//     fn middleware(&self) -> &[Self::Middleware] {
//         &self.foo.as_slice()
//     }

//     fn capabilities(
//         &self,
//         ctx: CTX,
//     ) -> impl Future<Output = fhir_model::r4::types::CapabilityStatement> + Send {
//         todo!()
//     }

//     fn search_system(
//         &self,
//         ctx: CTX,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn search_type(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn create(
//         &self,
//         ctx: CTX,
//         resource: fhir_model::r4::types::Resource,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn update(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//         resource: fhir_model::r4::types::Resource,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn conditional_update(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         parameters: Vec<crate::ParsedParameter>,
//         resource: fhir_model::r4::types::Resource,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn patch(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//         patches: json_patch::Patch,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn read(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//     ) -> impl Future<Output = Result<Option<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn vread(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//         version_id: String,
//     ) -> impl Future<Output = Result<Option<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn delete_instance(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//     ) -> impl Future<Output = Result<(), Error>> + Send {
//         todo!()
//     }

//     fn delete_type(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<(), Error>> + Send {
//         todo!()
//     }

//     fn delete_system(
//         &self,
//         ctx: CTX,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<(), Error>> + Send {
//         todo!()
//     }

//     fn history_system(
//         &self,
//         ctx: CTX,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn history_type(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn history_instance(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//         parameters: Vec<crate::ParsedParameter>,
//     ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, Error>> + Send {
//         todo!()
//     }

//     fn invoke_instance(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         id: String,
//         operation: String,
//         parameters: fhir_model::r4::types::Parameters,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn invoke_type(
//         &self,
//         ctx: CTX,
//         resource_type: fhir_model::r4::types::ResourceType,
//         operation: String,
//         parameters: fhir_model::r4::types::Parameters,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn invoke_system(
//         &self,
//         ctx: CTX,
//         operation: String,
//         parameters: fhir_model::r4::types::Parameters,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn transaction(
//         &self,
//         ctx: CTX,
//         bundle: fhir_model::r4::types::Resource,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     fn batch(
//         &self,
//         ctx: CTX,
//         bundle: fhir_model::r4::types::Resource,
//     ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, Error>> + Send {
//         todo!()
//     }

//     // Other methods...
// }
