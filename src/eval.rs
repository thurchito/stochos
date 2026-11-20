//! Evaluation of parsed calls against the available stochastic sources.

use std::fmt;

use rand::Rng;

use crate::parser::{Arg, Call, ParseError, parse};
use crate::sources::Bernoulli;
use crate::sources::bernoulli::InvalidProbability;

/// The result of evaluating a call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{n}"),
        }
    }
}

/// An error raised while parsing or evaluating input.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    Parse(ParseError),
    UnknownFunction(String),
    UnknownArgument { function: &'static str, arg: String },
    DuplicateArgument(String),
    ConflictingArguments(&'static str, &'static str),
    MissingArgument(&'static str),
    InvalidProbability(InvalidProbability),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::UnknownFunction(name) => write!(f, "unknown function `{name}`"),
            Self::UnknownArgument { function, arg } => {
                write!(f, "`{function}` has no argument `{arg}`")
            }
            Self::DuplicateArgument(arg) => write!(f, "argument `{arg}` given more than once"),
            Self::ConflictingArguments(a, b) => {
                write!(f, "`{a}` and `{b}` cannot both be given")
            }
            Self::MissingArgument(what) => write!(f, "missing argument: {what}"),
            Self::InvalidProbability(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for EvalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(e) => Some(e),
            Self::InvalidProbability(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ParseError> for EvalError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<InvalidProbability> for EvalError {
    fn from(e: InvalidProbability) -> Self {
        Self::InvalidProbability(e)
    }
}

/// Parses and evaluates a single line of input.
pub fn eval_line<R: Rng + ?Sized>(input: &str, rng: &mut R) -> Result<Value, EvalError> {
    eval(&parse(input)?, rng)
}

/// Evaluates a parsed call.
pub fn eval<R: Rng + ?Sized>(call: &Call, rng: &mut R) -> Result<Value, EvalError> {
    match call.name.as_str() {
        "bi_trial" => bi_trial(&call.args, rng),
        other => Err(EvalError::UnknownFunction(other.to_owned())),
    }
}

/// `bi_trial(p = ...)` or `bi_trial(q = ...)`.
fn bi_trial<R: Rng + ?Sized>(args: &[Arg], rng: &mut R) -> Result<Value, EvalError> {
    const NAME: &str = "bi_trial";

    let mut p = None;
    let mut q = None;
    for arg in args {
        let slot = match arg.name.as_str() {
            "p" => &mut p,
            "q" => &mut q,
            other => {
                return Err(EvalError::UnknownArgument {
                    function: NAME,
                    arg: other.to_owned(),
                });
            }
        };
        if slot.replace(arg.value).is_some() {
            return Err(EvalError::DuplicateArgument(arg.name.clone()));
        }
    }

    let bernoulli = match (p, q) {
        (Some(p), None) => Bernoulli::from_p(p)?,
        (None, Some(q)) => Bernoulli::from_q(q)?,
        (Some(_), Some(_)) => return Err(EvalError::ConflictingArguments("p", "q")),
        (None, None) => return Err(EvalError::MissingArgument("`p` or `q`")),
    };

    Ok(Value::Int(i64::from(bernoulli.trial(rng))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn run(input: &str) -> Result<Value, EvalError> {
        eval_line(input, &mut StdRng::seed_from_u64(7))
    }

    #[test]
    fn bi_trial_returns_zero_or_one() {
        let mut rng = StdRng::seed_from_u64(1);
        for input in ["bi_trial(p = 0.5)", "bi_trial(q=1/3)"] {
            for _ in 0..50 {
                let value = eval_line(input, &mut rng).unwrap();
                assert!(matches!(value, Value::Int(0 | 1)));
            }
        }
    }

    #[test]
    fn bi_trial_degenerate_cases() {
        assert_eq!(run("bi_trial(p=1)"), Ok(Value::Int(1)));
        assert_eq!(run("bi_trial(p=0)"), Ok(Value::Int(0)));
        assert_eq!(run("bi_trial(q=1)"), Ok(Value::Int(0)));
        assert_eq!(run("bi_trial(q=0)"), Ok(Value::Int(1)));
    }

    #[test]
    fn bi_trial_argument_errors() {
        assert_eq!(
            run("bi_trial(p=0.5, q=0.5)"),
            Err(EvalError::ConflictingArguments("p", "q"))
        );
        assert_eq!(
            run("bi_trial()"),
            Err(EvalError::MissingArgument("`p` or `q`"))
        );
        assert_eq!(
            run("bi_trial(p=0.1, p=0.2)"),
            Err(EvalError::DuplicateArgument("p".to_owned()))
        );
        assert!(matches!(
            run("bi_trial(r=0.5)"),
            Err(EvalError::UnknownArgument { .. })
        ));
        assert!(matches!(
            run("bi_trial(p=2)"),
            Err(EvalError::InvalidProbability(_))
        ));
    }

    #[test]
    fn unknown_function() {
        assert_eq!(
            run("foo(p=0.5)"),
            Err(EvalError::UnknownFunction("foo".to_owned()))
        );
    }
}
