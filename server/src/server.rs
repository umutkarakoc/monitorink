use std::collections::HashMap;

use crate::{AppState, Servers};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{post, put, Route},
    Json, Router,
};
use chrono::{DateTime, Timelike, Utc};
use http::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
struct InitServer {
    token: Option<Uuid>,
    name: String,
    os: String,
    version: String,
    kernel: String,
    ip: String
}

async fn init(
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(params): Json<InitServer>,
) -> impl IntoResponse {
    sqlx::query!(
        "INSERT INTO server (id, user_id, name, os, version, kernel, ip) 
		VALUES ($1, $2, $3, $4, $5, $6, $7) 
		on conflict (id) do 
        update set user_id=$2, name=$3, os=$4, version=$5, kernel=$6, ip=$7",
        id,
        params
            .token
            .unwrap_or("e58c9a52-a655-4822-853f-11148e178404".parse().unwrap()),
        params.name,
        params.os,
        params.version,
        params.kernel,
        params.ip
    )
    .execute(&db)
    .await
    .unwrap();
}

async fn create_state(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(params): Json<HashMap<String, i64>>,
) -> impl IntoResponse {
    if params.len() == 0 {
        StatusCode::OK
    } else {
        let db = state.db;
        // server_id,t,device_id,key,time
        let now = Utc::now();
        let min = now.timestamp() / 60;

        for param in params {
            let parts = param.0.split("::").collect::<Vec<&str>>();
            if let [t, device_id, key] = parts[..] {
                let res = sqlx::query!(
                    "INSERT INTO resource_state (server_id, device_id, t, key, value, min) 
					VALUES ($1, $2, $3, $4, $5, $6)
					on conflict (server_id,t,device_id,key,min) do update set updated_at = now() , value = $5 ",
                    id,
                    device_id,
                    t,
                    key,
                    param.1,
                    min
                )
                .execute(&db)
                .await;

                let mut servers = state.servers.write().unwrap();
                let server = servers.entry(id).or_insert(HashMap::new());

                *server.entry(param.0.to_string()).or_default() = (
                    t.to_string(),
                    device_id.to_string(),
                    key.to_string(),
                    param.1,
                );

                if let Err(e) = res {
                    println!("Error: {}", e);
                }
            }
        }
        // let params = params.iter().enumerate().map(|(i, (k, v))| {
        // 	let parts = k.split("::").collect::<Vec<&str>>();
        // 	if let [t, device_id, key] = parts[..] {
        // 		Some((t, device_id, key, v))
        // 	} else {
        // 		None
        // 	}
        // }).filter(|x| x.is_some())
        // 	.map(|x| x.unwrap())
        // 	.collect::<Vec<_>>();

        // let query = params.iter().enumerate().map(|(i, _)| {
        // 	let i = i * 5 + 1;
        // 	format!("(${}, ${}, ${}, ${}, ${})", i, i+1, i+2, i+3, i+4)
        // }).collect::<Vec<_>>().join(", ");

        // let query = format!("INSERT INTO resource_state (server_id, device_id, t, key, value)
        // 		VALUES {} ", query);

        // let query = sqlx::query(&query);

        // let query = params.iter().fold(query, |query, param| {
        // 	let query = query.bind(id);
        // 	let query = query.bind(param.0.to_string());
        // 	let query = query.bind(param.1.to_string());
        // 	let query = query.bind(param.2.to_string());
        // 	let query = query.bind(param.3);
        // 	query
        // });

        // query.execute(&db).await.unwrap();

        StatusCode::OK
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", post(init))
        .route("/:id/state", post(create_state))
}
