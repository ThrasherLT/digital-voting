//! This module contains utility code used throughout the project.

// TODO Maybe think of better naming.
// Note: If you're getting errors in this macro, the error probably originates in one of the
// places where this macro is being actually used.
#[macro_export]
macro_rules! impl_key_display {
    ($t:ty) => {
        impl AsRef<[u8]> for $t {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use base64::engine::Engine;

                write!(
                    f,
                    "{}",
                    base64::engine::general_purpose::STANDARD.encode(&self.0)
                )
            }
        }

        impl std::str::FromStr for $t {
            type Err = Error;

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                use base64::engine::Engine;

                let bytes = base64::engine::general_purpose::STANDARD.decode(s.as_bytes())?;
                Ok(Self(bytes))
            }
        }

        impl std::ops::Deref for $t {
            type Target = Vec<u8>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $t {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}