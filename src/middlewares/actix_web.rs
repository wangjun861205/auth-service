use crate::core::{
    hasher::Hasher, repository::Repository, service::Service as AuthService,
    token_manager::TokenManager,
};
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

pub struct AuthTokenService<S, R, H, T>
where
    S: Service<ServiceRequest>,
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
{
    auth_header_name: &'static str,
    uid_header_name: &'static str,
    service: Rc<S>,
    auth_service: AuthService<R, H, T>,
}

impl<S, R, H, T> Service<ServiceRequest> for AuthTokenService<S, R, H, T>
where
    S: Service<ServiceRequest> + 'static,
    S::Error: Display + Debug,
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
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
                let auth_service = self.auth_service.clone();
                let service = self.service.clone();
                let uid_header_name = self.uid_header_name;
                return Box::pin(async move {
                    let uid = auth_service
                        .verify_token(&token)
                        .await
                        .map_err(ErrorForbidden)?;
                    req.headers_mut().insert(
                        HeaderName::from_str(uid_header_name).unwrap(),
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

pub struct AuthTokenMiddleware<R, H, T>
where
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
{
    auth_header_name: &'static str,
    uid_header_name: &'static str,
    auth_service: AuthService<R, H, T>,
}

impl<R, H, T> AuthTokenMiddleware<R, H, T>
where
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
{
    pub fn new(
        auth_header_name: &'static str,
        uid_header_name: &'static str,
        auth_service: AuthService<R, H, T>,
    ) -> Self {
        Self {
            auth_header_name,
            uid_header_name,
            auth_service,
        }
    }
}

impl<S, R, H, T> Transform<S, ServiceRequest> for AuthTokenMiddleware<R, H, T>
where
    R: Repository + Clone + 'static,
    H: Hasher + Clone + 'static,
    T: TokenManager + Clone + 'static,
    S: Service<ServiceRequest> + 'static,
    S::Error: Display + Debug,
{
    type Transform = AuthTokenService<S, R, H, T>;
    type Response = S::Response;
    type InitError = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;
    fn new_transform(&self, service: S) -> Self::Future {
        let auth_service = self.auth_service.clone();
        let auth_header_name = self.auth_header_name;
        let uid_header_name = self.uid_header_name;
        Box::pin(async move {
            Ok(AuthTokenService {
                auth_header_name,
                uid_header_name,
                service: Rc::new(service),
                auth_service,
            })
        })
    }
}
