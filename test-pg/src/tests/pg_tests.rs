use super::{PgRunner, models::*, schema::*};
use diesel_enums::{AsyncTestRunner, pg_enum};

macro_rules! run_test {
	($target:ident) => {
		PgRunner::check_pg_enum::<$target>().await
	};
}

#[tokio::test]
async fn you_shall_pass() {
	run_test!(PokemonTypes).unwrap()
}

mod wrong_casing {
	use super::*;

	#[pg_enum]
	#[db(skip_test, case = "UPPERCASE", sql_type = sql_types::PokemonType)]
	enum PokemonTypes {
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

	#[tokio::test]
	async fn wrong_casing() {
		let errors = run_test!(PokemonTypes).unwrap_err();

		assert_eq!(errors.missing_from_db.len(), 18);
		assert_eq!(errors.missing_from_rust.len(), 18);
	}
}

mod missing_db_variant {
	use super::*;

	#[pg_enum]
	#[db(skip_test, sql_type = sql_types::PokemonType)]
	enum PokemonTypes {
		// Grass,
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

	#[tokio::test]
	async fn missing_db_variant() {
		let errors = run_test!(PokemonTypes).unwrap_err();

		let e = errors.missing_from_rust.first().unwrap();

		assert_eq!(e, "grass");
	}
}

mod extra_variant {
	use super::*;

	#[pg_enum]
	#[db(skip_test, sql_type = sql_types::PokemonType)]
	enum PokemonTypes {
		NotAPokemonType,
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

	#[tokio::test]
	async fn extra_variant() {
		let errors = run_test!(PokemonTypes).unwrap_err();

		let e = errors.missing_from_db.first().unwrap();

		assert_eq!(e, "not_a_pokemon_type");
	}
}
