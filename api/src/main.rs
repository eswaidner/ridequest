use std::{str::FromStr, sync::Arc};

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
use serde_json::{from_str, json};
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

#[derive(Debug, Serialize, Deserialize)]
struct StravaAuthResponse {
    token_type: String,
    expires_at: i64,
    expires_in: i64,
    refresh_token: String,
    access_token: String,
    athlete: AthleteSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct AthleteSummary {
    id: i64,
    username: String,
    firstname: String,
    lastname: String,
    profile_medium: String,
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut session_id: Option<Uuid> = None;

    if let Some(auth_code) = body.auth_code {
        println!("AUTH CODE: {}", auth_code);

        let client = reqwest::Client::new();

        let client_id =
            from_str::<i32>(&std::env::var("STRAVA_CLIENT_ID").expect("CLIENT_ID is unset"))
                .unwrap();

        let body = json!({
            "client_id": client_id,
            "client_secret": std::env::var("STRAVA_CLIENT_SECRET").expect("CLIENT_SECRET is unset"),
            "code": auth_code,
            "grant_type": "authorization_code",
        });

        let resp = client
            .post("https://www.strava.com/api/v3/oauth/token")
            .json(&body)
            .send()
            .await
            .expect("failed to get tokens from strava");

        if resp.status().is_success() {
            let authData: StravaAuthResponse = resp.json().await.unwrap();
            println!("STRAVA: {:?}", authData);

            // create athlete if not exists
            sqlx::query(
                "
                INSERT INTO athlete (id, username, created_at, updated_at)
                VALUES ($1, $2, now(), now())
                ON CONFLICT (id) DO NOTHING
            ",
            )
            .bind(authData.athlete.id)
            .bind(authData.athlete.username)
            .execute(&state.db)
            .await
            .expect("Failed to create athlete");

            let expires_at = DateTime::from_timestamp(authData.expires_at, 0).unwrap();

            session_id = Some(Uuid::new_v4());

            // create session
            sqlx::query("
                INSERT INTO session (uuid, athlete_id, refresh_token, access_token, access_expires_at, created_at)
                VALUES ($1, $2, $3, $4, $5, now())
                ON CONFLICT (athlete_id) DO UPDATE
                SET uuid = EXCLUDED.uuid,
                    refresh_token = EXCLUDED.refresh_token,
                    access_token = EXCLUDED.access_token,
                    access_expires_at = EXCLUDED.access_expires_at,
                    created_at = now()
            ").bind(session_id.clone().unwrap())
            .bind(authData.athlete.id)
            .bind(authData.refresh_token)
            .bind(authData.access_token)
            .bind(expires_at).execute(&state.db).await.expect("Failed to create session");
        } else {
            if let Err(e) = resp.error_for_status() {
                eprintln!("TOKEN EXCHANGE ERROR: {}", e);
            }
        }
    } else if let Some(c) = jar.get("session") {
        // get session in db
        // let result: (String, String, DateTime<Utc>) =
        //     sqlx::query_as("SELECT (refresh_token, access_token, access_expires_at) FROM session WHERE session.uuid = $1")
        //     .bind(c.value()).fetch_one(&state.db).await.expect("Failed to get session");

        //TODO handle invalid session uuid with bad request error
        session_id = Some(Uuid::from_str(c.value()).expect("Invalid session uuid"));
    }

    // return session uuid

    if let Some(id) = session_id {
        let session_cookie = Cookie::build(Cookie::new("session", id.to_string()))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::None);

        let new_jar = jar.add(session_cookie);

        return Ok(new_jar);
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
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
