use std::{pin::Pin, sync::Arc};

type Next<CTX: Send + Sync> =
    Box<dyn Fn(&CTX) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync>;

type Middleware<CTX: Send + Sync> =
    Box<dyn Fn(CTX, Option<Next<CTX>>) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync>;

struct Test<CTX: Send + Sync> {
    _phantom: std::marker::PhantomData<CTX>,
    middleware: Vec<Middleware<CTX>>,
}

fn create_next<CTX>(next_mid: Middleware<CTX>) -> Next<CTX> {
    let nxt: Next<CTX> = Box::new(|v| next_mid(v, None));
    nxt
}

impl<CTX: Send + Sync> Test<CTX> {
    pub fn new(middleware: Vec<Middleware<CTX>>) -> Self {
        Test {
            _phantom: std::marker::PhantomData,
            middleware: middleware,
        }
    }
    pub async fn call(&self, x: CTX) -> CTX {
        let mut ctx = x;

        for i in 0..self.middleware.len() {
            let next_mid = &self.middleware[i + 1];
            let nxt: Next<CTX> = Box::new(|v| next_mid(v, None));
            ctx = self.middleware[i](ctx, Some(nxt)).await;
        }

        ctx
    }
}

fn what(x: usize, _next: Option<Next<usize>>) -> Pin<Box<dyn Future<Output = usize> + Send>> {
    Box::pin(async move {
        println!("Hello {}", x);
        x + 1
    })
}

fn string_concat(
    x: String,
    _next: Option<Next<String>>,
) -> Pin<Box<dyn Future<Output = String> + Send>> {
    Box::pin(async move {
        println!("Hello {}", x);
        format!("{} world", x)
    })
}

fn main() {
    let test = Test::new(vec![Box::new(what), Box::new(what), Box::new(what)]);
    Test::new(vec![Box::new(what)]);

    let test2 = Test::new(vec![Box::new(string_concat), Box::new(string_concat)]);

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
