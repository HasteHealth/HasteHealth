use std::{pin::Pin, sync::Arc};

type Next<CTX> = Box<dyn Fn(CTX) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync>;

type Middleware<CTX> = Box<
    dyn Fn(CTX, Arc<Option<Next<CTX>>>) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync,
>;

struct Test2<CTX: Send + Sync> {
    _phantom: std::marker::PhantomData<CTX>,
    _execute: Next<CTX>,
}

impl<CTX: 'static + Send + Sync> Test2<CTX> {
    pub fn new(mut middleware: Vec<Middleware<CTX>>) -> Self {
        middleware.reverse();
        let next: Arc<Option<Next<CTX>>> = middleware.into_iter().fold(
            Arc::new(None),
            |prev_next: Arc<Option<Next<CTX>>>, middleware: Middleware<CTX>| {
                Arc::new(Some(Box::new(move |ctx| {
                    middleware(ctx, prev_next.clone())
                })))
            },
        );

        let k = Arc::into_inner(next);

        Test2 {
            _phantom: std::marker::PhantomData,
            _execute: k.unwrap().unwrap(),
        }
    }

    pub async fn call(&self, ctx: CTX) -> CTX {
        (self._execute)(ctx).await
    }
}

fn middlware_1(
    x: usize,
    _next: Arc<Option<Next<usize>>>,
) -> Pin<Box<dyn Future<Output = usize> + Send>> {
    Box::pin(async move {
        println!("Middleware1 {}", x);

        let return_v = _next.as_ref().as_ref().unwrap()(x + 1).await;

        println!("Back in middleware1");

        return_v + 1
    })
}

fn middleware_2(
    x: usize,
    _next: Arc<Option<Next<usize>>>,
) -> Pin<Box<dyn Future<Output = usize> + Send>> {
    Box::pin(async move {
        println!("Middleware2 {}", x);
        x + 2
    })
}

fn middleware_3(
    x: usize,
    _next: Arc<Option<Next<usize>>>,
) -> Pin<Box<dyn Future<Output = usize> + Send>> {
    Box::pin(async move {
        println!("Middleware3 {}", x);
        x + 3
    })
}

fn string_concat(
    x: String,
    _next: Arc<Option<Next<String>>>,
) -> Pin<Box<dyn Future<Output = String> + Send>> {
    Box::pin(async move {
        println!("Hello {}", x);
        format!("{} world", x)
    })
}

async fn z_main() {
    let test = Test2::new(vec![
        Box::new(middlware_1),
        Box::new(middleware_2),
        Box::new(middleware_3),
    ]);

    let test2 = Test2::new(vec![Box::new(string_concat), Box::new(string_concat)]);

    let z = test.call(42).await;
    println!("{}", z);
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
