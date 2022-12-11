// Calculates a "score" from a series of A, B, C, X, Y, Z letters representing a rock paper scissors game and orders to win, tie or take a dive

#![allow(unused_parens)]

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

fn main() -> Result<(), Error> {
	// Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	let invalid = || { return Error::new(ErrorKind::InvalidInput, "Expecting format '[A..C] [X..Z]'") };

	// Scan file
	for line in lines {
		// Sanitize / collect
		let line = line?.to_ascii_uppercase(); // Accept uppercase, but not unicode
		if line.is_empty() { continue } // Blank lines are allowed
		if line.len() != 3 { return Err(invalid()) }
		let mut chars = line.chars();
		let (them_ch, whitespace_ch, us_ch) = (chars.next().ok_or_else(invalid)?, chars.next().ok_or_else(invalid)?, chars.next().ok_or_else(invalid)?);
		if !whitespace_ch.is_whitespace() { return Err(invalid()) }
		let (them, us) = (them_ch as i64 - 'A' as i64,
			              us_ch   as i64 - 'X' as i64);
		for v in [us, them] {
			if (v < 0 || v > 2) { return Err(invalid()) }
		}

		let eq = us == them;
		let win = (us-them + 3)%3 == 1;
		let mut score: i64 = 0;
		score += (us + 1);
		if eq { score += 3 }
		if win { score += 6 }

		//println!("{} ({}) {} ({}) {} {} {}", them_ch, them, us_ch, us, if eq {"EQ"} else {"  "}, if win {"WIN"} else {"   "}, score);

		total += score;
	}

	// Final score
	println!("{}", total);

	Ok(())
}
