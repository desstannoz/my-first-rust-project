use axum::{
    extract::{Json, State},
    http::StatusCode,
    Extension
};
use sea_orm::EntityTrait;
use serde_json;

use crate::{
    middleware::auth::UserContext,
    DB,
    models::user::Entity as Users
};

pub async fn me(
    Extension(user_ctx): Extension<UserContext>,
    State(db): State<DB>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = Users::find_by_id(user_ctx.username)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => Ok(Json(serde_json::json!({
            "username": user.username
        }))),
        None => Err(StatusCode::NOT_FOUND)
    }
}
