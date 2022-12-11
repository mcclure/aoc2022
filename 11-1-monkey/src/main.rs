// Parses a series of monkey descriptions.

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

	let mut lines = input.lines().filter(|x|match x { Ok(x) => !x.is_empty(), _ => true }).peekable();

	let mut total: i64 = 0;

	{
		use pom::parser::*;

		fn next<I, T:Iterator<Item = Result<I, Error>>>(l:&mut T) -> Result<I, Error> { match (*l).next() { Some(x) => x, None => Err(Error::new(ErrorKind::InvalidInput, "Incomplete monkey")) } }

		fn positive<'a>() -> Parser<'a, char, u64> {
			let integer = one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
			integer.collect().convert(|s|String::from_iter(s.iter()).parse::<u64>())
		}

		// Scan file
		loop {
			let line = next(&mut lines)?;
			// Do stuff to "line"
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
