use axum::{response::IntoResponse, routing::get, Router};

const API_PREFIX: &str = "/api/v1";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(&path("/healthcheck"), get(healthcheck));

    println!("Server starting....");

    // launch server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5174").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> impl IntoResponse {
    return "healthy\n";
}

fn path(path: &str) -> String {
    return format!("{API_PREFIX}{path}");
}