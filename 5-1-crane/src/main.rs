// Simulate a crane robot based on a drawing and a series of instructions.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

use regex::Regex;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Series of either three spaces or [W], separated by spaces. Will capture W or S (for Word or Space)
	let separator_re = Regex::new(r"^\p{gc:Zs}").unwrap();
	let blank_re = Regex::new(r"^\p{gc:Zs}{3}").unwrap();
	let crate_re = Regex::new(r"^\[(\w)\]").unwrap();

	fn match_next<'a>(s:&'a str, m:regex::Captures) -> &'a str {
		return &s[m.get(0).unwrap().end()..]
	}

	fn match_next_get<'a, 'b>(s:&'a str, m:regex::Captures<'b>) -> (&'a str, &'b str) {
		return (&s[m.get(0).unwrap().end()..], m.get(1).unwrap().as_str())
	}

	// Scan file
	for line in lines {
		let line = line?;
		let rest = line.as_str();
		println!("Line");
		if let Some(capture) = crate_re.captures(rest) {
			let (a,b) = match_next_get(rest, capture);
			println!("{} | {}", b,a);
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
