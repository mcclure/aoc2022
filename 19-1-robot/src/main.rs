// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use either::Either;

#[derive(Debug,Copy,Clone,PartialEq)]
enum Cell {
	Ore,
	Clay,
	Obsidian,
	Geode
}
//impl Default for Cell { fn default() -> Self { Cell::Ore } }

type Count = i32;
type Time = i32;

// Creates, (costs1, costs2)
type Robot = (Cell, ((Count,Cell),Option<(Count,Cell)>));

const TIME_LIMIT:Time = 24;

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();

	let blueprints = {
	    // Load file from command-line argument or (if -) stdin
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, Count> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<Count>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn prelude<'a>() -> Parser<'a, u8, ()> {
			none_of(b":").repeat(0..) * sym(b':').discard()
		}

		fn mineral<'a>() -> Parser<'a, u8, Cell> {
			seq(b"ore").map(|_|Cell::Ore) |
			seq(b"clay").map(|_|Cell::Clay) |
			seq(b"obsidian").map(|_|Cell::Obsidian) |
			seq(b"geode").map(|_|Cell::Geode)
		}

		fn minerals<'a>() -> Parser<'a, u8, (Count, Cell)> {
			(positive() - whitespace()) +
			mineral()
		}

		fn robot<'a>() -> Parser<'a, u8, Robot> {
			(seq(b"Each ") * mineral() - seq (b" robot costs")) +
			(whitespace() * minerals()
			 + (whitespace() * seq(b"and") * whitespace() * minerals()).opt()
			 - sym(b'.'))
		}

		fn statement<'a>() -> Parser<'a, u8, Vec<Robot>> {
			prelude() * whitespace() * list(robot(), whitespace()) - end()
		}

		let invalid = |s:&str| { return Err(Error::new(ErrorKind::InvalidInput, format!("Unrecognized line: '{}'", s))) };

		let mut blueprints: Vec<Vec<Robot>> = Default::default();

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let parsed = (statement()).parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok(x) => {
					blueprints.push(x);
				}
			}
		}
		blueprints
	};

	struct HistoryNode {
		cell:Cell,
		at:Time,
		next: HistoryNodeRef
	}
	type HistoryNodeRef = Option<Rc<RefCell<HistoryNode>>>;

	// History, time, robot count, resource count, next
	struct Consider {
		history:HistoryNodeRef,
		time:Time,
		robots:[Count;4],
		cells:[Count;4],
		want:Option<Cell>
	}

	// id, geode count
	type BlueprintRecord = (usize,Count);

	let winning_blueprint: Option<BlueprintRecord> = None;
	fn best_blueprint(a:Option<BlueprintRecord>, b:BlueprintRecord) -> Option<BlueprintRecord> {
		match a {
			None => Some(b),
			Some(a) => Some(if b.1 > a.1 { b } else { a })
		}
	}
	fn score_consider(a:&Consider) -> Count {
		a.cells[3]
	}
	fn best_consider(a:Option<Consider>, b:Consider) -> Option<Consider> {
		match a {
			None => Some(b),
			Some(a) => Some(if score_consider(&b) > score_consider(&a) { b } else { a })
		}
	}

	let mut total:i64 = 0;

	for (i,blueprint) in blueprints.iter().enumerate() {
		let winning_consider: Option<Consider> = Default::default();

		let mut next_considers = vec![Consider{history:None, time:0, robots:[1,0,0,0], cells:[0;4], want:None}];
		while !next_considers.is_empty() {
			let considers = std::mem::take(&mut next_considers);
			for mut consider in considers {
				loop {
					
				}
			}
		}

		total += ((i as i64)+1)*(score_consider(&winning_consider.unwrap()) as i64);
	}

	// Final score
	match winning_blueprint {
		None => return Err(Error::new(ErrorKind::InvalidInput, "No blueprints?")),
		Some((id, count)) =>
			println!("Best {} with {}", id+1, count)
	}


	println!("{}", total);

	Ok(())
}
