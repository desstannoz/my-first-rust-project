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
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
