use diesel::{
  BelongingToDsl, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, Selectable,
  SelectableHelper,
  associations::{Associations, Identifiable},
  data_types::PgMoney,
  prelude::Insertable,
  query_builder::AsChangeset,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use serde::Serialize;

use crate::models::schema::{battleship, battleship_clicks, battleship_ships};

#[derive(Identifiable, Debug, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Battleship, foreign_key = belongs_to))]
#[diesel(table_name = crate::models::schema::battleship_ships)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct BattleshipShips {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
  pub placed_by: i64,
  pub size: i64,
  pub direction: String,
}

#[derive(Identifiable, Debug, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Battleship, foreign_key = belongs_to))]
#[diesel(table_name = crate::models::schema::battleship_clicks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct BattleshipClicks {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
  pub clicked_by: i64,
}

#[derive(Identifiable, Debug, Queryable, Selectable)]
#[diesel(table_name = crate::models::schema::battleship)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Battleship {
  pub id: i64,
  pub belongs_to: i64,
  pub state: String,
  pub opponent: Option<i64>,
  pub winner: Option<i64>,
  pub turn: i64,
  pub stake: PgMoney,
  pub pool: PgMoney,
}

pub async fn get_game_by_user_id(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
) -> Result<Option<Battleship>, diesel::result::Error> {
  Ok(
    battleship::table
      .filter(battleship::belongs_to.eq(id))
      .or_filter(battleship::opponent.eq(id))
      .select(Battleship::as_select())
      .order_by(battleship::created.desc())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_game_by_user_and_game_id(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
  game_id: i64,
) -> Result<Option<Battleship>, diesel::result::Error> {
  Ok(
    battleship::table
      .filter(battleship::belongs_to.eq(id))
      .or_filter(battleship::opponent.eq(id))
      .filter(battleship::id.eq(game_id))
      .select(Battleship::as_select())
      .order_by(battleship::created.desc())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_games_by_state(
  pool: &Pool<AsyncPgConnection>,
  state: String,
) -> Result<Option<Vec<Battleship>>, diesel::result::Error> {
  Ok(
    battleship::table
      .filter(battleship::state.eq(state))
      .select(Battleship::as_select())
      .order_by(battleship::created.desc())
      .get_results(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_ships_by_user_and_game(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
  game_id: i64,
) -> Result<Option<Vec<BattleshipShips>>, diesel::result::Error> {
  Ok(
    battleship_ships::table
      .filter(battleship_ships::placed_by.eq(id))
      .filter(battleship_ships::belongs_to.eq(game_id))
      .select(BattleshipShips::as_select())
      .get_results(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_clicks_by_user_and_game(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
  game_id: i64,
) -> Result<Option<Vec<BattleshipClicks>>, diesel::result::Error> {
  Ok(
    battleship_clicks::table
      .filter(battleship_clicks::clicked_by.eq(id))
      .filter(battleship_clicks::belongs_to.eq(game_id))
      .select(BattleshipClicks::as_select())
      .get_results(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

#[derive(Insertable)]
#[diesel(table_name = battleship)]
struct NewGame<'a> {
  id: i64,
  belongs_to: i64,
  state: &'a str,
  opponent: Option<i64>,
  winner: Option<i64>,
  turn: i64,
  stake: &'a PgMoney,
  pool: &'a PgMoney,
}

pub async fn create_game(
  pool: &Pool<AsyncPgConnection>,
  game: &Battleship,
) -> Result<Vec<Battleship>, diesel::result::Error> {
  Ok(
    diesel::insert_into(battleship::table)
      .values(&NewGame {
        id: game.id,
        belongs_to: game.belongs_to,
        state: game.state.as_str(),
        opponent: game.opponent,
        winner: game.winner,
        turn: game.turn,
        stake: &game.stake,
        pool: &game.pool,
      })
      .returning(Battleship::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?,
  )
}

pub async fn get_game_by_id(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
) -> Result<Option<Battleship>, diesel::result::Error> {
  Ok(
    battleship::table
      .filter(battleship::id.eq(id))
      .select(Battleship::as_select())
      .order_by(battleship::created.desc())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

#[derive(AsChangeset)]
#[diesel(table_name = battleship)]
pub struct SetGame<'a> {
  pub state: Option<&'a String>,
  pub pool: Option<PgMoney>,
  pub turn: Option<i64>,
  pub opponent: Option<i64>,
}

pub struct UpdateGame {
  pub state: Option<String>,
  pub pool: Option<PgMoney>,
  pub turn: Option<i64>,
  pub opponent: Option<i64>,
}

pub async fn update_game(
  pool: &Pool<AsyncPgConnection>,
  game_id: i64,
  game: &UpdateGame,
) -> Result<Battleship, diesel::result::Error> {
  Ok(
    diesel::update(battleship::table)
      .filter(battleship::id.eq(game_id))
      .set(&SetGame {
        state: game.state.as_ref(),
        turn: game.turn,
        pool: game.pool,
        opponent: game.opponent,
      })
      .returning(Battleship::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?
      .pop()
      .unwrap(),
  )
}

#[derive(Insertable)]
#[diesel(table_name = battleship_ships)]
pub struct CreateShip {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
  pub placed_by: i64,
  pub size: i64,
  pub direction: String,
}

pub async fn create_ships(
  pool: &Pool<AsyncPgConnection>,
  ships: &Vec<CreateShip>,
) -> Result<Vec<BattleshipShips>, diesel::result::Error> {
  Ok(
    diesel::insert_into(battleship_ships::table)
      .values(ships)
      .returning(BattleshipShips::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?,
  )
}
