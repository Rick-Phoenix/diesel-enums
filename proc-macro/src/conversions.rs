use crate::*;

pub fn sql_conversion_fallback(enum_ident: &Ident) -> TokenStream2 {
	quote! {
		impl<DB, T> diesel::deserialize::FromSql<T, DB> for #enum_ident
		where
			DB: diesel::backend::Backend,
		{
			fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
				unimplemented!()
			}
		}

		impl<DB, T> diesel::serialize::ToSql<T, DB> for #enum_ident
		where
			DB: diesel::backend::Backend,
		{
			fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result {
				unimplemented!()
			}
		}
	}
}
