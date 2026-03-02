use diesel::prelude::*;
use diesel_enums::db_enum;

use super::SqliteRunner;

use super::schema::*;

#[derive(Queryable, Selectable, Debug, Identifiable, Insertable)]
pub struct Pokemon {
	pub id: i32,
	pub name: String,
}

// Lookup table used for many-to-many relationship between pokemons and types
#[derive(Identifiable, Associations, Insertable)]
#[diesel(belongs_to(Pokemon))]
#[diesel(belongs_to(PokemonType, foreign_key = type_id))]
#[diesel(primary_key(pokemon_id, type_id))]
#[diesel(table_name = pokemon_types)]
pub struct PokemonTypeRow {
	pub pokemon_id: i32,
	pub type_id: PokemonType,
}

#[db_enum]
#[db(async_runner = SqliteRunner, table = types, case = "PascalCase")]
pub enum PokemonType {
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

// We keep the generic lookup table structure to insert new types in the future
#[derive(Queryable, Selectable, Debug, Insertable, Identifiable)]
#[diesel(table_name = types)]
pub struct Type {
	#[diesel(skip_insertion)]
	pub id: i32,
	pub name: String,
}
