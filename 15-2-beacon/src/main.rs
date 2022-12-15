// Recommend running in release mode

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::ops::RangeInclusive;
use either::Either;
use glam::IVec2;
use std::collections::HashSet;
use range_set::RangeSet;

const DEBUG_VERBOSE:bool = false;
const DEBUG_PROGRESS:bool = false;

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();
	let filename = args.nth(1);
	let cap = {
		let invalid_target = || { Error::new(ErrorKind::InvalidInput, "Argument 2 must be number") };
		match args.next() {
			None => return Err(invalid_target()),
			Some(x) => x.parse::<i32>().map_err(|_|invalid_target())
		}?};

	let mut sensors: Vec<(IVec2, i32)> = Default::default(); 
	let mut beacons: HashSet<IVec2> = Default::default();

	fn manhattan(v:IVec2) -> i32 { v.x.abs() + v.y.abs() }

	{
	    // Load file from command-line argument or (if none) stdin
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
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
					if DEBUG_VERBOSE {
						println!("sensor {}, beacon {} diff {}", sensor, beacon, sensor-beacon);
					}
					beacons.insert(beacon);
					sensors.push((sensor, manhattan(sensor-beacon)));
				}
			}
		}
	}

	let cap_range:RangeInclusive<i32> = 0..=cap;
	let mut cap_range_set: RangeSet<[RangeInclusive<i32>;20]> = RangeSet::new();
	cap_range_set.insert_range(cap_range.clone());
	for target_y in cap_range.clone() {
		let mut excluded = { // Scan target line
			let mut excluded: RangeSet<[RangeInclusive <i32>; 20]> = RangeSet::new();

			for (sensor, strength) in sensors.iter() {
				let depth = (target_y - sensor.y).abs();
				let align = sensor.x;
				let span = strength - depth;
				if span < 0 { continue }
				else {
					let range = (align-span)..=(align+span);
					excluded.insert_range(range);
				}
			}

			excluded
		};

		if let Some(intersect) = excluded.insert_range(cap_range.clone()) {
			if intersect != cap_range_set {
				println!("y={}: {:?}", target_y, intersect);
			} else {
				if DEBUG_PROGRESS && target_y%100000 == 0 {
					println!("...{}...", target_y);
				}
			}
		} else {
			println!("???: y={}", target_y);
		}
	}

	Ok(())
}
