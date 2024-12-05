use axum::{
    http::StatusCode,
    middleware::Next,
    response::Response,
    extract::Request,
};
use axum_extra::{
    TypedHeader,
    headers::{authorization::Bearer, Authorization}
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

pub const KEY: &[u8] = b"secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct UserContext {
    pub username: String,
}
trait RequestExt {
    fn user(&self) -> Option<&UserContext>;
}

impl RequestExt for Request {
    fn user(&self) -> Option<&UserContext> {
        self.extensions().get::<UserContext>()
    }
}
pub async fn auth_middleware(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token_data = decode::<Claims>(
        &auth.token(),
        &DecodingKey::from_secret(KEY),
        &Validation::default(),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(UserContext {
        username: token_data.claims.sub,
    });

    Ok(next.run(request).await)
} 