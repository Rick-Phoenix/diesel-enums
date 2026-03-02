use crate::*;

pub(crate) fn pg_enum_derive_impl(item: &ItemEnum) -> syn::Result<TokenStream2> {
	check_features()?;

	let PgContainerAttrs {
		sql_type_path,
		pg_enum_name,
		common_attrs: CommonAttrs {
			test_runner,
			case,
			skip_test,
		},
	} = PgContainerAttrs::parse(&item.ident, &item.attrs, IsDerive::Yes)?;

	let enum_ident = &item.ident;

	let mut conversion_to_bytes = TokenStream2::new();
	let mut conversion_from_bytes = TokenStream2::new();
	let mut conversion_to_str = TokenStream2::new();
	let mut conversion_from_str = TokenStream2::new();
	let mut db_variant_names: Vec<String> = Vec::new();

	for variant in &item.variants {
		let db_name = variant.ident.to_string().to_case(case);
		let variant_ident = &variant.ident;
		let span = variant_ident.span();

		let db_name_bytes =
			syn::LitByteStr::new(db_name.as_bytes(), proc_macro2::Span::call_site());

		conversion_to_bytes.extend(quote_spanned! {span=>
		  Self::#variant_ident => out.write_all(#db_name_bytes)?,
		});

		conversion_from_bytes.extend(quote_spanned! {span=>
		  #db_name_bytes => Ok(Self::#variant_ident),
		});

		conversion_to_str.extend(quote_spanned! {span=>
			Self::#variant_ident => #db_name,
		});

		conversion_from_str.extend(quote_spanned! {span=>
			#db_name => Ok(Self::#variant_ident),
		});

		db_variant_names.push(db_name);
	}

	let generated_test = test_runner
		.filter(|_| !skip_test)
		.map(|test_runner| test_runner.generate_test(EnumKind::PgEnum, enum_ident));

	Ok(quote! {
	  impl ::diesel_enums::PgEnum for #enum_ident {
			const PG_ENUM_NAME: &str = #pg_enum_name;
			const RUST_ENUM_NAME: &str = stringify!(#enum_ident);
			const VARIANT_MAPPINGS: &[&str] = &[ #(#db_variant_names),* ];

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
						enum_name: stringify!(#enum_ident),
						variant: name.to_string()
					})
				}
			}
	  }

	  impl ::diesel::deserialize::FromSql<#sql_type_path, ::diesel::pg::Pg> for #enum_ident
	  {
			fn from_sql(bytes: ::diesel::pg::PgValue<'_>) -> ::diesel::deserialize::Result<Self> {
				match bytes.as_bytes() {
					#conversion_from_bytes
					unknown => Err(::diesel_enums::UnknownVariantError {
						enum_name: stringify!(#enum_ident),
						variant: String::from_utf8_lossy(unknown).into_owned(),
					}.into()),
				}
			}
	  }

	  impl ::diesel::serialize::ToSql<#sql_type_path, ::diesel::pg::Pg> for #enum_ident
	  {
			fn to_sql<'b>(&'b self, out: &mut ::diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>) -> ::diesel::serialize::Result {
				use std::io::Write;

				match *self {
					#conversion_to_bytes
				};

				Ok(::diesel::serialize::IsNull::No)
			}
	  }

	  #generated_test
	})
}

pub(crate) fn pg_enum_proc_macro(item: &ItemEnum) -> syn::Result<TokenStream2> {
	let PgContainerAttrs { sql_type_path, .. } =
		PgContainerAttrs::parse(&item.ident, &item.attrs, IsDerive::No)?;

	Ok(quote! {
		#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
		#[derive(::diesel_enums::PgEnum, ::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression)]
	  #[diesel(sql_type = #sql_type_path)]
	  #item
	})
}

pub(crate) fn pg_enum_fallback_impl(enum_ident: &Ident) -> TokenStream2 {
	quote! {
		impl ::diesel_enums::PgEnum for #enum_ident {
			const PG_ENUM_NAME: &str = stringify!(#enum_ident);
			const RUST_ENUM_NAME: &str = stringify!(#enum_ident);
			const VARIANT_MAPPINGS: &[&str] = &[];

			fn db_name(self) -> &'static str {
				unimplemented!()
			}

			fn from_db_name(name: &str) -> Result<Self, ::diesel_enums::UnknownVariantError> {
				unimplemented!()
			}
		}

		impl<T> ::diesel::deserialize::FromSql<T, ::diesel::pg::Pg> for #enum_ident
		{
			fn from_sql(bytes: ::diesel::pg::PgValue<'_>) -> ::diesel::deserialize::Result<Self> {
				unimplemented!()
			}
		}

		impl<T> ::diesel::serialize::ToSql<T, ::diesel::pg::Pg> for #enum_ident
		{
			fn to_sql<'b>(&'b self, out: &mut ::diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>) -> ::diesel::serialize::Result {
				unimplemented!()
			}
		}
	}
}
