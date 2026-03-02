use super::{models::*, schema::*, *};
use diesel::prelude::*;

async fn run_pg_query<T: Send + 'static>(
	callback: impl FnOnce(&mut PgConnection) -> QueryResult<T> + Send + 'static,
) -> Result<T, Box<dyn std::error::Error>> {
	Ok(POSTGRES_POOL
		.get_or_init(|| async { create_pg_pool().await })
		.await
		.get()
		.await
		.expect("Could not get a connection")
		.interact(callback)
		.await??)
}

#[tokio::test]
async fn pg_queries() {
	run_pg_query(|conn| {
		conn.begin_test_transaction()?;

		let new_row = Pokemon {
			name: "Charizard".to_string(),
			type_: PokemonTypes::Fire,
		};

		let inserted_row: Pokemon = diesel::insert_into(pokemons::table)
			.values(&new_row)
			.get_result(conn)?;

		assert_eq!(new_row, inserted_row);

		let selected_row = pokemons::table
			.select(Pokemon::as_select())
			.filter(pokemons::type_.eq(PokemonTypes::Fire))
			.get_result(conn)?;

		assert_eq!(new_row, selected_row);

		let updated_row: Pokemon =
			diesel::update(pokemons::table.filter(pokemons::type_.eq(PokemonTypes::Fire)))
				.set(pokemons::type_.eq(PokemonTypes::Fire))
				.get_result(conn)?;

		assert_eq!(updated_row.type_, PokemonTypes::Fire);

		let deleted_row =
			diesel::delete(pokemons::table.filter(pokemons::type_.eq(PokemonTypes::Fire)))
				.get_result(conn)?;

		assert_eq!(new_row, deleted_row);

		Ok(())
	})
	.await
	.unwrap();
}
