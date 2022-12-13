// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

#[derive(Debug)]
enum Node {
	Num(u64),
	List(Vec<Node>)
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let mut lines = input.lines();

	let mut total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Scan file
	{
		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, u64> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(|s|String::from_iter(s.iter().map(|s|*s as char)).parse::<u64>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn comma_separator<'a>() -> Parser<'a, u8, ()> {
			(whitespace() * sym(b',') * whitespace()).discard() 
		}

		fn comma_separated_list<'a>() -> Parser<'a, u8, Node> {
			sym(b'[') * whitespace() * (
				list(
					call(comma_separated_list) |
					positive().map(|s|Node::Num(s))
				, comma_separator()).map(|s|Node::List(s))
			) - whitespace() - sym(b']')
		}

		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			println!("{}", line);
			let parsed = (comma_separated_list() - end()).parse(line.as_bytes());
			println!("{:?}", parsed);
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
