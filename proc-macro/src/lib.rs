#![cfg_attr(docsrs, feature(doc_cfg))]

use syn_utils::*;

mod main_logic;
mod pg_enum;

pub(crate) mod attributes;
pub(crate) mod conversions;
pub(crate) mod process_variants;

use std::ops::Range;

use bool_enum::bool_enum;
use convert_case::{Case, Casing, ccase};
use proc_macro::TokenStream;
pub(crate) use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{
	Attribute, Ident, ItemEnum, LitStr, Path, Token, Variant, parse_macro_input, parse_quote,
	punctuated::Punctuated,
};

use crate::{attributes::*, conversions::*, process_variants::*};

fn check_features() -> syn::Result<()> {
	let mut enabled: Vec<&str> = Vec::new();

	macro_rules! check_features {
		($($feat:literal),*) => {
			$(
				if cfg!(feature = $feat) {
					enabled.push($feat);
				}
			)*
		};
	}

	check_features!(
		"default-sqlite-runner",
		"default-pg-runner",
		"crate-runner",
		"async-crate-runner"
	);

	if enabled.len() > 1 {
		bail_call_site!(
			"Found more than one feature for the default runner: {}. Please choose only one",
			enabled.join(", ")
		)
	} else {
		Ok(())
	}
}

/// Shortcut for deriving [`DbEnum`] on the target enum, along with the other required derives for it.
///
/// Derives [`FromSqlRow`](diesel::FromSqlRow), [`AsExpression`](diesel::AsExpression), [`Debug`](std::fmt::Debug), [`Copy`](core::marker::Copy), [`Eq`](core::cmp::Eq) and [`Hash`](core::hash::Hash).
#[proc_macro_attribute]
pub fn db_enum(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	match main_logic::db_enum_proc_macro(&item) {
		Ok(tokens) => tokens.into(),
		Err(e) => {
			let err = e.into_compile_error();

			quote! {
			  #item
			  #err
			}
			.into()
		}
	}
}

/// Implements [`DbEnum`](diesel_enums::DbEnum) on an enum as well as
/// [`FromSql`](diesel::deserialize::FromSql) and [`ToSql`](diesel::serialize::ToSql) with the target SQL type.
///
/// It also implements [`HasTable`](diesel::associations::HasTable) and [`Identifiable`](diesel::associations::Identifiable) for the target enum.
///
/// The target should implement [`Debug`](std::fmt::Debug), [`Copy`](core::marker::Copy), [`Eq`](core::cmp::Eq), [`Hash`](core::hash::Hash), [`FromSqlRow`](diesel::FromSqlRow), and [`AsExpression`](diesel::AsExpression).
/// The [`db_enum`](macro@db_enum) attribute macro can be used to inject these derives automatically.
///
/// This can be used to create a mapping between a Rust enum and a database table with fixed values. For each variant, the ID is assumed to be an auto incrementing integer, but this can be overridden with variant attributes.
///
/// To keep the auto-incrementing inference while excluding one or many IDs, you can use the `skip_ids` attribute as explained below.
///
/// Unless the `skip_test` attribute is used, it generates a test that uses the [`check_db_mapping`](::diesel_enums::DbEnum::check_db_mapping) method to check if the mapping with the database is still valid.
///
/// # Container Attributes
#[doc = include_str!("../docs/common_attrs.md")]
///
/// - `id_type/sql_type`
///   - Example: `#[db(id_type = diesel::sql_types::BigInt)]` or `#[diesel(sql_type = diesel::sql_types::BigInt)]`
///   - Description:
///     - Defines the type to use for the ID mapping. It should be the corresponding type of the id column, and one of the numeric types in [`diesel::sql_types`]. <br/> When this derive is used, this attribute will be extracted by the `#[diesel(sql_type = ..)]` attribute. When the [`db_enum`](macro@db_enum) macro is used, the `#[db(..)]` attribute should be used instead. <br/> Defaults to [`diesel::sql_types::Integer`].
///
/// - `skip_ids`
///   - Example: `#[db(skip_ids(10, 12..15))]`
///   - Description:
///     - When inferring the ID of a given variant, the macro will skip the numbers specified in this list. It can contain single numbers or closed, non-inclusive ranges.
///
/// - `table`
///   - Example: `#[db(table = path::to::table)]`
///   - Description:
///     - The path to the corresponding table in the schema file.
///
/// - `table_name`
///   - Example: `#[db(table_name = "type")]`
///   - Description:
///     - The name of the target table. Defaults to the last segment of the `table` path.
///
/// - `name_column`
///   - Example: `#[db(name_column = custom_column)]`
///   - Description:
///     - Defines the column of the target table where the name of the variant is defined. Defaults to `name`.
///
/// # Variant Attributes
///
/// - `name`
///   - Example: `#[db(name = "custom_name")]`
///   - Description:
///     - Overrides the `case` attribute and sets the database name for the specific variant.
///
/// - `id`
///   - Example: `#[db(id = 150)]`
///   - Description:
///     - Sets the database ID for the specific variant.
#[proc_macro_derive(DbEnum, attributes(db))]
pub fn db_enum_derive_macro(input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	match main_logic::db_enum_derive(&item) {
		Ok(tokens) => tokens.into(),
		Err(e) => {
			let err = e.into_compile_error();
			let fallback = main_logic::db_enum_fallback_impl(&item.ident);

			quote! {
				#fallback
				#err
			}
			.into()
		}
	}
}

