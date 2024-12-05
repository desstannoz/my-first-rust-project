use axum::{extract::State, http::StatusCode, routing::get,routing::post, Json, Router};
use axum_extra::TypedHeader;
use axum_extra::headers::{authorization::Bearer, Authorization};
use bcrypt::{hash, verify};
use jsonwebtoken::{decode, encode, EncodingKey, Header, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::extract::Request;
use sea_orm::{Database, DatabaseConnection, EntityTrait, ActiveModelTrait, ActiveValue::Set as SetValue};
use sea_orm::ConnectionTrait;
use sea_orm::Statement;
use sea_orm::DatabaseBackend;

mod models;
use models::user::{Entity as Users, ActiveModel as UserActive};

type DB = DatabaseConnection;

async fn register(
    State(db): State<DB>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let username = payload["username"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let password = payload["password"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let hashed_password = hash(password, 10)
        .map_err(|e| {
            eprintln!("Hashing error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let user = UserActive {
        username: SetValue(username.to_string()),
        password: SetValue(hashed_password),
    };

    match user.insert(&db).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("Database error: {}", e);
            // Eğer kullanıcı zaten varsa
            if e.to_string().contains("UNIQUE constraint failed") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

const KEY: &[u8] = b"secret";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Clone)]
struct UserContext {
    username: String,
}

trait RequestExt {
    fn user(&self) -> Option<&UserContext>;
}

impl RequestExt for Request {
    fn user(&self) -> Option<&UserContext> {
        self.extensions().get::<UserContext>()
    }
}

async fn auth_middleware(
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

async fn login(
    State(db): State<DB>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<String>, StatusCode> {
    let username = payload["username"].as_str().unwrap();
    let password = payload["password"].as_str().unwrap();

    let user = Users::find_by_id(username.to_string())
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
            let token = encode(&Header::default(), &payload, &EncodingKey::from_secret(KEY)).unwrap();
            Ok(Json(token))
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

mod controllers;
use controllers::{user, auth};

#[tokio::main]
async fn main() {
    // Veritabani olusturma
    let db_path = "master.db";
    if !std::path::Path::new(db_path).exists() {
        std::fs::File::create(db_path).expect("Could not create database file");
        println!("Database file created: {}", db_path);
    }

    let db = Database::connect(format!("sqlite:{}", db_path))
        .await
        .expect("Failed to connect to database");

    // SQLite tablo oluşturma
    let stmt = Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password TEXT NOT NULL
        )"#.to_owned(),
    );

    db.execute(stmt)
        .await
        .expect("Could not create table");
    
    println!("Database initialized successfully");

    let protected: Router<DatabaseConnection> = Router::new()
        .route("/me", get(user::me))
        .route_layer(middleware::from_fn(auth_middleware));

    let auth = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));

    let app = Router::new()
        .nest("/api", protected)
        .nest("/api/auth", auth)
        .with_state(db);

    println!("Listening on http://0.0.0.0:3000");
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
