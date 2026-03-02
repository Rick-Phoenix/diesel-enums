use crate::*;

pub(crate) fn db_enum_derive(item: &ItemEnum) -> syn::Result<TokenStream2> {
	check_features()?;

	let enum_ident = &item.ident;
	let enum_ident_str = enum_ident.to_string();

	let ContainerAttrs {
		id_type,
		skip_ranges,
		common_attrs: CommonAttrs {
			test_runner,
			case,
			skip_test,
		},
		table_path,
		table_name,
		name_column,
	} = ContainerAttrs::parse(enum_ident, &item.attrs, IsDerive::Yes)?;

	let rust_id_type = id_type.rust_type();
	let id_sql_type = id_type.diesel_type();

	let variants_data = process_variants(&item.variants, case, &skip_ranges)?;

	let mut conversion_to_str = TokenStream2::new();
	let mut conversion_from_str = TokenStream2::new();
	let mut into_int = TokenStream2::new();
	let mut from_int = TokenStream2::new();
	let mut db_variants_tokens = TokenStream2::new();

	// We must use this to use literal numbers for the `to_sql` method
	// rather than `db_id`, because it's hard to satisfy the lifetime requirements,
	// even if `Self::IdType` is marked as 'static
	let mut to_sql_int = TokenStream2::new();

	for data in &variants_data {
		let db_name = &data.db_name;
		let variant_ident = &data.ident;
		let span = variant_ident.span();
		let mut id = proc_macro2::Literal::i32_unsuffixed(data.id);
		id.set_span(span);

		db_variants_tokens.extend(quote_spanned! {span=> (#id, #db_name), });

		into_int.extend(quote_spanned! {span=>
			Self::#variant_ident => #id,
		});

		to_sql_int.extend(quote_spanned! {span=>
			Self::#variant_ident => #id.to_sql(out),
		});

		from_int.extend(quote_spanned! {span=>
			#id => Ok(Self::#variant_ident),
		});

		conversion_to_str.extend(quote_spanned! {span=>
			Self::#variant_ident => #db_name,
		});

		conversion_from_str.extend(quote_spanned! {span=>
			#db_name => Ok(Self::#variant_ident),
		});
	}

	let generated_test = test_runner
		.filter(|_| !skip_test)
		.map(|test_runner| test_runner.generate_test(EnumKind::LookupTable, enum_ident));

	Ok(quote! {
		impl ::diesel_enums::DbEnum for #enum_ident {
			const VARIANT_MAPPINGS: &[(Self::IdType, &str)] = &[ #db_variants_tokens ];
			const RUST_ENUM_NAME: &str = #enum_ident_str;
			const TABLE_NAME: &str = #table_name;

			type IdType = #rust_id_type;
			type Table = #table_path::table;
			type NameColumn = #table_path::dsl::#name_column;
			type IdColumn = #table_path::dsl::id;

			#[inline]
			fn db_name(self) -> &'static str {
				match self {
					#conversion_to_str
				}
			}

			fn from_db_name(name: &str) -> Result<Self, ::diesel_enums::UnknownVariantError> {
				match name {
					#conversion_from_str
					_ => Err(::diesel_enums::UnknownVariantError {
						enum_name: #enum_ident_str,
						variant: name.to_string()
					})
				}
			}

			#[allow(clippy::useless_conversion)]
			fn from_db_id(value: #rust_id_type) -> Result<Self, ::diesel_enums::UnknownIdError> {
				match value {
					#from_int
					_ => Err(::diesel_enums::UnknownIdError {
						enum_name: #enum_ident_str,
						id: value.into()
					}),
				}
			}

			#[inline]
			fn db_id(self) -> #rust_id_type {
				match self {
					#into_int
				}
			}
		}

		impl<DB> diesel::deserialize::FromSql<::diesel::sql_types::Text, DB> for #enum_ident
		where
			DB: diesel::backend::Backend,
			String: diesel::deserialize::FromSql<::diesel::sql_types::Text, DB>,
		{
			fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
				let value = <String as diesel::deserialize::FromSql<::diesel::sql_types::Text, DB>>::from_sql(bytes)?;

				<Self as ::diesel_enums::DbEnum>::from_db_name(&value).map_err(Box::from)
			}
		}

		impl<DB> diesel::serialize::ToSql<::diesel::sql_types::Text, DB> for #enum_ident
		where
			DB: diesel::backend::Backend,
			str: diesel::serialize::ToSql<::diesel::sql_types::Text, DB>,
		{
			fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result {
				<Self as ::diesel_enums::DbEnum>::db_name(self.clone()).to_sql(out)
			}
		}

		impl<DB> diesel::deserialize::FromSql<#id_sql_type, DB> for #enum_ident
		where
			DB: diesel::backend::Backend,
			#rust_id_type: diesel::deserialize::FromSql<#id_sql_type, DB>,
		{
			fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
				let value = #rust_id_type::from_sql(bytes)?;

				Ok(<#enum_ident as ::diesel_enums::DbEnum>::from_db_id(value)?)
			}
		}

		impl<DB> diesel::serialize::ToSql<#id_sql_type, DB> for #enum_ident
		where
			DB: diesel::backend::Backend,
			#rust_id_type: diesel::serialize::ToSql<#id_sql_type, DB>,
		{
			fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result {
				match self {
					#to_sql_int
				}
			}
		}

		impl ::diesel::associations::HasTable for #enum_ident {
			type Table = <Self as ::diesel_enums::DbEnum>::Table;

			#[inline]
			fn table() -> Self::Table {
				<Self as ::diesel_enums::DbEnum>::Table::default()
			}
		}

		impl ::diesel::associations::Identifiable for #enum_ident {
			type Id = Self;

			#[inline]
			fn id(self) -> Self::Id {
				self
			}
		}

		impl ::diesel::associations::Identifiable for &#enum_ident {
			type Id = Self;

			#[inline]
			fn id(self) -> Self::Id {
				self
			}
		}

		#generated_test
	})
}

