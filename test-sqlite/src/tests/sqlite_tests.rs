use super::{models::PokemonType, schema::*, *};
use diesel_enums::{AsyncTestRunner, DbEnum, db_enum};

macro_rules! run_test {
	($target:ident) => {
		SqliteRunner::check_enum::<$target>().await
	};
}

#[tokio::test]
async fn you_shall_pass() {
	run_test!(PokemonType).unwrap_or_else(|e| panic!("{e}"));
}

mod altered_casing {
	use super::*;

	#[db_enum]
	#[db(async_runner = SqliteRunner, table = types, case = "PascalCase")]
	#[allow(non_camel_case_types)]
	enum Types {
		grass,
		poison,
		fire,
		flying,
		water,
		bug,
		normal,
		electric,
		ground,
		fairy,
		fighting,
		psychic,
		rock,
		steel,
		ice,
		ghost,
		dragon,
		dark,
	}
}

mod wrong_casing {
	use super::*;

	#[db_enum]
	#[db(table = types, skip_test)]
	enum Types {
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
		let errors = run_test!(Types).unwrap_err();

		assert_eq!(errors.missing_from_db.len(), 18);
		assert_eq!(errors.missing_from_rust.len(), 18);
	}
}

mod name_mismatch {
	use super::*;

	#[db_enum]
	#[db(table = types, skip_test, case = "PascalCase")]
	enum Types {
		#[db(name = "abc")]
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
	async fn name_mismatch() {
		let errors = run_test!(Types).unwrap_err();

		assert_eq!(errors.missing_from_db.len(), 1);
		assert_eq!(errors.missing_from_db.first().unwrap(), "abc");

		assert_eq!(errors.missing_from_rust.len(), 1);
		assert_eq!(errors.missing_from_rust.first().unwrap(), "Grass");
	}
}

mod id_mismatch {

	use super::*;

	#[db_enum]
	#[db(table = types, skip_test, case = "PascalCase")]
	enum Types {
		#[db(id = 20)]
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
	async fn id_mismatch() {
		let errors = run_test!(Types).unwrap_err();

		let mismatch = errors.id_mismatches.first().unwrap();

		assert_eq!(mismatch.variant(), "Grass");
		assert_eq!(mismatch.expected, 1);
		assert_eq!(mismatch.found, 20);
	}
}

mod skipped_ids {
	use super::*;

	#[db_enum]
	#[db(skip_test, skip_ids(1..6, 6, 7..=10), table = types, case = "PascalCase")]
	enum Types {
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
	async fn skipped_ids() {
		let errors = run_test!(Types).unwrap_err();

		assert_eq!(errors.id_mismatches.len(), 18);

		#[allow(clippy::cast_possible_wrap)]
		for (i, mismatch) in errors.id_mismatches.iter().enumerate() {
			assert_eq!(mismatch.expected, (i + 1) as i64);
			assert_eq!(mismatch.found, (i + 11) as i64);
		}
	}
}

mod custom_table_name {
	use super::*;

	#[db_enum]
	#[db(async_runner = SqliteRunner, table = types, case = "PascalCase", table_name = "types")]
	enum PokeTypes {
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
}

mod sqlite_queries {
	use diesel::prelude::*;

	use super::run_sqlite_query;
	use super::*;

	#[tokio::test]
	async fn modify() {
		run_sqlite_query(|conn| {
			conn.begin_test_transaction()?;

			diesel::delete(types::table.filter(types::id.eq(PokemonType::Poison))).execute(conn)?;

			diesel::insert_into(types::table)
				.values((
					types::id.eq(PokemonType::Poison),
					types::name.eq(PokemonType::Poison.db_name()),
				))
				.execute(conn)?;

			let result: (PokemonType, String) =
				diesel::update(types::table.filter(types::id.eq(PokemonType::Poison)))
					.set((
						types::id.eq(PokemonType::Poison),
						types::name.eq(PokemonType::Poison.db_name()),
					))
					.get_result(conn)?;

			assert_eq!(
				(
					PokemonType::Poison,
					PokemonType::Poison.db_name().to_string()
				),
				result
			);

			Ok(())
		})
		.await
		.unwrap();
	}
}
