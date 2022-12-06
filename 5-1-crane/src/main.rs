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
	let numbers_re = Regex::new(r"^[\s\d]+$").unwrap();

	// Returns rest of string after match
	fn match_next<'a>(m:regex::Captures, s:&'a str) -> &'a str {
		return &s[m.get(0).unwrap().end()..]
	}

	// Returns first match group, rest of string after match
	fn match_next_get<'a, 'b>(m:regex::Captures<'a>, s:&'b str) -> (&'a str, &'b str) {
		return (m.get(1).unwrap().as_str(), match_next(m, s))
	}

	// Scan file
	for line in lines {
		let line = line?;
		let mut rest = line.as_str();
		println!("Line");

		// Note: Moves to phase 2 on first empty line, ignores number "comment"
		// Does NOT check accuracy of number "comment"
		if rest.is_empty() { break }
		if let Some(_) = numbers_re.captures(rest) { continue }

		let mut column = 0;
		loop {
			if column > 0 {
				if let Some(capture) = separator_re.captures(rest) {
					rest = match_next(capture, rest)
				} else {
					break // End of string
				}
			}
			if let Some(capture) = blank_re.captures(rest) {
				rest = match_next(capture, rest);
			} else if let Some(capture) = crate_re.captures(rest) {
				let tag:&str;
				(tag, rest) = match_next_get(capture, rest);
				println!("Column {} tag {}", column, tag);
			}
			column += 1
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
