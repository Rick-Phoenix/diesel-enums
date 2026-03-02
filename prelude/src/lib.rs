//! # Diesel-enums
//!
//! `diesel-enums` can be used to create mappings between Rust enums and database tables with fixed values, as well as custom postgres enums.
//!
//! It creates a seamless interface with the `diesel` API, and generates the logic to enforce the correctness of the mapping.
//!
//! Refer to the documentation for [`DbEnum`](macro@PgEnum) and [`PgEnum`](macro@PgEnum) to learn more about usage.
//!
//! # Full Example With Sqlite
//!
//!```rust
#![doc = include_str!("../tests/readme_example.rs")]
//!```
//!
//! # Features
#![cfg_attr(
		feature = "document-features",
		doc = ::document_features::document_features!()
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::result_large_err)]

use diesel::query_dsl::methods::SelectDsl;
use diesel::{RunQueryDsl, query_dsl::methods::LoadQuery};
#[doc(inline)]
pub use diesel_enums_proc_macro::*;
use owo_colors::OwoColorize;
use std::{
	collections::HashMap,
	fmt::{Debug, Display},
	hash::Hash,
};
use thiserror::Error;

mod test_runners;
pub use test_runners::*;

/// Maps a Rust enum to a database table with fixed values.
///
/// When derived with the corresponding macro, it implements [`FromSqlRow`](diesel::FromSqlRow), [`AsExpression`](diesel::AsExpression), [`HasTable`](diesel::associations::HasTable) and [`Identifiable`](diesel::associations::Identifiable),
/// as well as [`FromSql`](diesel::deserialize::FromSql) and [`ToSql`](diesel::serialize::ToSql) with the target SQL type.
///
/// It implements the [`check_db_mapping`](DbEnum::check_db_mapping) method, which can be used to verify that the mapping with the database is valid. By default, the macro automatically generates a test that calls this method and panics on error.
pub trait DbEnum: Debug + Hash + Eq + Copy + Sized {
	#[doc(hidden)]
	const VARIANT_MAPPINGS: &[(Self::IdType, &str)];
	#[doc(hidden)]
	const RUST_ENUM_NAME: &str;
	#[doc(hidden)]
	const TABLE_NAME: &str;

	// Constrain that we are allowed to select these two columns from this table
	#[doc(hidden)]
	type Table: diesel::Table + SelectDsl<(Self::IdColumn, Self::NameColumn)> + Default;
	#[doc(hidden)]
	type IdColumn: diesel::Column + Default;
	#[doc(hidden)]
	type NameColumn: diesel::Column + Default;
	#[doc(hidden)]
	type IdType: Copy + Into<i64> + PartialEq + 'static;

	/// Returns the database name for this variant.
	fn db_name(self) -> &'static str;
	/// Attempts to create an instance from a string, if this matches one of the variants in the database.
	fn from_db_name(name: &str) -> Result<Self, UnknownVariantError>;

	/// Returns the database ID for this variant.
	fn db_id(self) -> Self::IdType;
	/// Attempts to create an instance from a number, if this matches the ID of one of the variants in the database.
	fn from_db_id(id: Self::IdType) -> Result<Self, UnknownIdError>;

	/// Verifies that this enum is mapped correctly to a database table.
	///
	/// By default, the derive macro automatically generates a test that calls this method and panics on error.
	#[cold]
	#[inline(never)]
	#[track_caller]
	fn check_db_mapping<'query, Conn>(conn: &mut Conn) -> Result<(), DbEnumError>
	where
		// Constrain that the output of that select query can be loaded into a tuple of (Self::IdType, String)
		<Self::Table as SelectDsl<(Self::IdColumn, Self::NameColumn)>>::Output:
			LoadQuery<'query, Conn, (Self::IdType, String)>,
		Conn: diesel::Connection,
	{
		let db_variants: Vec<(Self::IdType, String)> = Self::Table::default()
			.select((Self::IdColumn::default(), Self::NameColumn::default()))
			.load(conn)
			.unwrap_or_else(|e| {
				panic!(
					"\n ❌ Failed to load the variants for the rust enum `{}` from the database table `{}`: {}",
					Self::RUST_ENUM_NAME,
					Self::TABLE_NAME,
					e
				)
			});

		let mut error =
			DbEnumError::new(Self::RUST_ENUM_NAME, DbEnumSource::Table(Self::TABLE_NAME));

		let mut rust_variants_set: HashMap<&str, Self::IdType> = Self::VARIANT_MAPPINGS
			.iter()
			.map(|(id, name)| (*name, *id))
			.collect();

		for (id, name) in db_variants {
			let rust_variant_id = if let Some(id) = rust_variants_set.remove(name.as_str()) {
				id
			} else {
				error.missing_from_rust.push(name);
				continue;
			};

			if id != rust_variant_id {
				error.id_mismatches.push(IdMismatch {
					variant: name,
					expected: id.into(),
					found: rust_variant_id.into(),
				});
			}
		}

		error.missing_from_db.extend(
			rust_variants_set
				.into_keys()
				.map(|v| v.to_string()),
		);

		if error.is_clean() { Ok(()) } else { Err(error) }
	}
}

