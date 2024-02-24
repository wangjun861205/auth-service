use crate::core::{
    hasher::Hasher, repository::Repository, service::Service as AuthService,
    token_manager::TokenManager,
};
use std::{
    fmt::{Debug, Display},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::Poll,
};

use actix_web::{
    dev::{Service, ServiceRequest, Transform},
    error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
    Error, FromRequest, HttpMessage,
};
use serde::{Deserialize, Serialize};

pub trait Claim {
    async fn validate(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct ClaimWrap<C>(pub C)
where
    C: Clone + 'static;

impl<C> FromRequest for ClaimWrap<C>
where
    C: Clone,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.extensions().get::<ClaimWrap<C>>() {
            Some(c) => {
                let c = c.clone();
                Box::pin(async move { Ok(c) })
            }
            None => Box::pin(async move { Err(ErrorForbidden("claim not exists")) }),
        }
    }
}

type AuthTokenFuture<S: Service<ServiceRequest>> =
    Pin<Box<dyn Future<Output = Result<S::Response, Error>>>>;

pub struct AuthTokenService<S, R, H, T, C>
where
    S: Service<ServiceRequest>,
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
    for<'de> C: Deserialize<'de> + Serialize + Claim,
{
    auth_header_name: &'static str,
    service: Rc<S>,
    auth_service: AuthService<R, H, T, C>,
    _phantom: PhantomData<C>,
}

impl<S, R, H, T, C> Service<ServiceRequest> for AuthTokenService<S, R, H, T, C>
where
    S: Service<ServiceRequest> + 'static,
    S::Error: Display + Debug,
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
    for<'de> C: Deserialize<'de> + Serialize + Claim + Clone + 'static,
{
    type Response = S::Response;
    type Error = Error;
    type Future = AuthTokenFuture<S>;

    fn poll_ready(
        &self,
        _ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(header_value) = req.headers().get(self.auth_header_name) {
            let header_value = header_value.clone();
            if let Ok(token) = header_value.to_str() {
                let token = token.to_string();
                let auth_service = self.auth_service.clone();
                let service = self.service.clone();
                return Box::pin(async move {
                    let claim = auth_service
                        .verify_token(&token)
                        .await
                        .map_err(ErrorForbidden)?;
                    claim.validate().await.map_err(ErrorForbidden)?;
                    req.extensions_mut().insert(ClaimWrap(claim));
                    service.call(req).await.map_err(ErrorInternalServerError)
                });
            }
            return Box::pin(async move { Err(ErrorUnauthorized("invalid header value")) });
        }
        Box::pin(async move { Err(ErrorUnauthorized("header not exists")) })
    }
}

pub struct AuthTokenMiddleware<R, H, T, C>
where
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
    for<'de> C: Serialize + Deserialize<'de>,
{
    auth_header_name: &'static str,
    auth_service: AuthService<R, H, T, C>,
}

impl<R, H, T, C> AuthTokenMiddleware<R, H, T, C>
where
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
    for<'de> C: Serialize + Deserialize<'de>,
{
    pub fn new(auth_header_name: &'static str, auth_service: AuthService<R, H, T, C>) -> Self {
        Self {
            auth_header_name,
            auth_service,
        }
    }
}

impl<S, R, H, T, C> Transform<S, ServiceRequest> for AuthTokenMiddleware<R, H, T, C>
where
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
    S: Service<ServiceRequest> + 'static,
    S::Error: Display + Debug,
    for<'de> C: Serialize + Deserialize<'de> + Claim + Clone + 'static,
{
    type Transform = AuthTokenService<S, R, H, T, C>;
    type Response = S::Response;
    type InitError = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;
    fn new_transform(&self, service: S) -> Self::Future {
        let auth_service = self.auth_service.clone();
        let auth_header_name = self.auth_header_name;
        Box::pin(async move {
            Ok(AuthTokenService::<_, _, _, _, C> {
                auth_header_name,
                service: Rc::new(service),
                auth_service,
                _phantom: PhantomData,
            })
        })
    }
}
