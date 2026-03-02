use crate::*;

pub struct ContainerAttrs<'a> {
	pub table_path: Path,
	pub table_name: String,
	pub name_column: Ident,
	pub id_type: IdType,
	pub skip_ranges: Vec<Range<i32>>,
	pub common_attrs: CommonAttrs<'a>,
}

pub struct CommonAttrs<'a> {
	pub test_runner: Option<TestRunner>,
	pub case: Case<'a>,
	pub skip_test: bool,
}

pub enum TestRunner {
	Sync(Path),
	Async(Path),
}

impl TestRunner {
	pub fn default_runner(enum_kind: EnumKind) -> Option<Self> {
		let is_pg_enum = enum_kind.is_pg_enum();

		if cfg!(feature = "default-sqlite-runner") && !is_pg_enum {
			Some(Self::Async(parse_quote!(::diesel_enums::AsyncSqliteRunner)))
		} else if cfg!(feature = "default-pg-runner") {
			Some(Self::Async(parse_quote!(::diesel_enums::AsyncPgRunner)))
		} else if cfg!(feature = "crate-runner") {
			Some(Self::Sync(parse_quote!(
				crate::db_test_runner::DbTestRunner
			)))
		} else if cfg!(feature = "async-crate-runner") {
			Some(Self::Async(parse_quote!(
				crate::db_test_runner::DbTestRunner
			)))
		} else {
			None
		}
	}
}

#[derive(Clone, Copy)]
pub enum EnumKind {
	PgEnum,
	LookupTable,
}

impl EnumKind {
	/// Returns `true` if the enum kind is [`PgEnum`].
	///
	/// [`PgEnum`]: EnumKind::PgEnum
	#[must_use]
	pub const fn is_pg_enum(self) -> bool {
		matches!(self, Self::PgEnum)
	}
}

impl TestRunner {
	pub fn generate_test(&self, enum_kind: EnumKind, enum_ident: &Ident) -> TokenStream2 {
		let test_name = format_ident!("{}_db_enum_check", ccase!(snake, enum_ident.to_string()));
		let method = match enum_kind {
			EnumKind::PgEnum => quote! { check_pg_enum },
			EnumKind::LookupTable => quote! { check_enum },
		};

		match self {
			Self::Sync(path) => {
				quote! {
					#[cfg(test)]
					#[test]
					fn #test_name() {
						if let Err(e) = <#path as ::diesel_enums::SyncTestRunner<_>>::#method::<#enum_ident>() {
							panic!("{e}");
						}
					}
				}
			}
			Self::Async(path) => {
				quote! {
					#[cfg(test)]
					#[tokio::test]
					async fn #test_name() {
						if let Err(e) = <#path as ::diesel_enums::AsyncTestRunner<_>>::#method::<#enum_ident>().await {
							panic!("{e}");
						}
					}
				}
			}
		}
	}
}

pub struct PgContainerAttrs<'a> {
	pub sql_type_path: Path,
	pub pg_enum_name: String,
	pub common_attrs: CommonAttrs<'a>,
}

bool_enum!(pub IsDerive);

#[derive(Default)]
pub enum IdType {
	I8,
	I16,
	#[default]
	I32,
	I64,
}

impl IdType {
	pub fn from_path(path: &Path) -> syn::Result<Self> {
		let ident = path
			.last_segment()
			.expect("Empty path")
			.ident
			.to_string();

		match ident.as_str() {
			"TinyInt" => Ok(Self::I8),
			"SmallInt" => Ok(Self::I16),
			"Integer" => Ok(Self::I32),
			"BigInt" => Ok(Self::I64),
			_ => {
				bail!(
					path,
					"Unknown sql type. Must be one of `TinyInt`, `SmallInt`, `Integer` or `BigInt` from `diesel::sql_types`"
				)
			}
		}
	}

	pub fn rust_type(&self) -> TokenStream2 {
		match self {
			Self::I8 => quote! { i8 },
			Self::I16 => quote! { i16 },
			Self::I32 => quote! { i32 },
			Self::I64 => quote! { i64 },
		}
	}

