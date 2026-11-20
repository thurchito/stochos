//! The interactive read-eval-print loop.

use std::io::{self, BufRead, Write};

use rand::Rng;

use crate::eval::eval_line;

/// The prompt printed before each line of input.
pub const PROMPT: &str = "> ";

/// Runs the REPL until end of input or an `exit`/`quit` command.
///
/// Results are written to `output`; evaluation errors are written to `errors`
/// so that a failing line does not end the session.
pub fn run<I, O, E, R>(input: I, mut output: O, mut errors: E, rng: &mut R) -> io::Result<()>
where
    I: BufRead,
    O: Write,
    E: Write,
    R: Rng + ?Sized,
{
    let mut lines = input.lines();
    loop {
        write!(output, "{PROMPT}")?;
        output.flush()?;

        let Some(line) = lines.next().transpose()? else {
            // End of input (e.g. Ctrl-D): leave the cursor on a fresh line.
            writeln!(output)?;
            return Ok(());
        };

        match line.trim() {
            "" => continue,
            "exit" | "quit" => return Ok(()),
            line => match eval_line(line, rng) {
                Ok(value) => writeln!(output, "{value}")?,
                Err(e) => writeln!(errors, "error: {e}")?,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn session(input: &str) -> (String, String) {
        let mut out = Vec::new();
        let mut err = Vec::new();
        run(
            input.as_bytes(),
            &mut out,
            &mut err,
            &mut StdRng::seed_from_u64(0),
        )
        .unwrap();
        (
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn prints_results_after_prompts() {
        let (out, err) = session("bi_trial(p=1)\n\nbi_trial(q=1)\n");
        assert_eq!(out, "> 1\n> > 0\n> \n");
        assert!(err.is_empty());
    }

    #[test]
    fn reports_errors_and_continues() {
        let (out, err) = session("nope()\nbi_trial(p=1)\nexit\nbi_trial(p=1)\n");
        assert_eq!(out, "> > 1\n> ");
        assert_eq!(err, "error: unknown function `nope`\n");
    }
}
