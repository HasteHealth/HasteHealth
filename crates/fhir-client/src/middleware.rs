use std::{pin::Pin, sync::Arc};

pub struct Context<CTX, Request, Response> {
    pub ctx: CTX,
    pub request: Request,
    pub response: Option<Response>,
}

pub type Next<State, CTX, Request, Response, Error> = Box<
    dyn Fn(
            State,
            Context<CTX, Request, Response>,
        )
            -> Pin<Box<dyn Future<Output = Result<Context<CTX, Request, Response>, Error>> + Send>>
        + Send
        + Sync,
>;

pub type MiddlewareChain<State, CTX, Request, Response, Error> = Box<
    dyn Fn(
            State,
            Context<CTX, Request, Response>,
            Option<Arc<Next<State, CTX, Request, Response, Error>>>,
        )
            -> Pin<Box<dyn Future<Output = Result<Context<CTX, Request, Response>, Error>> + Send>>
        + Send
        + Sync,
>;

pub struct Middleware<State, CTX, Request, Response, Error> {
    _state: std::marker::PhantomData<State>,
    _phantom: std::marker::PhantomData<CTX>,
    _execute: Arc<Next<State, CTX, Request, Response, Error>>,
}

impl<
    State: 'static + Send + Sync,
    CTX: 'static + Send + Sync,
    Request: 'static + Send + Sync,
    Response: 'static + Send + Sync,
    Error: 'static + Send + Sync,
> Middleware<State, CTX, Request, Response, Error>
{
    pub fn new(mut middleware: Vec<MiddlewareChain<State, CTX, Request, Response, Error>>) -> Self {
        middleware.reverse();
        let next: Option<Arc<Next<State, CTX, Request, Response, Error>>> = middleware
            .into_iter()
            .fold(
            None,
            |prev_next: Option<Arc<Next<State, CTX, Request, Response, Error>>>,
             middleware: MiddlewareChain<State, CTX, Request, Response, Error>| {
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
    ) -> Result<Context<CTX, Request, Response>, Error> {
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
        _state: (),
        x: Context<(), usize, usize>,
        _next: Option<Arc<Next<(), (), usize, usize, String>>>,
    ) -> Pin<Box<dyn Future<Output = Result<Context<(), usize, usize>, String>> + Send>> {
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

    fn middleware_2(
        _state: (),
        x: Context<(), usize, usize>,
        _next: Option<Arc<Next<(), (), usize, usize, String>>>,
    ) -> Pin<Box<dyn Future<Output = Result<Context<(), usize, usize>, String>> + Send>> {
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

    fn middleware_3(
        _state: (),
        x: Context<(), usize, usize>,
        _next: Option<Arc<Next<(), (), usize, usize, String>>>,
    ) -> Pin<Box<dyn Future<Output = Result<Context<(), usize, usize>, String>> + Send>> {
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

    #[tokio::test]
    async fn test_middleware() {
        let test = Middleware::new(vec![
            Box::new(middleware_1),
            Box::new(middleware_2),
            Box::new(middleware_3),
        ]);

        let ret = test.call((), (), 42).await;
        assert_eq!(Some(48), ret.unwrap().response);
    }
}
