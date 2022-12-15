// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;
use glam::IVec2;

#[derive(Copy,Clone)]
enum Cell {
	Wall
}

fn main() -> Result<(), Error> {
	let origin:IVec2 = IVec2::new(500,0);
	let mut min:IVec2 = origin;
	let mut max:IVec2 = origin;
	let mut board: HashMap<IVec2, Cell> = Default::default();
	fn add(board:&mut HashMap<IVec2, Cell>, min:&mut IVec2, max:&mut IVec2, v:IVec2, c:Cell) {
		*min = min.min(v);
		*max = max.max(v);
		board.insert(v, c);
	}

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
				if n {u} else {-u}) // n for None
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn comma<'a>() -> Parser<'a, u8, ()> {
			whitespace() * sym(b',') * whitespace()
		}

		fn arrow<'a>() -> Parser<'a, u8, ()> {
			whitespace() * seq(b"->") * whitespace()
		}

		fn pair<'a>() -> Parser<'a, u8, IVec2> {
			((integer() - comma()) + integer()).map(|(x,y)|IVec2::new(x,y))
		}

		fn sequence<'a>() -> Parser<'a, u8, Vec<IVec2>> {
			list(pair(), arrow())
		}

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let invalid = |s:&str| { Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };
			let parsed = (sequence() - end()).parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok(x) => {
					for v in x.windows(2) {
						let (a,b) = (v[0], v[1]);
						if !(a.x == b.x || a.y == b.y) {
							return Err(Error::new(ErrorKind::InvalidInput, format!("Can't go diagonal ({} -> {})", a, b)));
						}

						let step = if a.x > b.x { IVec2::new(-1,0) }
						      else if a.y > b.y	{ IVec2::new(0,-1) }
						      else if a.x < b.x { IVec2::new(1, 0) }
						      else if a.y < b.y { IVec2::new(0, 1) }
						      else { panic!("Unreachable") };

						let mut at = a;
						while at != b {
							add(&mut board, &mut min, &mut max, at, Cell::Wall);
							at += step;
						} 
					}
				}
			}
		}
	}

	fn board_debug(board:&HashMap<IVec2, Cell>, min:&IVec2, max:&IVec2, origin:IVec2) {
		println!("{} ... {}", min, max);
		for y in min.y..=max.y {
			for x in min.x..=max.x {
				let at = IVec2::new(x,y);
				print!("{}",
					if at == origin { "+" }
					else if let Some(c) = board.get(&at) {
						match c {
							Cell::Wall => "█",
						}
					} else { "·" }
				)
			}
			println!("");
		}
		println!("");
	}

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	board_debug(&board, &min, &max, origin);

	// Final score
	//println!("{}", total);

	Ok(())
}