	pub fn diesel_type(&self) -> TokenStream2 {
		match self {
			Self::I8 => quote! { ::diesel::sql_types::TinyInt },
			Self::I16 => quote! {  ::diesel::sql_types::SmallInt },
			Self::I32 => quote! {  ::diesel::sql_types::Integer },
			Self::I64 => quote! {  ::diesel::sql_types::BigInt },
		}
	}
}

impl ContainerAttrs<'_> {
	pub fn parse(
		enum_ident: &Ident,
		attrs: &[Attribute],
		is_derive: IsDerive,
	) -> syn::Result<Self> {
		let mut table_name: Option<String> = None;
		let mut table_path: Option<Path> = None;
		let mut name_column: Option<Ident> = None;
		let mut test_runner: Option<TestRunner> = None;
		let mut case: Option<Case> = None;
		let mut id_type: Option<IdType> = None;
		let mut skip_ids: Option<Vec<Range<i32>>> = None;
		let mut skip_test = false;

		for attr in attrs {
			if attr.path().is_ident("db") {
				attr.parse_nested_meta(|meta| {
          let ident_str = meta.ident_str()?;

          match ident_str.as_str() {
						"skip_test" => {
							skip_test = true;
						}
            "id_type" => {
              let path = meta.parse_value::<Path>()?;

              id_type = Some(IdType::from_path(&path)?);
            }
            "skip_ids" => {
              let ranges = meta.parse_list::<ClosedRangeList>()?;

              skip_ids = Some(ranges.list);
            }
            "table" => {
              table_path = Some(meta.parse_value::<Path>()?);
            }
            "table_name" => {
              table_name = Some(meta.parse_value::<LitStr>()?.value());
            }
            "name_column" => {
              name_column = Some(meta.parse_value::<Ident>()?);
            }
            "async_runner" => {
              test_runner = Some(TestRunner::Async(meta.parse_value::<Path>()?));
            }
						"sync_runner" => {
              test_runner = Some(TestRunner::Sync(meta.parse_value::<Path>()?));
            }
            "case" => {
              let case_value = match meta.parse_value::<LitStr>()?.value().as_str() {
                "snake_case" => Case::Snake,
                "UPPER_SNAKE" => Case::UpperSnake,
                "camelCase" => Case::Camel,
                "PascalCase" => Case::Pascal,
                "lowercase" => Case::Lower,
                "UPPERCASE" => Case::Upper,
                "kebab-case" => Case::Kebab,
                _ => {
                  return Err(error!(
                    meta.path,
                    "Invalid value for `case`. Allowed values are: [ snake_case, UPPER_SNAKE, camelCase, PascalCase, lowercase, UPPERCASE, kebab-case ]"
                  ));
                }
              };

              case = Some(case_value);
            }
            _ => return Err(meta.error(
              "Unknown attribute"
            ))
          };

          Ok(())
        })?;
			} else if *is_derive && id_type.is_none() && attr.path().is_ident("diesel") {
				attr.parse_nested_meta(|meta| {
					if meta.path.is_ident("sql_type") {
						let path = meta.parse_value::<Path>()?;

						id_type = Some(IdType::from_path(&path)?);
					}

					drain_token_stream!(meta.input);

					Ok(())
				})?;
			}
		}

		if skip_test && test_runner.is_some() {
			bail_call_site!("Cannot use `skip_test` if a runner is specified");
		}

		if !skip_test && test_runner.is_none() {
			if let Some(default_runner) = TestRunner::default_runner(EnumKind::LookupTable) {
				test_runner = Some(default_runner);
			} else {
				bail_call_site!(
					"Missing test runner. If you want to use one of the default runners, enable it as a feature. If you want to skip the automatically generated test, use the `skip_test` attribute"
				);
			}
		}

		let table_path = table_path.unwrap_or_else(|| {
			let pluralized_name = format_ident!("{}s", ccase!(snake, enum_ident.to_string()));

			parse_quote!( crate::schema::#pluralized_name )
		});

		let table_name = table_name.unwrap_or_else(|| {
			table_path
				.last_segment()
				.expect("Empty path")
				.ident
				.to_string()
		});

		let name_column = name_column.unwrap_or_else(|| new_ident("name"));

		Ok(ContainerAttrs {
			table_name,
			table_path,
			name_column,
			common_attrs: CommonAttrs {
				test_runner,
				case: case.unwrap_or(Case::Snake),
				skip_test,
			},
			id_type: id_type.unwrap_or_default(),
			skip_ranges: skip_ids.unwrap_or_default(),
		})
	}
}

