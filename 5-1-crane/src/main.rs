// Simulate a crane robot based on a drawing and a series of instructions.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tok {
    Blank,
    Crate,
}


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
	let crateLine = Regex::new(r"(?:(?:^|\s)(?:\[(?P<W>\w)\]|\s{3}))+").unwrap();

	// Scan file
	for line in lines {
		let line = line?;
		println!("Line");
		for capture in crateLine.captures_iter(&line) {
			println!("{:?} {:?} {} {:?}", capture.name("W"), capture.get(0), capture.len(), capture.get(1));
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
