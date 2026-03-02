use diesel::prelude::*;
use diesel_enums::pg_enum;

use super::{PgRunner, schema::*};

#[derive(Queryable, Selectable, Debug, Insertable, PartialEq, Eq, Clone)]
pub struct Pokemon {
	pub name: String,
	pub type_: PokemonTypes,
}

#[pg_enum]
#[db(async_runner = PgRunner, sql_type = sql_types::PokemonType)]
pub enum PokemonTypes {
	Grass,
	Poison,
	Fire,
	Flying,
	Water,
	Bug,
	Normal,
	Electric,
	Ground,
	Fairy,
	Fighting,
	Psychic,
	Rock,
	Steel,
	Ice,
	Ghost,
	Dragon,
	Dark,
}
