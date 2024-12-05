use axum::{
    extract::{Json, State},
    http::StatusCode,
    Extension,
};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, Set, ColumnTrait};
use serde_json;

use crate::{
    middleware::auth::UserContext, 
    models::user::{ActiveModel as UserActive, Column, Entity as Users}, 
    DB
};

pub async fn me(
    Extension(user_ctx): Extension<UserContext>,
    State(db): State<DB>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = Users::find()
        .filter(Column::Username.eq(user_ctx.username.to_string()))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => Ok(Json(serde_json::json!(user))),
        None => Err(StatusCode::NOT_FOUND),
    }
}