pub(crate) fn db_enum_proc_macro(item: &ItemEnum) -> syn::Result<TokenStream2> {
	let ContainerAttrs { id_type, .. } =
		ContainerAttrs::parse(&item.ident, &item.attrs, IsDerive::No)?;

	let id_sql_type = id_type.diesel_type();

	Ok(quote! {
		#[derive(PartialEq, Eq, Clone, Copy, Hash, diesel_enums::DbEnum, Debug, diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
	  #[diesel(sql_type = #id_sql_type)]
	  #item
	})
}

pub(crate) fn db_enum_fallback_impl(enum_ident: &Ident) -> TokenStream2 {
	let sql_conversion = sql_conversion_fallback(enum_ident);

	quote! {
		impl ::diesel_enums::DbEnum for #enum_ident {
			const VARIANT_MAPPINGS: &[(Self::IdType, &str)] = &[];
			const RUST_ENUM_NAME: &str = "error";
			const TABLE_NAME: &str = "error";

			type IdType = i32;
			type Table = ::diesel_enums::__macro_fallbacks::DummyTable;
			type NameColumn = ::diesel_enums::__macro_fallbacks::DummyColumn;
			type IdColumn = ::diesel_enums::__macro_fallbacks::DummyColumn;

			fn from_db_id(value: i32) -> Result<Self, ::diesel_enums::UnknownIdError> {
				unimplemented!()
			}

			fn db_id(self) -> i32 {
				unimplemented!()
			}

			fn db_name(self) -> &'static str {
				unimplemented!()
			}

			fn from_db_name(name: &str) -> Result<Self, ::diesel_enums::UnknownVariantError> {
				unimplemented!()
			}
		}

		impl ::diesel::associations::HasTable for #enum_ident {
			type Table = <Self as ::diesel_enums::DbEnum>::Table;

			fn table() -> Self::Table {
				<Self as ::diesel_enums::DbEnum>::Table::default()
			}
		}

		impl diesel::associations::Identifiable for #enum_ident {
			type Id = Self;

			fn id(self) -> Self::Id {
				self
			}
		}

		impl diesel::associations::Identifiable for &#enum_ident {
			type Id = Self;

			fn id(self) -> Self::Id {
				self
			}
		}

		#sql_conversion
	}
}
