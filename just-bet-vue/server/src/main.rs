mod controllers;
mod models;

use crate::controllers::{games::minesweeper, user};
use anyhow::Ok;
use axum::{Router, middleware, routing};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

async fn establish_connection() -> Result<Pool<AsyncPgConnection>, anyhow::Error> {
  dotenv().ok();
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
  Ok(Pool::builder(config).build()?)
}

struct AppState {
  pool: Pool<AsyncPgConnection>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

  let pool = establish_connection().await?;

  let state = Arc::new(AppState { pool });

  let auth_routes = Router::new()
    .route("/auth", routing::post(user::auth))
    .route("/get-user", routing::get(user::get))
    .route("/get-balance", routing::get(user::get_balance))
    .route("/game/:id", routing::get(minesweeper::get))
    .route("/latest-game", routing::get(minesweeper::get_latest))
    .route("/create-game", routing::post(minesweeper::create))
    .route(
      "/game/minesweeper/:id/click",
      routing::get(minesweeper::click),
    )
    .layer(middleware::from_fn_with_state(
      Arc::clone(&state),
      user::verify,
    ));

  let app = Router::new()
    .nest(
      "/api",
      Router::new()
        .route("/hello-world", routing::get(|| async { "hello world" }))
        .route("/login", routing::post(user::login))
        .route("/register", routing::post(user::register))
        // .with_state(pool),
        .merge(auth_routes)
        .with_state(state),
    )
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );

  println!("server listen on port : {}", listener.local_addr().unwrap());

  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .compact()
    .init();

  axum::serve(listener, app).await.unwrap();

  Ok(())
}
