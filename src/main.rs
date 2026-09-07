//! The shell: stream stdin through the simulation to stdout. Everything else
//! lives in the library and never touches the process's I/O.

use std::error::Error;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        // The reader went away, for example `| head`. Not an error.
        Err(error) if is_broken_pipe(error.as_ref()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let mut output = BufWriter::new(io::stdout().lock());
    for outcome in martian_robots::simulate(io::stdin().lock())? {
        writeln!(output, "{}", outcome?)?;
    }
    output.flush()?;
    Ok(())
}

fn is_broken_pipe(error: &(dyn Error + 'static)) -> bool {
    error
        .downcast_ref::<io::Error>()
        .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
}
