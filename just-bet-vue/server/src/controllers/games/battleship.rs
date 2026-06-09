use std::sync::Arc;

use axum::{
  Json,
  extract::{
    State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
};

use crate::AppState;

#[axum::debug_handler]
pub async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
  ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(serde::Deserialize, Debug)]
struct SocketMessage {
  r#type: String,
  data: Option<serde_json::Value>,
}

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
  // let _ = socket.send(Message::Text(format!("connected"))).await;

  while let Some(msg) = socket.recv().await {
    let msg = if let Ok(msg) = msg {
      println!("msg, {:?}", msg);

      let message: Option<Message> = match msg {
        Message::Text(text) => {
          let socket_message: SocketMessage = serde_json::from_str(&text).unwrap();

          let response = if socket_message.r#type == "connected" {
            Some(Message::Text("User connected".to_string()))
          } else if socket_message.r#type == "get_games" {
            // Some()
            None
          } else {
            None
          };
          // if socket_message.r#type == "connected" {
          //   return Some(Message::Text("User connected".to_string()));
          // }
          response
        }
        _ => None,
      };

      match message {
        Some(message) => message,
        None => Message::Text("Socket doesn't exist".to_string()),
      }
      // msg
    } else {
      // client disconnected
      return;
    };

    if socket.send(msg).await.is_err() {
      // client disconnected
      return;
    }
    // if let Ok(msg) = msg {

    //   match msg {
    //     Message::Text(text) => {
    //       println!("{:?}", socket_message);

    //         // let result = socket.send(Message::Text(format!("User connected"))).await;
    //         // msg
    //       }
    //       // let result = socket
    //       //   .send(Message::Text(format!("Echo back text: {}", text).into()))
    //       //   .await;
    //       msg
    //     }
    //     _ => {}
    //   }

    //   // msg
    // } else {
    //   // client disconnected
    //   println!("Client disconnected");
    //   return;
    // };

    // let _ = socket.send(Message::Text(format!("testing"))).await;

    // if socket.send(msg.unwrap()).await.is_err() {
    //   // client disconnected
    //   println!("Client disconnected 1");
    //   return;
    // }
  }
}
