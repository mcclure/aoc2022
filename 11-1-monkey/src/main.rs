// Parses a series of monkey descriptions.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

enum Op {
	Plus, Times
}

enum Operand {
	Old,
	Literal(u64)
}

struct Monkey {
	starting:Vec<i64>,
	operation:(Op, Operand),
	divisible:i64,
	ifTrue:i64,
	ifFalse:i64
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	// Filter input to remove blank lines.
	let mut lines = input.lines().filter(|x|match x { Ok(x) => !x.is_empty(), _ => true }).peekable();

	let mut total: u64 = 0;
	let mut monkeys:Vec<Monkey> = Default::default();

	{
		use pom::parser::*;

		let invalide = || { Error::new(ErrorKind::InvalidInput, "Unrecognized line") };

		fn positive<'a>() -> Parser<'a, u8, u64> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(|s|String::from_iter(s.iter().map(|s|*s as char)).parse::<u64>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn not_number<'a>() -> Parser<'a, u8, ()> {
			none_of(b"0123456789").repeat(0..).discard()
		}

		fn comma_separator<'a>() -> Parser<'a, u8, ()> {
			(whitespace() * sym(b',') * whitespace()).discard() 
		}

		fn comma_separated_positive<'a>() -> Parser<'a, u8, Vec<u64>> {
			list(positive(), comma_separator())
		}

		fn ends_with_positive<'a>() -> Parser<'a, u8, u64> { // Matches any line ending with a integer
			not_number() * positive() - whitespace()
		}

		fn ends_with_positive_list<'a>() -> Parser<'a, u8, Vec<u64>> { // Matches any line ending with
			not_number() * comma_separated_positive() - whitespace()   // a comma-separated list of ints
		}

		fn ends_with_operation<'a>() -> Parser<'a, u8, (Op, Operand)> {
			none_of(b"*+").repeat(0..) * (
				( sym(b'+').map(|_|Op::Plus) | sym(b'*').map(|_|Op::Times) ) +
				( seq(b"old").map(|_|Operand::Old) | positive().map(Operand::Literal))
			)
		}

		fn next<I, T:Iterator<Item = Result<I, Error>>>(l:&mut T) -> Result<I, Error> { match (*l).next() { Some(x) => x, None => Err(Error::new(ErrorKind::InvalidInput, "Incomplete monkey")) } }

		// Scan file
		loop {
			let _ = next(&mut lines)?; // Monkey number
			//let monkey = Monkey {
			//	starting: 
			//}
			let temp = next(&mut lines)?;
			let a = ends_with_positive().parse(temp.as_bytes()).map_err(|_|invalide())?;
			let b = next(&mut lines)?;
			let c = next(&mut lines)?;
			let d = next(&mut lines)?;
			let e = next(&mut lines)?;

			// In the actual program, I would push() a monkey struct onto an array of monkeys.
			println!("'{}', '{}', '{}', '{}', '{}'", a, b, c, d, e);

			// If EOF occurs at this known place, break cleanly.
			if let None = lines.peek() { break }
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
