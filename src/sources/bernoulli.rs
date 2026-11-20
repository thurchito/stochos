//! Bernoulli trial: a single experiment with outcomes success (`1`) and
//! failure (`0`), where `p + q = 1`.

use std::fmt;

use rand::Rng;

/// A Bernoulli distribution parameterised by its probability of success.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bernoulli {
    p: f64,
}

/// Error returned when a probability lies outside `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidProbability(pub f64);

impl fmt::Display for InvalidProbability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "probability must be within [0, 1], got {}", self.0)
    }
}

impl std::error::Error for InvalidProbability {}

impl Bernoulli {
    /// Creates a trial from the probability of success `p`.
    pub fn from_p(p: f64) -> Result<Self, InvalidProbability> {
        if (0.0..=1.0).contains(&p) {
            Ok(Self { p })
        } else {
            Err(InvalidProbability(p))
        }
    }

    /// Creates a trial from the probability of failure `q`.
    pub fn from_q(q: f64) -> Result<Self, InvalidProbability> {
        if (0.0..=1.0).contains(&q) {
            Ok(Self { p: 1.0 - q })
        } else {
            Err(InvalidProbability(q))
        }
    }

    /// Probability of success.
    pub fn p(&self) -> f64 {
        self.p
    }

    /// Probability of failure.
    pub fn q(&self) -> f64 {
        1.0 - self.p
    }

    /// Runs one trial, returning `1` on success and `0` on failure.
    pub fn trial<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        u8::from(rng.random_bool(self.p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn q_is_complement_of_p() {
        let b = Bernoulli::from_q(0.25).unwrap();
        assert_eq!(b.p(), 0.75);
        assert_eq!(b.q(), 0.25);
    }

    #[test]
    fn rejects_out_of_range_probabilities() {
        assert_eq!(Bernoulli::from_p(1.5), Err(InvalidProbability(1.5)));
        assert_eq!(Bernoulli::from_q(-0.1), Err(InvalidProbability(-0.1)));
        assert!(Bernoulli::from_p(f64::NAN).is_err());
    }

    #[test]
    fn degenerate_probabilities_are_deterministic() {
        let mut rng = StdRng::seed_from_u64(0);
        let always = Bernoulli::from_p(1.0).unwrap();
        let never = Bernoulli::from_p(0.0).unwrap();
        for _ in 0..100 {
            assert_eq!(always.trial(&mut rng), 1);
            assert_eq!(never.trial(&mut rng), 0);
        }
    }

    #[test]
    fn success_rate_approximates_p() {
        let mut rng = StdRng::seed_from_u64(42);
        let b = Bernoulli::from_p(0.3).unwrap();
        let n = 100_000;
        let successes: u32 = (0..n).map(|_| u32::from(b.trial(&mut rng))).sum();
        let rate = f64::from(successes) / f64::from(n);
        assert!((rate - 0.3).abs() < 0.01, "rate was {rate}");
    }
}
