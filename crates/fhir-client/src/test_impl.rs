use std::{pin::Pin, sync::Arc};

type Next<CTX: 'static + Send + Sync> =
    Box<dyn Fn(CTX) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync>;

type Middleware<CTX: 'static + Send + Sync> =
    Box<dyn Fn(CTX, Option<Next<CTX>>) -> Pin<Box<dyn Future<Output = CTX> + Send>> + Send + Sync>;

struct Test<CTX: 'static + Send + Sync> {
    _phantom: std::marker::PhantomData<CTX>,
    middleware: Arc<Vec<Middleware<CTX>>>,
}

impl<CTX: Send + Sync> Test<CTX> {
    pub fn new(middleware: Vec<Middleware<CTX>>) -> Self {
        Test {
            _phantom: std::marker::PhantomData,
            middleware: Arc::new(middleware),
        }
    }

    pub async fn call(&self, x: CTX) -> CTX {
        let mut ctx = x;

        for i in 0..self.middleware.len() {
            let middleware = self.middleware.clone();
            let nxt: Next<CTX> = Box::new(move |v| {
                // next_mid(v, None);
                if middleware.get(1).is_some() {
                    println!("Middleware at index {} is present", i + 1);
                } else {
                    println!("No middleware at index {}", i + 1);
                }

                let p = middleware[i + 1](v, None);

                // &self.middleware[i + 1](v, None);
                Box::pin(async { p.await })
            });

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

fn what_2(
    x: TestCTX,
    _next: Option<Next<TestCTX>>,
) -> Pin<Box<dyn Future<Output = TestCTX> + Send>> {
    Box::pin(async move {
        println!("Hello World");
        x
    })
}

struct TestCTX {
    K: String,
    P: u32,
}

fn main() {
    let test = Test::new(vec![Box::new(what), Box::new(what), Box::new(what)]);
    Test::new(vec![Box::new(what)]);

    let test3 = Test::new(vec![Box::new(what_2)]);
    test3.call(TestCTX {
        K: "Hello".into(),
        P: 42,
    });

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
