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

	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting an even-length string of ASCII letters")) };

	// Scan file
	for line in lines {
		let line = line?;
		let len = line.len();
		if !line.is_ascii() || len%2 != 0 { return invalid() }
		let letters = line.as_bytes();
		let mut left_set: HashSet<u8> = HashSet::with_capacity(26*2);
		for (index,letter) in letters.iter().enumerate() {
			let letter = *letter;
			if index < len/2 {
				if !(letter as char).is_alphabetic() { return invalid() }
				left_set.insert(letter);
			} else {
				if left_set.contains(&letter) {
//					println!("Line {} collide: {}", index, letter);
					let score;
					if letter >= ('a' as u8) && letter <= ('z' as u8) {
						score = letter-('a' as u8) + 1;
					} else if letter >= ('A' as u8) && letter <= ('Z' as u8) {
						score = letter-('A' as u8) + 1 + 26;
					} else {
						return invalid();
					}
					left_set.remove(&letter);
					total += score as i64;
				}
			}
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
