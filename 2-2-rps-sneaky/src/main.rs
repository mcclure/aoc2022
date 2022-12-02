// Calculates a "score" from a series of A, B, C, X, Y, Z letters representing a rock paper scissors game

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
		let (them_ch, whitespace_ch, result_ch) = (chars.next().ok_or_else(invalid)?, chars.next().ok_or_else(invalid)?, chars.next().ok_or_else(invalid)?);
		if !whitespace_ch.is_whitespace() { return Err(invalid()) }
		let them = them_ch as i64 - 'A' as i64;
		if them < 0 || them > 2 { return Err(invalid()) }

		let us = |margin:i64| { (them + margin + 3)%3 + 1 };

		let score = match result_ch {
			'X' => us(-1),    // Lose
			'Y' => us(0) + 3, // Draw
			'Z' => us(1) + 6, // Win
			_ => return Err(invalid())
		};

		//println!("{} ({}) {} {} {}", them_ch, them, if result_ch=='Y' {"EQ"} else {"  "}, if result_ch=='Z' {"WIN"} else {"   "}, score);

		total += score;
	}

	// Final score
	println!("{}", total);

	Ok(())
}