/// Shortcut for deriving [`PgEnum`] on the target enum, along with the other required derives for it.
///
/// Derives [`FromSqlRow`](diesel::FromSqlRow), [`AsExpression`](diesel::AsExpression), [`Debug`](std::fmt::Debug), [`Copy`](core::marker::Copy), [`Eq`](core::cmp::Eq) and [`Hash`](core::hash::Hash).
#[proc_macro_attribute]
pub fn pg_enum(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	match pg_enum::pg_enum_proc_macro(&item) {
		Ok(tokens) => tokens.into(),
		Err(e) => {
			let err = e.into_compile_error();

			quote! {
			  #item
			  #err
			}
			.into()
		}
	}
}

/// Implements [`PgEnum`](::diesel_enums::PgEnum) on an enum as well as
/// [`FromSql`](diesel::deserialize::FromSql) and [`ToSql`](diesel::serialize::ToSql) with the target custom type.
///
/// The target should also implement [`Debug`](std::fmt::Debug), [`FromSqlRow`](diesel::FromSqlRow), [`AsExpression`](diesel::AsExpression).
/// The [`pg_enum`](macro@pg_enum) attribute macro can be used to inject these derives automatically.
///
/// This can be used to create a mapping between a postgres enum and a Rust enum. Unless the `skip_test` attribute is used, it generates a test that uses the [`check_db_mapping`](::diesel_enums::PgEnum::check_db_mapping) method to check if the Rust variants and the postgres variants have a valid mapping.
///
/// **NOTE**: It is necessary to add the following to the `diesel.toml` configuration:
///
/// ```toml
/// custom_type_derives = ["diesel::query_builder::QueryId"]
/// ```
///
/// # Attributes
#[doc = include_str!("../docs/common_attrs.md")]
///
/// - `sql_type`
///   - Example: `#[db(sql_type = path::to::Type)]` or `#[diesel(sql_type = path::to::Type)]`
///   - Description:
///     - Sets the custom type to use for this enum. It should point to the type generated by `diesel_cli`, which is usually located inside a module called `sql_types` in the generated schema. <br/> Defaults to `crate::schema::sql_types::#enum_ident`. <br/> When this derive is used, this attribute will be extracted from the `#[diesel(..)]` attribute. When the [`pg_enum`](macro@pg_enum) macro is used, the `#[db(..)]` attribute must be used instead.
///
/// - `name`
///   - Example: `#[db(name = "my_type")]`
///   - Description:
///     - The name of the enum inside postgres. Defaults to the `snake_case`d last segment of the `sql_type` path.
#[proc_macro_derive(PgEnum, attributes(db))]
pub fn pg_enum_derive_macro(input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	match pg_enum::pg_enum_derive_impl(&item) {
		Ok(tokens) => tokens.into(),
		Err(e) => {
			let err = e.into_compile_error();
			let fallback = pg_enum::pg_enum_fallback_impl(&item.ident);

			quote! {
				#fallback
				#err
			}
			.into()
		}
	}
}
