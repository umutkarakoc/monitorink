use std::collections::HashMap;

use crate::{layout, logged_user::LoggedUser, AppState};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::get, Router};
use maud::{html, Markup};
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

const TOGB: f64 = 0.000000001;

pub async fn root(
    State(state): State<AppState>,
    LoggedUser(user_id): LoggedUser,
) -> impl IntoResponse {
    let db = state.db;

    let servers = sqlx::query!("select * from server where user_id = $1", user_id)
        .fetch_all(&db)
        .await
        .unwrap();

    let header = html! {
         div class="box" {
            div class="container" {
                h4 class="title" {"Monitorink"}
                }
        }
    };

    fn row(
        id: &Uuid,
        name: &String,
        os: &String,
        version: &String,
        kernel: &String,
        ip: &String,
        states: Option<&HashMap<String, (String, String, String, i64)>>,
    ) -> Markup {
        let empty = HashMap::new();
        let states = if states.is_some() {
            states.unwrap()
        } else {
            &empty
        };

        let mem = states
            .get("system::mem::total")
            .map(|r| r.3.to_owned() as f64)
            .unwrap_or(0f64);
        let mem_gb = format!("{:.2}GB", (mem * TOGB));

        // states.iter().for_each(|k| println!("{} : {}", k.0, k.1 .3));

        let render_percent = |key: &str| {
            let v = states.get(key).as_deref().map(|x| x.3).unwrap_or(0);
            let color = if v < 25 {
                "has-background-primary"
            } else if v < 50 {
                "has-background-warning"
            } else if v < 75 {
                "has-background-danger"
            } else {
                "has-background-danger-dark"
            };

            let size = v / 2;

            html! {
                div class={"box has-background-grey m-0 p-0 " } style="overflow:hidden; width: 50px; height: 24px; text-align: center; position:relative; " {
                    div style={"position: absolute; left:0; top:0; width: "(size)"px; height: 100%"} class=(color) {}

                    span class="subtitle has-text-white is-size-7" style="position: absolute; z-index: 0; left:0; top:0; transform: translate(calc(25px - 50%), calc(12px - 50%))" { (v)"%"}
                }
            }
        };

        let services = vec![
            ("MySql", 3306),
            ("PostgreSQL", 5432),
            ("LiteSpeed", 7080),
            ("SSH", 22),
            ("HTTP", 80),
            ("TLS/SSL", 443),
            ("Redis", 6379),
        ]
        .into_iter()
        .map(|(name, port)| {
            let exist = states
                .get(format!("service::{}::exist", name).as_str())
                .map(|r| r.3)
                .unwrap_or(1i64);
            let running = states
                .get(format!("service::{}::running", name).as_str())
                .map(|r| r.3)
                .unwrap_or(164);
            let port = states
                .get(format!("service::{}::port", name).as_str())
                .map(|r| r.3)
                .unwrap_or(port);

            println!("{} {} {}", name, exist, running);
            let color = if exist != 1 {
                "has-background-grey"
            } else if running == 1 {
                "has-background-primary"
            } else {
                "has-background-danger"
            };
            html! {
                div class={"box p-0 mt-0 ml-0 mr-0 mb-1 subtitle has-text-white is-size-7 " (color)} 
                    style="width:100%; height: 22px; text-align: center; line-height: 22px" { (name) " - " (port) }
            }
        })
        .collect::<Vec<Markup>>();
        let services = html! {
            div class="dropdown is-hoverable is-right" {
                div class="dropdown-trigger" {
                    button class="box has-background-light p-0 " aria-haspopup="true" aria-controls={"services_"(id.to_string()) }
                        style="height:24px; width:50px; border: none; line-height: 22px" {
                        span class="has-text-primary" { ( states.get("service::all::up").map(|r| r.3).unwrap_or(0) ) } "/"
                        span class="has-text-danger" { ( states.get("service::all::down").map(|r| r.3).unwrap_or(0) ) } "/"
                        span class="has-text-dark" { ( states.get("service::all::nope").map(|r| r.3).unwrap_or(0) ) }
                    }
                }

                div class="dropdown-menu" id={"services_"(id.to_string())} role="menu" {
                    div class="dropdown-content p-2" style="min-width: 100px;" {
                        @for s in services.iter() { (s) }
                    }
                }
            }
        };

        html! {
            tr {
                td { (name) }
                td { div {
                    div { (os) " " (version) }
                    div {(kernel)}
                }}

                td { (ip) }
                td { ( states.get("system::cpu::core").map(|r| r.3).unwrap_or(0) ) "C" }
                td { ( mem_gb.to_string() ) }

                td { (render_percent("system::cpu::used")) }
                td { (render_percent("system::load::1m")) }
                td { (render_percent("system::mem::percent")) }
                td { (render_percent("system::swap::percent")) }
                td { (render_percent("disk::1::percent")) }

                td { (services) }

            }
        }
    }

    let table = html! {
        table class="table is-fullwidth" {
            thead {
                tr {
                    td { "Name" }
                    td { "OS" }
                    td { "IP" }
                    td { "CPU" }
                    td { "MEM" }
                    td { "CPU" }
                    td { "1m Load"}
                    td { "Memory" }
                    td { "Swap" }
                    td { "Disk 1" }
                    td { "Services" }
                }
                tbody {
                    @for s in servers.iter() {
                        (row(&s.id, &s.name, &s.os, &s.version, &s.kernel, &s.ip, state.servers.read().unwrap().get(&s.id) ))
                    }
                }
            }
        }
    };

    layout::page(
        None,
        html! {
            div #app class="has-background-light p-2" style="width: 100vw; height: 100vh"
                hx-get="" hx-swap="multi:#app" hx-trigger="every 2s" {
                div class="container" {
                    (header)

                    div class="box" {
                        (table)
                    }
                }
            }
        },
    )
}
