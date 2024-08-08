use crate::appconfig::ENV;
use async_session::CookieStore;
use axum::{extract::FromRef, http::StatusCode, response::IntoResponse, routing::*, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
};
use tower_http::services::ServeDir;
use uuid::Uuid;
mod appconfig;
mod auth;
mod home;
mod layout;
mod logged_user;
mod server;

pub type Servers = Arc<RwLock<HashMap<Uuid, HashMap<String, (String, String, String, i64)>>>>;
#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    session_store: CookieStore,
    servers: Servers,
}
impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.db.clone()
    }
}
impl FromRef<AppState> for CookieStore {
    fn from_ref(app_state: &AppState) -> CookieStore {
        app_state.session_store.clone()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&ENV.database_url)
        .await
        .expect("can connect to database");
    let task_db = db.clone();

    let session_store = CookieStore::new();

    let app_state = AppState {
        db,
        session_store,
        servers: Arc::new(RwLock::new(HashMap::new())),
    };

    let serve_dir = get_service(ServeDir::new(ENV.assets.clone())).handle_error(handle_error);

    let app = Router::new()
        .route("/", get(home::root))
        .nest("/server", server::router())
        .nest("/auth", auth::router())
        .with_state(app_state)
        .fallback_service(serve_dir);

    axum::Server::bind(&ENV.addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
