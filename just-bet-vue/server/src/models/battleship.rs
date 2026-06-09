use diesel::{
  BelongingToDsl, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, Selectable,
  SelectableHelper,
  associations::{Associations, Identifiable},
  data_types::PgMoney,
  prelude::Insertable,
  query_builder::AsChangeset,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};

use crate::models::schema::battleship;

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
      .select(Battleship::as_select())
      .order_by(battleship::created.desc())
      .first(&mut pool.get().await.unwrap())
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
