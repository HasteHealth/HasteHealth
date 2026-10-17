//! The server's request pipeline.
//!
//! A request runs through a [`Pipeline`]: its [`Middleware`] layers in order,
//! then the [`Handler`] that ends it. A layer gets the request and a [`Next`]
//! that runs the rest of the pipeline, so it can check or change the request
//! before and the response after. The handler at the end always produces a
//! response, so a layer that runs `next` always gets one back.
//!
//! `Next` borrows the layers that follow and the handler, so running it costs
//! no allocation or reference counting. Everything is generic over one
//! [`Backend`], so a layer names no repository, search or terminology type.

use crate::fhir_client::{ClientState, ServerCTX};
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::{pin::Pin, sync::Arc};

pub mod auditing;
pub mod auth_z;
pub mod capabilities;
pub mod check_project;
pub mod custom_models;
pub mod element_filtering;
pub mod operation_definitions;
pub mod operations;
pub mod patch_as_update;
pub mod rate_limit;
pub mod search_includes;
pub mod set_artifact_tenant;
pub mod storage;
pub mod tenant_tier_limits;
pub mod transaction;
pub mod validation;

/// The server client and the stack it runs on: its repository, search engine
/// and terminology. The client implements it, so a layer is generic over the
/// one type that names them all, and `ctx.client` sends requests back through
/// the whole pipeline.
pub trait Backend:
    FHIRClient<Arc<ServerCTX<Self>>, OperationOutcomeError> + Sized + 'static
{
    type Repo: Repository + Send + Sync + 'static;
    type Search: SearchEngine + Send + Sync + 'static;
    type Terminology: FHIRTerminology + Send + Sync + 'static;
}

/// The future a layer or handler returns: the response, once the rest of the
/// pipeline has run. It borrows the pipeline and the state for `'a`.
pub type Output<'a> =
    Pin<Box<dyn Future<Output = Result<FHIRResponse, OperationOutcomeError>> + Send + 'a>>;

/// A layer of the pipeline. It runs `next` for the response, or answers the
/// request itself.
pub trait Middleware<B: Backend>: Send + Sync {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a>;
}

/// The end of a pipeline: what answers the request once every layer has run.
pub trait Handler<B: Backend>: Send + Sync {
    fn handle<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a>;
}

/// The rest of the pipeline from a layer's point of view: the layers after it
/// and the handler. Copy, so a layer can run it more than once.
pub struct Next<'a, B: Backend> {
    rest: &'a [Box<dyn Middleware<B>>],
    end: &'a dyn Handler<B>,
}

impl<B: Backend> Clone for Next<'_, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B: Backend> Copy for Next<'_, B> {}

impl<'a, B: Backend> Next<'a, B> {
    /// Runs the rest of the pipeline on `request`.
    pub fn run(
        self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        match self.rest.split_first() {
            Some((layer, rest)) => layer.call(
                state,
                ctx,
                request,
                Next {
                    rest,
                    end: self.end,
                },
            ),
            None => self.end.handle(state, ctx, request),
        }
    }
}

/// Layers in order, then the handler that ends the pipeline.
pub struct Pipeline<B: Backend> {
    layers: Box<[Box<dyn Middleware<B>>]>,
    end: Box<dyn Handler<B>>,
}

impl<B: Backend> Pipeline<B> {
    pub fn new(layers: Vec<Box<dyn Middleware<B>>>, end: impl Handler<B> + 'static) -> Self {
        Pipeline {
            layers: layers.into_boxed_slice(),
            end: Box::new(end),
        }
    }

    /// Runs `request` through every layer and the handler.
    pub fn run<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        Next {
            rest: &self.layers,
            end: self.end.as_ref(),
        }
        .run(state, ctx, request)
    }
}

/// A pipeline ends another: the router's lanes are pipelines of their own.
impl<B: Backend> Handler<B> for Pipeline<B> {
    fn handle<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        self.run(state, ctx, request)
    }
}
