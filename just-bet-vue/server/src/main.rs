mod controllers;
mod models;

use anyhow::Ok;
use axum::{Router, routing};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;

use crate::controllers::user;

async fn establish_connection() -> Result<Pool<AsyncPgConnection>, anyhow::Error> {
  dotenv().ok();
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
  Ok(Pool::builder(config).build()?)
}

struct AppState {
  pool: Pool<AsyncPgConnection>,
}

// fn auth(State(state): State<A>) {
//     ()
// }

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

  let pool = establish_connection().await?;

  let state = Arc::new(AppState { pool });

  let app = Router::new()
    .nest(
      "/api",
      Router::new()
        .route("/hello-world", routing::get(|| async { "hello world" }))
        .route("/login", routing::post(user::login))
        .route("/register", routing::post(user::register)),
      //   .layer(middleware::from_fn_with_state(
      //     Arc::clone(&state),
      //     auth
      //     )),

      // .with_state(pool),
    )
    .with_state(state);

  println!("server listen on port : {}", listener.local_addr().unwrap());

  axum::serve(listener, app).await.unwrap();

  Ok(())
}
