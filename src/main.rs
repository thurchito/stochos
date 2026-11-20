use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();
    let stderr = io::stderr().lock();

    match stochos::repl::run(stdin, stdout, stderr, &mut rand::rng()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("stochos: {e}");
            ExitCode::FAILURE
        }
    }
}
