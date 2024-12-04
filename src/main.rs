use axum::{Router, routing::get, extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json;

async fn handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "message": "Hello, world!", "data": "Hello, world!" }))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    println!("Listening on http://0.0.0.0:3000");
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    axum::serve(
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
