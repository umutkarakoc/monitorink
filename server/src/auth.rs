use crate::AppState;
use crate::ENV;
use async_session::{CookieStore, Session, SessionStore};
use axum::http::HeaderMap;
use axum::response::Html;
use axum::{extract::*, http::StatusCode, response::IntoResponse, routing::*, Router};
use chrono::Utc;
use reqwest::header::{LOCATION, SET_COOKIE};
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use std::ops::Add;
use uuid::Uuid;
use maud::html;
use super::layout;

pub async fn get_login() -> Html<String> {
    layout::page( None,
        html!{
            div style="width: 100vw; height: 100vh; display: flex; justify-content: center;
                align-items: stretch; flex-direction: row"{
            div style="max-width: 600px; padding: 40px; z-index: 100; display: flex; flex-direction: row;
                justify-content: center; align-items: center;" {
                div class="card p-6 is-flex is-flex-direction-column is-flex-justify-content-center" style="max-width: 650px; min-width: 500px; width: 100%;" {

                    h3 class="title is-size-4 mt-6" {"Login to Monitorink"}
                    form hx-post="/auth/send_magic_link" hx-target="this" hx-swap="outerHTML" 
                        class="is-flex is-flex-direction-column mb-4" {
                        div class="field"{
                            label for="email"{}
                            div class="control has-icons-left" {
                                span class="icon" { i class="fa-solid fa-envelope"{} }
                                input id="email" name="email" type="email" required placeholder="email"
                                    value="" class="input"{}
                            }
                        }

                        button class="button is-info" {"Login"}
                    }
                }
            }}
        })
}


#[derive(Deserialize)]
pub struct SendCodeParams {
    pub email: String,
}

pub async fn send_magic_link(
    State(client): State<PgPool>,
    Form(params): Form<SendCodeParams>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let email = params.email.to_string();
    let token = Uuid::new_v4();

    let link = format!("{}/auth/confirm/{}", ENV.host, token);
    println!("link: {}", link);

    sqlx::query!(
        "INSERT INTO magic_link (id, email, token) VALUES ($1, $2, $3)",
        id,
        email,
        token,
    )
    .execute(&client)
    .await
    .unwrap();

    html!{
        form hx-post={"/auth/check/"(id.to_string())} hx-trigger="every 1s"  hx-swap="none" class="is-flex is-flex-direction-column"{
            div class="field"{
                div class="field"{
                    label for="email" {}
                    div class="control has-icons-left" {
                        span class="icon" { i class="fa-solid fa-envelope" {}}
                        input id="email" name="email" type="email" hx-preserve="true" required placeholder="email" 
                            value=(email)  class="input" {}
                    }
                }
                div class="notification"{
                    "We just send a magic login link to your email address. Check your email for confirmation."
                    progress class="progress is-small is-info" max="100" {"15%"}
                }
            }
        }
    }.into_string().into_response()
}

pub async fn confirm(
    State(store): State<CookieStore>,
    State(client): State<PgPool>,
    Path(token): Path<Uuid>,
) -> impl IntoResponse {
    let login_code = sqlx::query!(
        r#"SELECT * FROM magic_link
        WHERE token = $1 and state = 'sent' and created_at > $2
        ORDER BY created_at DESC LIMIT 1"#,
        token,
        Utc::now().add(chrono::Duration::minutes(-30)),
    )
    .fetch_one(&client)
    .await;

    match login_code {
        Ok(login_code) => {
            let exist = sqlx::query!(
                r#"SELECT * FROM "user" WHERE "email" = $1"#,
                login_code.email,
            ).fetch_one(&client).await;

            if exist.is_err() {
                let id = Uuid::new_v4();
                let email = login_code.email.clone();
                let name = login_code.email.clone()[0..login_code.email.find('@').unwrap()].to_string();

                sqlx::query!(
                        r#"INSERT INTO "user" (id, "email", name, created_at, registered_at) VALUES ($1, $2, $3, $4, $5)"#,
                        id,
                        email,
                        name,
                        Utc::now(),
                        Some(Utc::now())
                    )
                    .execute(&client)
                    .await.unwrap();
            }

            sqlx::query!(
                "UPDATE magic_link SET state = 'verified' WHERE id = $1",
                login_code.id
            )
            .execute(&client)
            .await
            .unwrap();

            layout::page(None, html! { 
                app style="width: 100vw; height: 100vh; display: flex; flex-direction:column; 
                    justify-content: center; align-items: center" {
                    h5 class="title is-size-4" { "Your login success"}
                    p class="subtitle is-size=5" {"You can close this window"} 
                }
            }).into_response()
        }
        Err(_err) => 
            layout::page(None, html! { 
                app style="width: 100vw; height: 100vh; display: flex; flex-direction:column; 
                    justify-content: center; align-items: center" {
                    h5 class="title is-size-4" { "Your magic link is timeout"}
                    p class="subtitle is-size=5" {"You can close this window"} 
                }
            }).into_response(),
    }
}

pub async fn check(
    State(client): State<PgPool>,
    State(store): State<CookieStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let magic_link = sqlx::query!(
        "SELECT * FROM magic_link
            WHERE id = $1
            ORDER BY created_at DESC LIMIT 1",
        id
    )
    .fetch_one(&client)
    .await
    .map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR,"unknown error".to_string() ).into_response()
    })
    .unwrap();

    if magic_link.created_at < Utc::now().add(chrono::Duration::minutes(-30)) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "time_out"})),
        ).into_response();
    }

    if magic_link.state != "verified" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "code_not_verified"})),
        ).into_response();
    }

    sqlx::query!(
        "UPDATE magic_link SET state = 'used' WHERE id = $1",
        magic_link.id
    )
    .execute(&client)
    .await
    .unwrap();

    let user = sqlx::query!(
        r#"SELECT * FROM "user" WHERE "email" = $1"#,
        magic_link.email,
    )
    .fetch_one(&client)
    .await.unwrap();

    let mut session = Session::new();
        session.insert("user_id", user.id).unwrap();
        let token = store.store_session(session).await.unwrap().unwrap();

        let cookie = format!(
            "monitorink_token={}; HttpOnly; Max-Age={}; SameSite=None; Secure; Path=/",
            token,
            60 * 60 * 24 * 30 * 5
        );

    let mut headers = HeaderMap::new();
    headers.insert("hx-redirect", "/".parse().unwrap());
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (StatusCode::OK, headers).into_response()
}

async fn logout() -> impl IntoResponse {
    let cookie = format!("monitorink_token=deleted; HttpOnly; SameSite=None; Secure; Path=/");

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers.insert(LOCATION, "/auth/login".parse().unwrap());
    (headers, StatusCode::FOUND).into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login))
        .route("/logout", post(logout).get(logout))
        .route("/send_magic_link", post(send_magic_link))
        .route("/confirm/:token", get(confirm))
        .route("/check/:id", post(check))
}