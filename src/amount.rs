use std::fmt;
use std::ops::{Add, Sub};

use crate::processing_error::ProcessingError;

/// Number of fixed-point units in one whole currency unit.
pub const SCALE: i64 = 10_000;

/// A monetary amount stored as an `i64` scaled by [`SCALE`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(i64);

impl Amount {
    pub const ZERO: Amount = Amount(0);

    pub fn from_scaled(scaled: i64) -> Self {
        Amount(scaled)
    }

    pub fn scaled(self) -> i64 {
        self.0
    }

    /// Parses a non-negative decimal string with at most four fractional digits.
    pub fn parse(input: &str) -> Result<Self, ProcessingError> {
        let text = input.trim();
        if text.is_empty() {
            return Err(ProcessingError::InvalidAmount(input.to_string()));
        }
        if text.starts_with('-') {
            return Err(ProcessingError::NegativeAmount(input.to_string()));
        }
        if text.starts_with('+') {
            return Err(ProcessingError::InvalidAmount(input.to_string()));
        }

        let mut parts = text.split('.');
        let integer = parts.next().unwrap_or("");
        let fraction = parts.next();
        if parts.next().is_some() {
            return Err(ProcessingError::InvalidAmount(input.to_string()));
        }
        let fraction = fraction.unwrap_or("");

        if integer.is_empty() && fraction.is_empty() {
            return Err(ProcessingError::InvalidAmount(input.to_string()));
        }
        if !integer.chars().all(|c| c.is_ascii_digit()) {
            return Err(ProcessingError::InvalidAmount(input.to_string()));
        }
        if !fraction.chars().all(|c| c.is_ascii_digit()) {
            return Err(ProcessingError::InvalidAmount(input.to_string()));
        }
        if fraction.len() > 4 {
            return Err(ProcessingError::TooManyDecimals(input.to_string()));
        }

        let whole: i64 = if integer.is_empty() {
            0
        } else {
            integer.parse().map_err(|_| ProcessingError::Overflow)?
        };

        let mut fraction_digits = String::from(fraction);
        while fraction_digits.len() < 4 {
            fraction_digits.push('0');
        }
        let fractional: i64 = if fraction_digits.is_empty() {
            0
        } else {
            fraction_digits
                .parse()
                .map_err(|_| ProcessingError::Overflow)?
        };

        let scaled = whole
            .checked_mul(SCALE)
            .and_then(|value| value.checked_add(fractional))
            .ok_or(ProcessingError::Overflow)?;
        Ok(Amount(scaled))
    }

    pub fn checked_add(self, other: Amount) -> Option<Amount> {
        self.0.checked_add(other.0).map(Amount)
    }

    pub fn checked_sub(self, other: Amount) -> Option<Amount> {
        self.0.checked_sub(other.0).map(Amount)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        let negative = value < 0;
        let magnitude = value.unsigned_abs();
        let whole = magnitude / SCALE as u64;
        let fraction = magnitude % SCALE as u64;

        let mut digits = format!("{fraction:04}");
        while digits.len() > 1 && digits.ends_with('0') {
            digits.pop();
        }

        if negative {
            write!(f, "-{whole}.{digits}")
        } else {
            write!(f, "{whole}.{digits}")
        }
    }
}

impl Add for Amount {
    type Output = Amount;

    fn add(self, rhs: Amount) -> Amount {
        self.checked_add(rhs).expect("amount overflow")
    }
}

impl Sub for Amount {
    type Output = Amount;

    fn sub(self, rhs: Amount) -> Amount {
        self.checked_sub(rhs).expect("amount underflow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(input: &str) -> Amount {
        Amount::parse(input).expect("amount should parse")
    }

    #[test]
    fn parses_whole_amounts() {
        assert_eq!(parsed("5").scaled(), 50_000);
        assert_eq!(parsed("5").to_string(), "5.0");
    }

    #[test]
    fn parses_fractional_amounts() {
        assert_eq!(parsed("1.23").scaled(), 12_300);
        assert_eq!(parsed("1.23").to_string(), "1.23");
    }

    #[test]
    fn parses_leading_dot_fractions() {
        assert_eq!(parsed(".5").scaled(), 5_000);
        assert_eq!(parsed(".5").to_string(), "0.5");
    }

    #[test]
    fn parses_trailing_fractional_zeros() {
        assert_eq!(parsed("1.2300").scaled(), 12_300);
        assert_eq!(parsed("1.2300").to_string(), "1.23");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(parsed("  1.5  ").scaled(), 15_000);
    }

    #[test]
    fn rejects_negative_amounts() {
        assert!(matches!(
            Amount::parse("-1.0"),
            Err(ProcessingError::NegativeAmount(_))
        ));
    }

    #[test]
    fn rejects_too_many_fraction_digits() {
        assert!(matches!(
            Amount::parse("1.23456"),
            Err(ProcessingError::TooManyDecimals(_))
        ));
    }

    #[test]
    fn rejects_malformed_amounts() {
        assert!(matches!(
            Amount::parse("abc"),
            Err(ProcessingError::InvalidAmount(_))
        ));
        assert!(matches!(
            Amount::parse("1.2.3"),
            Err(ProcessingError::InvalidAmount(_))
        ));
        assert!(matches!(
            Amount::parse(""),
            Err(ProcessingError::InvalidAmount(_))
        ));
    }

    #[test]
    fn rejects_scaled_overflow() {
        assert!(matches!(
            Amount::parse("922337203685477.5808"),
            Err(ProcessingError::Overflow)
        ));
    }

    #[test]
    fn accepts_the_largest_scaled_value() {
        assert_eq!(parsed("922337203685477.5807").scaled(), i64::MAX);
    }

    #[test]
    fn renders_zero_with_one_decimal_place() {
        assert_eq!(Amount::ZERO.to_string(), "0.0");
    }

    #[test]
    fn renders_one_unit_fractions() {
        assert_eq!(parsed("0.0001").to_string(), "0.0001");
    }

    #[test]
    fn renders_negative_amounts_with_a_sign() {
        let zero = Amount::ZERO;
        let value = zero.checked_sub(parsed("10.0")).expect("subtraction fits");
        assert_eq!(value.to_string(), "-10.0");
    }

    #[test]
    fn checked_add_reports_overflow() {
        assert!(Amount::from_scaled(i64::MAX)
            .checked_add(Amount::from_scaled(1))
            .is_none());
    }

    #[test]
    fn checked_sub_reports_underflow() {
        assert!(Amount::from_scaled(i64::MIN)
            .checked_sub(Amount::from_scaled(1))
            .is_none());
    }
}