#[cfg(feature = "postgres")]
use diesel::connection::LoadConnection;
#[cfg(feature = "postgres")]
/// Maps a Rust enum to a custom enum in postgres.
///
/// When derived with the corresponding macro, it implements [`FromSqlRow`](diesel::FromSqlRow) and [`AsExpression`](diesel::AsExpression),
/// as well as [`FromSql`](diesel::deserialize::FromSql) and [`ToSql`](diesel::serialize::ToSql) with the target SQL type.
///
/// It implements the [`check_db_mapping`](DbEnum::check_db_mapping) method, which can be used to verify that the mapping with the database is valid. By default, the macro automatically generates a test that calls this method and panics on error.
///
/// **NOTE**: It is necessary to add the following to the `diesel.toml` configuration:
///
/// ```toml
/// custom_type_derives = ["diesel::query_builder::QueryId"]
/// ```
pub trait PgEnum: Debug + Sized {
	#[doc(hidden)]
	const VARIANT_MAPPINGS: &[&str];
	#[doc(hidden)]
	const RUST_ENUM_NAME: &str;
	#[doc(hidden)]
	const PG_ENUM_NAME: &str;

	/// Returns the database name for this variant.
	fn db_name(self) -> &'static str;
	/// Attempts to create an instance from a string, if this matches one of the variants in the database.
	fn from_db_name(name: &str) -> Result<Self, UnknownVariantError>;

	/// Verifies that this enum is mapped correctly to the database enum.
	///
	/// By default, the derive macro automatically generates a test that calls this method and panics on error.
	#[cold]
	#[inline(never)]
	#[track_caller]
	fn check_db_mapping<Conn>(conn: &mut Conn) -> Result<(), DbEnumError>
	where
		Conn: diesel::Connection<Backend = diesel::pg::Pg> + LoadConnection,
	{
		use diesel::RunQueryDsl;
		use std::collections::HashSet;

		let mut error = DbEnumError::new(
			Self::RUST_ENUM_NAME,
			DbEnumSource::CustomEnum(Self::PG_ENUM_NAME),
		);

		let mut variants_set: HashSet<&str> = Self::VARIANT_MAPPINGS.iter().copied().collect();

		let pg_variants: Vec<DeserializedPgEnum> = diesel::sql_query(format!(
			"SELECT unnest(enum_range(NULL::{})) AS variant",
			Self::PG_ENUM_NAME
		))
		.load(conn)
		.unwrap_or_else(|_| {
			panic!(
				"\n ❌ Failed to load the variants for the postgres enum {}",
				Self::PG_ENUM_NAME
			)
		});

		for variant in pg_variants {
			let variant_name = variant.variant;

			let was_present = variants_set.remove(variant_name.as_str());

			if !was_present {
				error.missing_from_rust.push(variant_name);
			}
		}

		error
			.missing_from_db
			.extend(variants_set.into_iter().map(|s| s.to_string()));

		if error.is_clean() { Ok(()) } else { Err(error) }
	}
}

#[cfg(feature = "postgres")]
#[derive(diesel::deserialize::QueryableByName)]
struct DeserializedPgEnum {
	#[diesel(sql_type = diesel::sql_types::Text)]
	variant: String,
}

/// An error that can occur when trying to create an instance of a [`DbEnum`] or [`PgEnum`] from a string.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[error("No variant named `{variant}` exists for the enum `{enum_name}`")]
pub struct UnknownVariantError {
	pub enum_name: &'static str,
	pub variant: String,
}

/// An error that can occur when trying to create an instance of a [`DbEnum`] from a number.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[error("The id `{id}` does not match any variant for the enum `{enum_name}`")]
pub struct UnknownIdError {
	pub enum_name: &'static str,
	pub id: i64,
}

/// Represents a mismatch between a Rust variant and a database variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdMismatch {
	variant: String,
	pub expected: i64,
	pub found: i64,
}

