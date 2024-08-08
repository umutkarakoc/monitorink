use axum::{async_trait, extract::{FromRequestParts, FromRef}, http::request::Parts};
use once_cell::sync::Lazy;

use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Debug, Default, Deserialize, Clone )]
pub struct AppConfig {
    pub secret: String,
    pub addr: String,
    pub host: String,
    pub database_url: String,
    pub assets: String,
}

pub static ENV: Lazy<AppConfig> = Lazy::new(|| {
    dotenv::dotenv().ok();
    let config_ = config::Config::builder()
        .add_source(config::Environment::default())
        .build()
        .unwrap();
    let config: AppConfig = config_.try_deserialize().unwrap();

    config
});