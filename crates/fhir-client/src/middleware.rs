use std::{pin::Pin, sync::Arc};

use crate::request::{FHIRRequest, FHIRResponse};

pub struct Context<CTX, Request, Response> {
    pub ctx: CTX,
    pub request: Request,
    pub response: Option<Response>,
}

pub type Next<State, CTX, Request, Response> = Box<
    dyn Fn(
            State,
            Context<CTX, Request, Response>,
        ) -> Pin<Box<dyn Future<Output = Context<CTX, Request, Response>> + Send>>
        + Send
        + Sync,
>;

pub type MiddlewareChain<State, CTX, Request, Response> = Box<
    dyn Fn(
            State,
            Context<CTX, Request, Response>,
            Option<Arc<Next<State, CTX, Request, Response>>>,
        ) -> Pin<Box<dyn Future<Output = Context<CTX, Request, Response>> + Send>>
        + Send
        + Sync,
>;

pub struct Middleware<State, CTX, Request, Response> {
    _state: std::marker::PhantomData<State>,
    _phantom: std::marker::PhantomData<CTX>,
    _execute: Arc<Next<State, CTX, Request, Response>>,
}

impl<
    State: 'static + Send + Sync,
    CTX: 'static + Send + Sync,
    Request: 'static + Send + Sync,
    Response: 'static + Send + Sync,
> Middleware<State, CTX, Request, Response>
{
    pub fn new(mut middleware: Vec<MiddlewareChain<State, CTX, Request, Response>>) -> Self {
        middleware.reverse();
        let next: Option<Arc<Next<State, CTX, Request, Response>>> = middleware.into_iter().fold(
            None,
            |prev_next: Option<Arc<Next<State, CTX, Request, Response>>>,
             middleware: MiddlewareChain<State, CTX, Request, Response>| {
                Some(Arc::new(Box::new(move |state, ctx| {
                    middleware(state, ctx, prev_next.clone())
                })))
            },
        );

        Middleware {
            _state: std::marker::PhantomData,
            _phantom: std::marker::PhantomData,
            _execute: next.unwrap(),
        }
    }

    pub async fn call(
        &self,
        state: State,
        ctx: CTX,
        request: Request,
    ) -> Context<CTX, Request, Response> {
        (self._execute)(
            state,
            Context {
                ctx,
                request,
                response: None,
            },
        )
        .await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn middleware_1(
        state: (),
        x: Context<(), usize, usize>,
        _next: Option<Arc<Next<(), (), usize, usize>>>,
    ) -> Pin<Box<dyn Future<Output = Context<(), usize, usize>> + Send>> {
        Box::pin(async move {
            let mut x = if let Some(next) = _next {
                let p = next((), x).await;
                p
            } else {
                x
            };
            println!("Middleware 1 executed");
            x.response = x.response.map(|r| r + 1);
            x
        })
    }

    fn middleware_2(
        state: (),
        x: Context<(), usize, usize>,
        _next: Option<Arc<Next<(), (), usize, usize>>>,
    ) -> Pin<Box<dyn Future<Output = Context<(), usize, usize>> + Send>> {
        Box::pin(async move {
            let mut x = if let Some(next) = _next {
                let p = next((), x).await;
                p
            } else {
                x
            };

            println!("Middleware 2 executed {:?}", x.response);
            x.response = x.response.map(|r| r + 2);
            x
        })
    }

    fn middleware_3(
        state: (),
        x: Context<(), usize, usize>,
        _next: Option<Arc<Next<(), (), usize, usize>>>,
    ) -> Pin<Box<dyn Future<Output = Context<(), usize, usize>> + Send>> {
        Box::pin(async move {
            let mut x = if let Some(next) = _next {
                let p = next((), x).await;
                p
            } else {
                x
            };

            x.response = x.response.map_or(Some(x.request + 3), |r| Some(r + 3));
            x
        })
    }

    #[tokio::test]
    async fn test_middleware() {
        let test = Middleware::new(vec![
            Box::new(middleware_1),
            Box::new(middleware_2),
            Box::new(middleware_3),
        ]);

        let ret = test.call((), (), 42).await;
        assert_eq!(Some(48), ret.response);
    }
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
