use std::{env, fs::File, io, process::ExitCode};

use judge::compare::{Comparator, TokenComparator};

fn main() -> io::Result<ExitCode> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <answer_file> <output_file>", args[0]);
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid argument count",
        ));
    }
    let mut answer_file = File::open(&args[1])?;
    let mut output_file = File::open(&args[2])?;
    let result = TokenComparator.compare(&mut answer_file, &mut output_file)?;
    match result.ok {
        true => Ok(ExitCode::SUCCESS),
        false => Ok(ExitCode::FAILURE),
    }
}
