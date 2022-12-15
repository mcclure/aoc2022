// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use glam::IVec2;

fn main() -> Result<(), Error> {
	let mut beacons: Vec<(IVec2, IVec2)> = Default::default(); 

	{
	    // Load file from command-line argument or (if none) stdin
		let filename = std::env::args().fuse().nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
			None => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, i32> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(|s|String::from_iter(s.iter().map(|s|*s as char)).parse::<i32>())
		}

		fn integer<'a>() -> Parser<'a, u8, i32> {
			(sym(b'-').opt().map(|x|x.is_none()) + positive()).map(|(n,u)|
				if n {-u} else {u})
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn next_x<'a>() -> Parser<'a, u8, ()> {
			none_of(b"x").repeat(0..) * sym(b'x') * whitespace() * sym(b'=') * whitespace()
		}

		fn separator<'a>() -> Parser<'a, u8, ()> {
			whitespace() * sym(b',') * whitespace() * sym(b'y') * whitespace() * sym(b'=') * whitespace()
		}

		fn single<'a>() -> Parser<'a, u8, IVec2> {
			next_x() * ((integer() - separator()) + integer()).map(|(x,y)|IVec2::new(x,y))
		}

		let invalid = |s:&str| { Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let parsed = (single() + single() - end()).parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok((sensor, beacon)) => {
					beacons.push((beacon, sensor-beacon));
				}
			}
		}
	}

	// Final score
	println!("{:?}", beacons);

	Ok(())
}
