use std::{fmt::Debug, pin::Pin, sync::Arc};

#[derive(Debug)]
pub struct Context<CTX: Debug, Request: Debug, Response: Debug> {
    pub ctx: CTX,
    pub request: Request,
    pub response: Option<Response>,
}

pub type MiddlewareOutput<Context, Error> =
    Pin<Box<dyn Future<Output = Result<Context, Error>> + Send>>;
pub type Next<State, Context, Error> =
    Box<dyn Fn(State, Context) -> MiddlewareOutput<Context, Error> + Send + Sync>;

pub type MiddlewareChainOld<State, CTX, Request, Response, Error> = Box<
    dyn Fn(
            State,
            Context<CTX, Request, Response>,
            Option<Arc<Next<State, Context<CTX, Request, Response>, Error>>>,
        ) -> MiddlewareOutput<Context<CTX, Request, Response>, Error>
        + Send
        + Sync,
>;

pub type MiddlewareNext<'a, State, CTX, Request, Response, Error> =
    Arc<Next<State, Context<CTX, Request, Response>, Error>>;

pub trait MiddlewareChain<State, CTX: Debug, Request: Debug, Response: Debug, Error>:
    Send + Sync
{
    fn call(
        &self,
        state: State,
        ctx: Context<CTX, Request, Response>,
        next: Option<MiddlewareNext<State, CTX, Request, Response, Error>>,
    ) -> MiddlewareOutput<Context<CTX, Request, Response>, Error>;
}

pub struct Middleware<State, CTX: Debug, Request: Debug, Response: Debug, Error> {
    _state: std::marker::PhantomData<State>,
    _phantom: std::marker::PhantomData<CTX>,
    execute: Arc<Next<State, Context<CTX, Request, Response>, Error>>,
}

impl<
    State: 'static + Send + Sync,
    CTX: 'static + Send + Sync + Debug,
    Request: 'static + Send + Sync + Debug,
    Response: 'static + Send + Sync + Debug,
    Error: 'static + Send + Sync,
> Middleware<State, CTX, Request, Response, Error>
{
    /// Create a new [`Middleware`] execution chain.
    ///
    /// # Panics
    ///
    /// This function will panic if the provided `middleware` vector is empty,
    /// as an empty chain cannot be unwrapped into an execution target.
    #[must_use]
    pub fn new(
        mut middleware: Vec<Box<dyn MiddlewareChain<State, CTX, Request, Response, Error>>>,
    ) -> Self {
        middleware.reverse();
        let next: Option<MiddlewareNext<State, CTX, Request, Response, Error>> = middleware
            .into_iter()
            .fold(
            None,
            |prev_next: Option<MiddlewareNext<State, CTX, Request, Response, Error>>,
             middleware: Box<dyn MiddlewareChain<State, CTX, Request, Response, Error>>| {
                Some(Arc::new(Box::new(move |state, ctx| {
                    middleware.call(state, ctx, prev_next.clone())
                })))
            },
        );

        Middleware {
            _state: std::marker::PhantomData,
            _phantom: std::marker::PhantomData,
            execute: next.unwrap(),
        }
    }

    /// Executes the middleware chain for the given request.
    ///
    /// # Errors
    ///
    /// Returns an error if any `MiddlewareChain` in the execution pipeline fails
    /// while processing the state or request context.
    pub async fn call(
        &self,
        state: State,
        ctx: CTX,
        request: Request,
    ) -> Result<Context<CTX, Request, Response>, Error> {
        (self.execute)(
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

    struct MiddlewareChain1 {}
    impl MiddlewareChain<(), (), usize, usize, String> for MiddlewareChain1 {
        fn call(
            &self,
            _state: (),
            x: Context<(), usize, usize>,
            _next: Option<Arc<Next<(), Context<(), usize, usize>, String>>>,
        ) -> Pin<Box<dyn Future<Output = Result<Context<(), usize, usize>, String>> + Send>>
        {
            Box::pin(async move {
                let mut x = if let Some(next) = _next {
                    let p = next((), x).await;
                    p
                } else {
                    Ok(x)
                }?;
                println!("Middleware 1 executed");
                x.response = x.response.map(|r| r + 1);
                Ok(x)
            })
        }
    }

    struct MiddlewareChain2 {}
    impl MiddlewareChain<(), (), usize, usize, String> for MiddlewareChain2 {
        fn call(
            &self,
            _state: (),
            x: Context<(), usize, usize>,
            _next: Option<Arc<Next<(), Context<(), usize, usize>, String>>>,
        ) -> Pin<Box<dyn Future<Output = Result<Context<(), usize, usize>, String>> + Send>>
        {
            Box::pin(async move {
                let mut x = if let Some(next) = _next {
                    let p = next((), x).await;
                    p
                } else {
                    Ok(x)
                }?;

                println!("Middleware 2 executed {:?}", x.response);
                x.response = x.response.map(|r| r + 2);
                Ok(x)
            })
        }
    }

    struct MiddlewareChain3 {}
    impl MiddlewareChain<(), (), usize, usize, String> for MiddlewareChain3 {
        fn call(
            &self,
            _state: (),
            x: Context<(), usize, usize>,
            _next: Option<Arc<Next<(), Context<(), usize, usize>, String>>>,
        ) -> Pin<Box<dyn Future<Output = Result<Context<(), usize, usize>, String>> + Send>>
        {
            Box::pin(async move {
                let mut x = if let Some(next) = _next {
                    let p = next((), x).await;
                    p
                } else {
                    Ok(x)
                }?;

                x.response = x.response.map_or(Some(x.request + 3), |r| Some(r + 3));
                Ok(x)
            })
        }
    }

    #[tokio::test]
    async fn test_middleware() {
        let test = Middleware::new(vec![
            Box::new(MiddlewareChain1 {}),
            Box::new(MiddlewareChain2 {}),
            Box::new(MiddlewareChain3 {}),
        ]);

        let ret = test.call((), (), 42).await;
        assert_eq!(Some(48), ret.unwrap().response);
    }
}
