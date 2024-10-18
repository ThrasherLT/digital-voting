//! This module contains utility code used throughout the project.

pub mod byte_ops;

// TODO Maybe think of better naming.
#[macro_export]
macro_rules! impl_key_display {
    ($t:ty) => {
        // Implement AsRef<[u8]>
        impl AsRef<[u8]> for $t {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        // Implement Display
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use base64::engine::Engine;

                    write!(f, "{}", base64::engine::general_purpose::STANDARD.encode(&self.0))
            }
        }

        // Implement FromStr
        impl std::str::FromStr for $t {
            type Err = Error;

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                use base64::engine::Engine;

                let bytes = base64::engine::general_purpose::STANDARD.decode(s.as_bytes())?;
                Ok(Self(bytes))
            }
        }
    };
}
