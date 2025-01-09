#[macro_export]
macro_rules! id {
    ($name: ident) => {
        ::paste::paste! {
            #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::sqlx::Type, ::serde::Serialize, ::serde::Deserialize)]
            #[sqlx(transparent)]
            pub struct [<$name Id>](pub i64);

            impl ::std::fmt::Display for [<$name Id>] {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                    write!(f, "{}", self.0)
                }
            }
        }
   };
}
