use super::{models::*, run_sqlite_query, schema::*};
use diesel::prelude::*;
use diesel_enums::DbEnum;

#[tokio::test]
async fn queries() {
	let fire_pokemons_by_id: Vec<String> = run_sqlite_query(|conn| {
		PokemonTypeRow::belonging_to(&PokemonType::Fire)
			.inner_join(pokemons::table)
			.select(pokemons::name)
			.limit(5)
			.load(conn)

		// Equivalent of the above
		// pokemon_types::table
		// 	.inner_join(types::table)
		// 	.inner_join(pokemons::table)
		// 	.filter(types::id.eq(PokemonType::Fire))
		// 	.select(pokemons::name)
		// 	.limit(5)
		// 	.load(conn)
	})
	.await
	.unwrap();

	let fire_pokemons_by_name: Vec<String> = run_sqlite_query(|conn| {
		pokemon_types::table
			.inner_join(types::table)
			.inner_join(pokemons::table)
			// And we can also use the enum to filter by name without
			// relying on plain strings
			.filter(types::name.eq(PokemonType::Fire.db_name()))
			.select(pokemons::name)
			.limit(5)
			.load(conn)
	})
	.await
	.unwrap();

	assert_eq!(fire_pokemons_by_name.len(), 5);
	assert_eq!(fire_pokemons_by_id.len(), 5);

	let fire_pokemons = [
		"Charmander".to_string(),
		"Charmeleon".to_string(),
		"Charizard".to_string(),
		"Vulpix".to_string(),
		"Ninetales".to_string(),
	];

	for pokemon in &fire_pokemons {
		assert!(fire_pokemons_by_name.contains(pokemon));
		assert!(fire_pokemons_by_id.contains(pokemon));
	}
}