impl PgContainerAttrs<'_> {
	pub fn parse(
		enum_ident: &Ident,
		attrs: &[Attribute],
		is_derive: IsDerive,
	) -> syn::Result<Self> {
		let mut pg_enum_name: Option<String> = None;
		let mut test_runner: Option<TestRunner> = None;
		let mut case: Option<Case> = None;
		let mut sql_type_path: Option<Path> = None;
		let mut skip_test = false;

		for attr in attrs {
			if attr.path().is_ident("db") {
				attr.parse_nested_meta(|meta| {
          let ident_str = meta.ident_str()?;

          match ident_str.as_str() {
						"skip_test" => {
							skip_test = true;
						}
            "sql_type" => {
              sql_type_path = Some(meta.parse_value::<Path>()?);
            }
            "name" => {
              pg_enum_name = Some(meta.parse_value::<LitStr>()?.value());
            }
						"async_runner" => {
							test_runner = Some(TestRunner::Async(meta.parse_value::<Path>()?));
						}
						"sync_runner" => {
							test_runner = Some(TestRunner::Sync(meta.parse_value::<Path>()?));
						}
            "case" => {
              let case_value = match meta.parse_value::<LitStr>()?.value().as_str() {
                "snake_case" => Case::Snake,
                "UPPER_SNAKE" => Case::UpperSnake,
                "camelCase" => Case::Camel,
                "PascalCase" => Case::Pascal,
                "lowercase" => Case::Lower,
                "UPPERCASE" => Case::Upper,
                "kebab-case" => Case::Kebab,
                _ => {
                  return Err(error!(
                    meta.path,
                    "Invalid value for `case`. Allowed values are: [ snake_case, UPPER_SNAKE, camelCase, PascalCase, lowercase, UPPERCASE, kebab-case ]"
                  ));
                }
              };

              case = Some(case_value);
            }
            _ => return Err(meta.error(
              "Unknown attribute"
            ))
          };

          Ok(())
        })?;
			} else if *is_derive && sql_type_path.is_none() && attr.path().is_ident("diesel") {
				attr.parse_nested_meta(|meta| {
					if meta.path.is_ident("sql_type") {
						let path = meta.parse_value::<Path>()?;
						sql_type_path = Some(path);
					}

					drain_token_stream!(meta.input);

					Ok(())
				})?;
			}
		}

		if skip_test && test_runner.is_some() {
			bail_call_site!("Cannot use `skip_test` if a runner is specified");
		}

		if !skip_test && test_runner.is_none() {
			if let Some(default_runner) = TestRunner::default_runner(EnumKind::PgEnum) {
				test_runner = Some(default_runner);
			} else {
				bail_call_site!(
					"Missing test runner. If you want to use one of the default runners, enable it as a feature. If you want to skip the automatically generated test, use the `skip_test` attribute"
				);
			}
		}

		let sql_type_path =
			sql_type_path.unwrap_or_else(|| parse_quote!( crate::schema::sql_types::#enum_ident ));

		let pg_enum_name = pg_enum_name.unwrap_or_else(|| {
			let trailing_ident = sql_type_path
				.last_segment()
				.expect("Empty path")
				.ident
				.to_string();

			ccase!(snake, trailing_ident)
		});

		Ok(PgContainerAttrs {
			sql_type_path,
			pg_enum_name,
			common_attrs: CommonAttrs {
				test_runner,
				case: case.unwrap_or(Case::Snake),
				skip_test,
			},
		})
	}
}
