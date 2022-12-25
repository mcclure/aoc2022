// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use clap::Parser;

#[derive(Parser)]
struct Cli {
	#[arg(short = 'r', long = "reverse")]
	reverse: bool,
	#[arg(short = 's', long = "silent")]
	silent: bool,
	filename: Option<String>
}

fn from_snafu(s:&str) -> Option<i64> {
	let mut base = 1;
	let mut result = 0;
	for ch in s.chars().rev() {
		let digit = match ch {
			'=' => -2,
			'-' => -1,
			'0' => 0,
			'1' => 1,
			'2' => 2,
			_ => return None
		};
		result += digit*base;
		base *= 5;
	}
	Some(result)
}

fn to_snafu(mut i:i64) -> String {
	if i<0 { panic!("Currently don't support negative numbers ({})", i) }
	if i == 0 { return "0".to_string() }
	let mut result: Vec<char> = Default::default();
	let mut carry = false;
	while i > 0 {
		let digit = i%5;
		carry = i > 2;
		i /= 5;
		result.push(match digit {
			0 => '0',
			1 => '1',
			2 => '2',
			3 => {i -= 2; '='},
			4 => {i -= 1; '-'},
			_ => panic!("Unreachable")
		});
	}
	if carry { result.push('1') }
	result.iter().rev().collect::<String>()
}

fn main() -> Result<(), Error> {
	let (filename, reverse, silent) = {
		let cli = Cli::parse();
		(cli.filename, cli.reverse, cli.silent)
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
				let result = line.parse::<i64>().map_err(|_|invalid(line))?;
				if !silent { println!("{}", to_snafu(result)); }
				result
			} else {
				let result = from_snafu(line).ok_or_else(||invalid(line))?;
				if !silent { println!("{}", result); }
				result
			};

			total += result;
		}
	}

	// Final score
	println!("\nTOTAL: {} ({})", total, to_snafu(total));

	Ok(())
}
