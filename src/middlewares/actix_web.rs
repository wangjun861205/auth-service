use crate::core::token_manager::TokenManager;
use std::{
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
    rc::Rc,
    str::FromStr,
    task::Poll,
};

use actix_web::{
    dev::{Service, ServiceRequest, Transform},
    error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
    http::header::{HeaderName, HeaderValue},
    Error,
};

type AuthTokenFuture<S: Service<ServiceRequest>> =
    Pin<Box<dyn Future<Output = Result<S::Response, Error>>>>;

pub struct AuthTokenService<S, T>
where
    S: Service<ServiceRequest>,
    T: TokenManager,
{
    auth_header_name: &'static str,
    service: Rc<S>,
    token_manager: T,
}

impl<S, T> Service<ServiceRequest> for AuthTokenService<S, T>
where
    S: Service<ServiceRequest> + 'static,
    T: TokenManager + Clone + 'static,
    S::Error: Display + Debug,
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        if let Some(header_value) = req.headers().get(self.auth_header_name) {
            let header_value = header_value.clone();
            if let Ok(token) = header_value.to_str() {
                let token = token.to_string();
                let token_manager = self.token_manager.clone();
                let service = self.service.clone();
                return Box::pin(async move {
                    let uid = token_manager
                        .verify_token(token)
                        .await
                        .map_err(ErrorForbidden)?;
                    req.headers_mut().insert(
                        HeaderName::from_str("User-ID").unwrap(),
                        HeaderValue::from_str(&uid).unwrap(),
                    );
                    service.call(req).await.map_err(ErrorInternalServerError)
                });
            }
            return Box::pin(async move { Err(ErrorUnauthorized("invalid header value")) });
        }
        Box::pin(async move { Err(ErrorUnauthorized("header not exists")) })
    }
}

pub struct AuthTokenMiddleware<T>
where
    T: TokenManager,
{
    auth_header_name: &'static str,
    token_manager: T,
}

impl<T> AuthTokenMiddleware<T>
where
    T: TokenManager,
{
    pub fn new(auth_header_name: &'static str, token_manager: T) -> Self {
        Self {
            auth_header_name,
            token_manager,
        }
    }
}

impl<S, T> Transform<S, ServiceRequest> for AuthTokenMiddleware<T>
where
    T: TokenManager + Clone + 'static,
    S: Service<ServiceRequest> + 'static,
    S::Error: Display + Debug,
{
    type Transform = AuthTokenService<S, T>;
    type Response = S::Response;
    type InitError = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;
    fn new_transform(&self, service: S) -> Self::Future {
        let token_manager = self.token_manager.clone();
        let auth_header_name = self.auth_header_name;
        Box::pin(async move {
            Ok(AuthTokenService {
                auth_header_name,
                service: Rc::new(service),
                token_manager,
            })
        })
    }
}
