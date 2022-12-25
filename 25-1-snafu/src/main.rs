// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use clap::Parser;

#[derive(Parser)]
struct Cli {
	#[arg(short = 'r', long = "reverse")]
	reverse: bool,
	filename: Option<String>
}

fn from_snafu(s:&str) -> Option<i64> {
	None
}

fn to_snafu(i:i64) -> String {
	"".to_string()
}

fn main() -> Result<(), Error> {
	let (filename, reverse) = {
		let cli = Cli::parse();
		(cli.filename, cli.reverse)
	};

	let mut total: i64 = 0;

	{
	    let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		let invalid = |s:&str| { Error::new(ErrorKind::InvalidInput, format!("Couldn't parse line: '{}'", s)) };

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();

			let result: i64 = if reverse {
				0
			} else {
				let result = from_snafu(line).ok_or_else(||invalid(line))?;
				println!("{}", result);
				result
			};

			total += result;
		}
	}

	// Final score
	println!("\nTOTAL: {} ({})", total, to_snafu(total));

	Ok(())
}
