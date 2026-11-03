//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Reduced rationals for frame rates and speeds (docs/03 §3.3.2):
//! always reduced, `den > 0`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A rational number `{ num, den }` with `den > 0`, always reduced (03 §3.3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct Rational {
    /// Numerator.
    pub num: u32,
    /// Denominator; strictly positive.
    pub den: u32,
}

impl Rational {
    /// Construct a reduced rational; `None` when `den == 0` (03 §3.3.2).
    pub fn new(num: u32, den: u32) -> Option<Self> {
        if den == 0 {
            return None;
        }
        let g = gcd(num, den);
        Some(Self {
            num: num / g,
            den: den / g,
        })
    }

    /// The rational as an f64 approximation (display only; math stays exact).
    pub fn as_f64(self) -> f64 {
        f64::from(self.num) / f64::from(self.den)
    }
}

/// Greatest common divisor (Euclid; gcd(0, n) = n so 0/n reduces to 0/1).
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn reduces_examples() {
        let r = Rational::new(30000, 1001).unwrap_or(Rational { num: 0, den: 1 });
        assert_eq!((r.num, r.den), (30000, 1001)); // already coprime
        let r = Rational::new(60000, 2002).unwrap_or(Rational { num: 0, den: 1 });
        assert_eq!((r.num, r.den), (30000, 1001)); // reduced
        let r = Rational::new(0, 5).unwrap_or(Rational { num: 1, den: 1 });
        assert_eq!((r.num, r.den), (0, 1));
        let r = Rational::new(10, 1).unwrap_or(Rational { num: 0, den: 1 });
        assert_eq!((r.num, r.den), (10, 1));
    }

    #[test]
    fn zero_denominator_is_rejected() {
        assert!(Rational::new(1, 0).is_none());
        assert!(Rational::new(0, 0).is_none());
    }

    proptest! {
        #[test]
        fn rationals_are_reduced_with_positive_denominator(num in 0u32.., den in 1u32..) {
            // den >= 1 by the strategy; Rational::new then always succeeds.
            let Some(r) = Rational::new(num, den) else {
                unreachable!("strategy guarantees den >= 1")
            };
            prop_assert!(r.den > 0);
            prop_assert_eq!(gcd(r.num, r.den), 1);
            // Exact rational equivalence (no float comparisons):
            // r.num / r.den == num / den.
            prop_assert_eq!(
                u64::from(r.num) * u64::from(den),
                u64::from(num) * u64::from(r.den)
            );
        }
    }
}
