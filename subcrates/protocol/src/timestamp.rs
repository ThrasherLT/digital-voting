//! Module for timestamp type and related operations.

use thiserror::Error;

/// Errors that can occur when working with election votes.
#[derive(Error, Debug)]
pub enum Error {
    /// The timestamp lower limit cannot be larger than the upper limit.
    #[error("Timestamp lower limit cannot be larger than upper limit")]
    InvalidLimits,
}
type Result<T> = std::result::Result<T, Error>;

/// The type of the timestamp that the protocol will use.
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Helper struct for cleaner timestamp verification code.
pub struct Limits {
    /// Lower limit for an acceptable timestamp.
    timestamp_lower_limit: Timestamp,
    /// Upper limit for an acceptable timestamp.
    timestamp_upper_limit: Timestamp,
}

impl Limits {
    /// Create a new instance of timestamp limits for verification.
    /// Note that limits are inclusive.
    ///
    /// # Arguments
    ///
    /// `timestamp_lower_limits` - The lower acceptable limit for the timestamp to be verified.
    /// `timestamp_upper_limits` - The upper acceptable limit for the timestamp to be verified.
    ///
    /// # Errors
    ///
    /// If lower limit is larger than upper limit.
    pub fn new(timestamp_lower_limit: Timestamp, timestamp_upper_limit: Timestamp) -> Result<Self> {
        if timestamp_lower_limit > timestamp_upper_limit {
            return Err(Error::InvalidLimits);
        }

        Ok(Self {
            timestamp_lower_limit,
            timestamp_upper_limit,
        })
    }

    /// Verify the input timestamp against the upper and lower limits.
    /// Note that limits are inclusive.
    ///
    /// # Arguments
    ///
    /// `timestamp` - The timestamp to be verified.
    ///
    /// # Returns
    ///
    /// `true`, if the timestmap is valid and `false` if it is not.
    #[must_use]
    pub fn verify(&self, timestamp: Timestamp) -> bool {
        timestamp >= self.timestamp_lower_limit && timestamp <= self.timestamp_upper_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_timestamp_limits() {
        let timestamp = chrono::Utc::now();

        let timestamp_limits = Limits::new(timestamp, timestamp).unwrap();

        assert!(timestamp_limits.verify(timestamp));
        assert!(!timestamp_limits.verify(timestamp + std::time::Duration::from_nanos(1)));
        assert!(!timestamp_limits.verify(timestamp - std::time::Duration::from_nanos(1)));

        assert!(Limits::new(timestamp + std::time::Duration::from_nanos(1), timestamp).is_err());
    }
}
