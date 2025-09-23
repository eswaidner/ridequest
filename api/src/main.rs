use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderValue, Method, StatusCode,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

const API_PREFIX: &str = "/api/v1";

// global app state
struct AppState {
    db: PgPool,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct Athlete {
    id: i32,
    username: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    max_speed: f32,
    max_power: f32,
    max_cadence: f32,
    total_distance: f32,
    total_ascension: f32,
    total_energy: f32,
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").expect("database url is not set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to database");

    let state = Arc::new(AppState {
        db: db_pool.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .route(&path("/healthcheck"), get(healthcheck_handler))
        .route(&path("/auth/login"), post(login_handler))
        .route(&path("/auth/logout"), post(logout_handler))
        .with_state(state)
        .layer(cors);

    println!("Server starting....");

    // launch server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5174").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck_handler() -> impl IntoResponse {
    return "healthy\n";
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    auth_code: Option<String>,
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    //TODO if auth_code, get tokens from strava

    if let Some(auth_code) = body.auth_code {
        println!("AUTH CODE: {}", auth_code);
        //TODO get or create session in db
    } else {
        //TODO get session in db
    }

    // return session uuid

    let session_id = Uuid::nil();

    let session_cookie = Cookie::build(Cookie::new("session", session_id.to_string()))
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::None);

    let new_jar = jar.add(session_cookie);

    return Ok(new_jar);
}

async fn logout_handler(State(state): State<Arc<AppState>>, jar: CookieJar) -> impl IntoResponse {
    let mut new_jar = jar.clone();

    if let Some(cookie) = jar.get("session") {
        println!("Logout Session: {}", cookie.value());
        new_jar = jar.remove("session");
    } else {
        println!("no session cookie");
    }

    return new_jar;
}

fn path(path: &str) -> String {
    return format!("{API_PREFIX}{path}");
}
