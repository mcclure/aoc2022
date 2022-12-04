// Summary
use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

use pom::parser::*;
use pom::Parser;
use std::str;
use std::str::FromStr;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	// p for positive
	fn pinteger() -> Parser<u8, i64> {
		let integer = one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
		integer.collect().convert(str::from_utf8).convert(|s|i64::from_str(&s))
	}
	let range = pinteger() - sym(b'-') + pinteger();
	let range_pair = range - sym(b',') + range;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Scan file
	for line in lines {
		let line = line?;
		let content = range_pair.parse(br#"Test"#);
		println!("{:?}", content);
	}

	// Final score
	println!("{}", total);

	Ok(())
}
