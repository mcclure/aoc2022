// Given a list of ranges, determine how many fully enclose each other.

#![allow(unused_parens)]

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

use pom::parser::*;

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
	fn pinteger<'a>() -> Parser<'a, char, i64> {
		let integer = one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
		integer.collect().convert(|s|String::from_iter(s.iter()).parse::<i64>())
	}
	fn range<'a>() -> Parser<'a, char, (i64, i64)>
		{ pinteger() - sym('-') + pinteger() }
	fn range_pair<'a>() -> Parser<'a, char, ((i64, i64), (i64, i64))>
		{ range() - sym(',') + range() }

	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting input with format [num]-[num],[num]-[num]")) };

	// Scan file
	for line in lines {
		let line = line?;
		let line_array:Vec<char> = line.chars().collect();
		let content = range_pair().parse(&line_array);
		match content {
			Ok(_t @ ((a,b),(c,d))) => {
				if b<a || d<c { return invalid() }
				let pass = a<=c && b>=d || c<=a && d>=b;
				//println!("{:?} {}", _t, pass);
				if pass { total += 1 }
			},
			_ => return invalid()
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
