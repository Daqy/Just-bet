use axum::{routing, Router};

#[tokio::main]
async fn main() {
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  
  let app = Router::new().route("/", routing::get(|| async {"hell world"}));

  println!("server listen on port : {}", listener.local_addr().unwrap());
  
  axum::serve(listener, app).await.unwrap();
}