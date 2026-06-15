//   let message: Option<Message> = match msg {
//     Message::Text(text) => {
//       let socket_message: SocketMessage<String> = serde_json::from_str(&text).unwrap();

//       let response = if socket_message.r#type == "connected" {
//         Some(Message::Text(
//           serde_json::to_string(&SocketMessage::<String> {
//             r#type: "connect".to_string(),
//             data: None,
//           })
//           .unwrap(),
//         ))
//       } else if socket_message.r#type == "get-games" {
//         let games =
//           battleship::get_games_by_state(&state.pool, CONSTANTS.awaiting.to_string()).await;

//         let games = match games {
//           Ok(games) => match games {
//             Some(games) => games,
//             None => Vec::new(),
//           },
//           Err(_) => Vec::new(),
//         };

//         let mut response_game: Vec<BattleshipsGame> = Vec::new();

//         for game in games {
//           let belongs_to_username = user::user_exist_by_id(&state.pool, &game.belongs_to)
//             .await
//             .unwrap()
//             .unwrap()
//             .username;

//           response_game.push(BattleshipsGame {
//             id: game.id.to_string(),
//             state: game.state.clone(),
//             belongs_to: belongs_to_username,
//             stake: game.stake.0,
//             pool: game.pool.0,
//             ready: false,
//             winner: None,
//             opponent: None,
//             clicks: Vec::new(),
//             ships: Vec::new(),
//             turn: game.turn.to_string(),
//           })
//         }

//         Some(Message::Text(
//           serde_json::to_string(&SocketMessage {
//             r#type: "get-games".to_string(),
//             data: Some(response_game),
//           })
//           .unwrap(),
//         ))
//       } else {
//         None
//       };

//       response
//     }
//     _ => None,
//   };

//   match message {
//     Some(message) => message,
//     None => Message::Text("Socket doesn't exist".to_string()),
//   }
//   // msg
// } else {
//   // client disconnected
//   return;
// };

// if socket.send(msg).await.is_err() {
//   // client disconnected
//   return;
// }