impl IdMismatch {
	/// Returns the variant's name for this mismatch.
	pub fn variant(&self) -> &str {
		&self.variant
	}
}

/// An error that is produced when a rust enum does not match a database enum or table.
///
/// It includes the list of errors that may occur simultaneously, such as id mismatches as well as missing variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Error)]
pub struct DbEnumError {
	rust_enum_name: &'static str,
	db_source: DbEnumSource,
	/// The list of variants that are missing from the database.
	pub missing_from_db: Vec<String>,
	/// The list of variants that are missing from the Rust enum.
	pub missing_from_rust: Vec<String>,
	/// The list of ID mismatches between the Rust and database enum.
	///
	/// This is always empty for [`PgEnum`]s since they do not have IDs.
	pub id_mismatches: Vec<IdMismatch>,
}

impl DbEnumError {
	pub(crate) fn new(rust_enum: &'static str, db_source: DbEnumSource) -> Self {
		Self {
			rust_enum_name: rust_enum,
			db_source,
			missing_from_db: vec![],
			missing_from_rust: vec![],
			id_mismatches: vec![],
		}
	}

	pub(crate) fn is_clean(&self) -> bool {
		self.missing_from_db.is_empty()
			&& self.missing_from_rust.is_empty()
			&& self.id_mismatches.is_empty()
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DbEnumSource {
	CustomEnum(&'static str),
	Table(&'static str),
}

impl DbEnumSource {
	pub(crate) fn name(&self) -> &str {
		match self {
			Self::CustomEnum(name) => name,
			Self::Table(name) => name,
		}
	}

	pub(crate) fn db_type(&self) -> &str {
		match self {
			Self::CustomEnum(_) => "enum",
			Self::Table { .. } => "table",
		}
	}
}

#[doc(hidden)]
pub mod __macro_fallbacks {
	#[derive(Default)]
	pub struct DummyTable;
	#[derive(Default)]
	pub struct DummyColumn;

	impl diesel::query_builder::Query for DummyTable {
		type SqlType = i64;
	}

	impl diesel::QuerySource for DummyTable {
		type DefaultSelection = DummyColumn;
		type FromClause = DummyColumn;

		fn default_selection(&self) -> Self::DefaultSelection {
			unimplemented!()
		}

		fn from_clause(&self) -> Self::FromClause {
			unimplemented!()
		}
	}

	impl diesel::Column for DummyColumn {
		type Table = DummyTable;

		const NAME: &'static str = "error";
	}

	impl diesel::SelectableExpression<DummyTable> for DummyColumn {}

	impl diesel::AppearsOnTable<DummyTable> for DummyColumn {}

	impl diesel::Expression for DummyColumn {
		type SqlType = diesel::sql_types::Integer;
	}

	impl diesel::expression::ValidGrouping<()> for DummyColumn {
		type IsAggregate = diesel::expression::is_aggregate::Yes;
	}

	impl diesel::Table for DummyTable {
		type AllColumns = DummyColumn;
		type PrimaryKey = DummyColumn;

		fn primary_key(&self) -> Self::PrimaryKey {
			unimplemented!()
		}

		fn all_columns() -> Self::AllColumns {
			unimplemented!()
		}
	}
}

impl Display for DbEnumError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _ = writeln!(
			f,
			"\n ❌ The rust enum `{}` and the database {} `{}` are out of sync: ",
			self.rust_enum_name.bright_yellow(),
			self.db_source.db_type(),
			self.db_source.name().bright_cyan()
		);

		if !self.missing_from_db.is_empty() {
			let _ = writeln!(
				f,
				"\n  - Variants missing from the {}:",
				"database".bright_cyan()
			);

			for variant in &self.missing_from_db {
				let _ = writeln!(f, "    • {variant}");
			}
		}

		if !self.missing_from_rust.is_empty() {
			let _ = writeln!(
				f,
				"\n  - Variants missing from the {}:",
				"rust enum".bright_yellow()
			);

			for variant in &self.missing_from_rust {
				writeln!(f, "    • {variant}").unwrap();
			}
		}

		if !self.id_mismatches.is_empty() {
			for IdMismatch {
				variant,
				expected,
				found,
			} in &self.id_mismatches
			{
				let _ = writeln!(
					f,
					"\n  - Wrong id mapping for `{}`",
					variant.bright_yellow()
				);
				let _ = writeln!(f, "    Expected: {}", expected.bright_green());
				let _ = writeln!(f, "    Found: {}", found.bright_red());
			}
		}

		Ok(())
	}
}
