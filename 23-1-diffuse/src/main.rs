// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashSet;
use either::Either;
use glam::IVec2;

// Set 0 to disable
const DEBUG_ROUND:usize = 10;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();

	let mut elves = {
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};
		let lines = input.lines();
		
		let mut elves: Vec<IVec2> = Default::default();

		for (y,line) in lines.enumerate() {
			let line = line?;
			let line = line.trim_end();
			if line.is_empty() { break } // NOT DONE

			for (x,ch) in line.chars().enumerate() {
				if ch == '#' {
					elves.push(IVec2::new(x as i32,y as i32));
				}
			}
		}

		elves
	};

	if elves.is_empty() { return Err(Error::new(ErrorKind::InvalidInput, "No elves?")) }

	for round in 0.. {
		let mut elves_map:HashSet<IVec2> = Default::default();
		let mut elves_min = IVec2::new(i32::MAX, i32::MAX);
		let mut elves_max = IVec2::new(i32::MIN, i32::MIN);

		for &elf in elves.iter() {
			elves_min = elves_min.min(elf);
			elves_max = elves_max.max(elf);
			elves_map.insert(elf);
		}

		if DEBUG_ROUND > 0 && round % DEBUG_ROUND == 0 {
			let mut empty = 0;
			for y in elves_min.y..=elves_max.y {
				for x in elves_min.x..=elves_max.x {
					print!("{}",
						if elves.contains(&IVec2::new(x,y)) {
							'#'
						} else {
							empty += 1;
							'.'
						}
					)
				}
				println!("");
			}
			println!("Score: {}\n", empty);
		}

		break
	}

	Ok(())
}
