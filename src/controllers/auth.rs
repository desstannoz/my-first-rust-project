use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter};
use serde_json;

use crate::{
    middleware::auth::{Claims, KEY},
    models::user::{ActiveModel as UserActive, Entity as Users, Column},
    DB,
};

pub async fn register(
    State(db): State<DB>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let username = payload["username"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let password = payload["password"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let hashed_password = hash(password, 10).map_err(|e| {
        eprintln!("Hashing error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = UserActive {
        id: sea_orm::ActiveValue::NotSet,
        username: sea_orm::ActiveValue::Set(username.to_string()),
        password: sea_orm::ActiveValue::Set(hashed_password),
    };

    match user.insert(&db).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("Database error: {}", e);
            if e.to_string().contains("UNIQUE constraint failed") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn login(
    State(db): State<DB>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<String>, StatusCode> {
    let username = payload["username"].as_str().unwrap();
    let password = payload["password"].as_str().unwrap();

    let user = Users::find()
        .filter(Column::Username.eq(username.to_string()))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) if verify(password, &user.password).unwrap() => {
            let payload = Claims {
                sub: username.to_string(),
                exp: chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::minutes(60))
                    .unwrap()
                    .timestamp() as usize,
            };
            let token = encode(
                &Header::default(),
                &payload,
                &EncodingKey::from_secret(KEY),
            )
            .unwrap();
            Ok(Json(token))
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
