use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use sea_orm::DatabaseConnection;

use crate::{
    controllers::{user, auth},
    middleware::auth::auth_middleware
};
type DB = DatabaseConnection;

pub fn routes(db: DB) -> Router {
    let protected = Router::new()
        .route("/me", get(user::me))
        .route_layer(middleware::from_fn(auth_middleware));

    let auth = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));

    Router::new()
        .nest("/api", protected)
        .nest("/api/auth", auth)
        .with_state(db)
}
