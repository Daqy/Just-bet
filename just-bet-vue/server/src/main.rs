mod models;
mod controllers;

use axum::{Router, routing};
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

use crate::controllers::user;
// use crate::models::user::User;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
      .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let app = Router::new().nest(
        "/api",
        Router::new()
          .route("/hello-world", routing::get(|| async { "hello world" }))
          .route("/login", routing::post(user::login))
          // .with_state(pool),
    );

    // use self::models::schema::users::dsl::*;
    //
    // let connection = &mut establish_connection();
    // let results = users
    //   .limit(5)
    //   .select(User::as_select())
    //   .load(connection)
    //   .expect("Error loading users");
    //
    // println!("Displaying {} users", results.len());
    // for user in results {
    //     println!("username: {}", user.username);
    //     println!("email: {}", user.email);
    //     println!("password: {}", user.password_hash);
    //     println!("balance: {}", user.balance.0);
    //     println!("expiry: {}", user.claim_expires_timestamp.0);
    // }

    println!("server listen on port : {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

