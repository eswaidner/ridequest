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
    http: reqwest::Client,
    env: Environment,
}

// parsed environment variables
struct Environment {
    strava_client_id: i32,
    strava_client_secret: String,
    db_url: String,
}

// database athlete model
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct _Athlete {
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
    // load and parse environment variables
    let env = Environment {
        strava_client_id: from_str::<i32>(
            &std::env::var("STRAVA_CLIENT_ID").expect("STRAVA_CLIENT_ID is unset"),
        )
        .unwrap(),
        strava_client_secret: std::env::var("STRAVA_CLIENT_SECRET")
            .expect("STRAVA_CLIENT_SECRET is unset"),
        db_url: std::env::var("DATABASE_URL").expect("database url is not set"),
    };

    // create connection pool to database
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env.db_url)
        .await
        .expect("failed to connect to database");

    // initialize app state
    let state = Arc::new(AppState {
        db: db_pool.clone(),
        http: reqwest::Client::new(),
        env: env,
    });

    // configure cors middleware
    // localhost:5173 (app) and localhost:5174 (api) are separate origins
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // configure REST router
    let app = Router::new()
        .route(&path("/healthcheck"), get(handle_healthcheck))
        .route(&path("/auth/login"), post(handle_login))
        .route(&path("/auth/logout"), post(handle_logout))
        .with_state(state)
        .layer(cors);

    println!("Server starting....");

    // launch server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5174").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// add api prefix to endpoint path
fn path(path: &str) -> String {
    return format!("{API_PREFIX}{path}");
}

//TODO refactor into handlers module
// HANDLERS ----------------------------------------------------

async fn handle_healthcheck() -> impl IntoResponse {
    return "healthy\n";
}

// login request body
#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    auth_code: Option<String>,
}

// token exchange response body
#[derive(Debug, Serialize, Deserialize)]
struct StravaAuthResponse {
    token_type: String,
    expires_at: i64,
    expires_in: i64,
    refresh_token: String,
    access_token: String,
    athlete: StravaAthleteSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct StravaAthleteSummary {
    id: i64,
    username: String,
    firstname: String,
    lastname: String,
    profile_medium: String,
}

async fn handle_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // handle existing session
    if let Some(c) = jar.get("session") {
        //TODO get existing session in db
        // let result: (String, String, DateTime<Utc>) =
        //     sqlx::query_as("SELECT (refresh_token, access_token, access_expires_at) FROM session WHERE session.uuid = $1")
        //     .bind(c.value()).fetch_one(&state.db).await.expect("Failed to get session");

        // bad request if session id is invalid
        //TODO only bad request if auth code was not provided
        return match Uuid::from_str(c.value()) {
            Ok(_) => Ok(jar),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        };
    }

    // bad request if no existing session or auth code provided
    let auth_code = match body.auth_code {
        Some(code) => code,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    // create a new session
    let session_id = create_session(&auth_code, state)
        .await
        .expect("Failed to create session");

    // return session uuid as http-only cookie
    let session_cookie = Cookie::build(Cookie::new("session", session_id.to_string()))
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::None);

    // set the session cookie in the response
    let new_jar = jar.add(session_cookie);

    return Ok(new_jar);
}

// perform token exchange and upsert session into db
async fn create_session(auth_code: &str, state: Arc<AppState>) -> Result<Uuid, reqwest::Error> {
    let session_id = Uuid::new_v4();

    let token_exchange_body = json!({
        "client_id": &state.env.strava_client_id,
        "client_secret": &state.env.strava_client_secret,
        "code": auth_code,
        "grant_type": "authorization_code",
    });

    let token_exchange_resp = state
        .http
        .post("https://www.strava.com/api/v3/oauth/token")
        .json(&token_exchange_body)
        .send()
        .await
        .expect("failed to get tokens from strava");

    let resp = match token_exchange_resp.error_for_status() {
        Ok(resp) => resp,
        Err(e) => return Err(e),
    };

    let auth_data: StravaAuthResponse = resp.json().await.unwrap();

    // create athlete if not exists
    sqlx::query(
        "
            INSERT INTO athlete (id, username, created_at, updated_at)
            VALUES ($1, $2, now(), now())
            ON CONFLICT (id) DO NOTHING
        ",
    )
    .bind(auth_data.athlete.id)
    .bind(auth_data.athlete.username)
    .execute(&state.db)
    .await
    .expect("Failed to create athlete");

    let expires_at = DateTime::from_timestamp(auth_data.expires_at, 0).unwrap();

    // create or update session
    sqlx::query("
            INSERT INTO session (uuid, athlete_id, refresh_token, access_token, access_expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (athlete_id) DO UPDATE
            SET uuid = EXCLUDED.uuid,
                refresh_token = EXCLUDED.refresh_token,
                access_token = EXCLUDED.access_token,
                access_expires_at = EXCLUDED.access_expires_at,
                created_at = now()
        ").bind(&session_id)
        .bind(auth_data.athlete.id)
        .bind(auth_data.refresh_token)
        .bind(auth_data.access_token)
        .bind(expires_at).execute(&state.db).await.expect("Failed to create session");

    return Ok(session_id);
}

// delete a session from the db and unset the session cookie
async fn handle_logout(State(state): State<Arc<AppState>>, jar: CookieJar) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session") {
        println!("Logout Session: {}", cookie.value());

        // get session id from session cookie
        let session_id = Some(Uuid::from_str(cookie.value()).expect("Invalid session uuid"));

        // delete session by uuid
        sqlx::query("DELETE FROM session WHERE uuid = $1")
            .bind(session_id)
            .execute(&state.db)
            .await
            .expect("Failed to delete session");

        // unset session cookie
        return jar.remove("session");
    } else {
        println!("no session cookie");
        return jar;
    }
}
