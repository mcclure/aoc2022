// Calculates a "score" from letters duplicated between halves of a string.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use std::collections::HashSet;

fn main() -> Result<(), Error> {
	// Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	// Will not enforce even criteria
	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting a string of ASCII letters")) };

	let mut last_set: HashSet<u8> = HashSet::with_capacity(26*2);

	// Scan file
	for (line_idx, line) in lines.enumerate() {
		let line = line?;
		if line.is_empty() { continue; } // Assume blank lines are entry errors and skip
		if !line.is_ascii() { return invalid() }
		let mut this_set: HashSet<u8> = HashSet::with_capacity(26*2);
		let first_set = line_idx%3 == 0;
		let final_set = line_idx%3 == 2;
		let letters = line.as_bytes();

		// Construct this set
		for letter in letters {
			if first_set || last_set.contains(letter) {
				this_set.insert(*letter);
			}
		}

		// Manage last set
		if !final_set {
			last_set = this_set;
		} else {
			if this_set.len() > 1 {
				return Err(Error::new(ErrorKind::InvalidInput, "Found group with multiple duplicate letters"));
			}
			let letter = match this_set.iter().next() {
				None => return Err(Error::new(ErrorKind::InvalidInput, "Found group with no duplicate letters")),
				Some(x) => *x
			};
			//println!("Line {} common: {}", line_idx, letter as char);
			let score;
			if letter >= ('a' as u8) && letter <= ('z' as u8) {
				score = letter-('a' as u8) + 1;
			} else if letter >= ('A' as u8) && letter <= ('Z' as u8) {
				score = letter-('A' as u8) + 1 + 26;
			} else {
				return invalid(); // Unnecessary
			}
			total += score as i64;
			last_set.clear();
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
