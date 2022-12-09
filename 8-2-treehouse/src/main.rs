// Counts number of spaces visible from other spaces in a height map

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

	let lines = input.lines();

	let mut grid: Vec<Vec<i8>> = Default::default();

	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Blank lines?")) };
	let invalide2 = || { return Error::new(ErrorKind::InvalidInput, "Invalid characters") };
	let invalid3 = || { return Err(Error::new(ErrorKind::InvalidInput, "Unequal lines")) };
	let invalid4 = || { return Err(Error::new(ErrorKind::InvalidInput, "Empty input")) };

	// Parse file
	let width = {
		let mut blank_state = 0; // 0,1,2
		let mut width: Option<usize> = None;
		for line in lines {
			let line = line?;
			let line = line.trim_end();
			if line.is_empty() {
				if blank_state == 1 { blank_state = 2 }
				continue;
			} else {
				match blank_state { 0 => blank_state = 1, 2 => return invalid(), _=>() }
			}

			let row = line.chars().map(
					|x|x.to_digit(10).map(|x|x as i8).ok_or_else(invalide2)
				).collect::<Result<Vec<i8>, Error>>()?;

			match width { None => width = Some(row.len()),
				Some(x) => if x != row.len() { return invalid3() }}

			grid.push(row);
		}

		match width {
			None => return invalid4(),
			Some(width) => { width }
		}
	};

	//for y in &grid { for x in y { print!("{}", *x) } println!(""); }

	// Check visibility.
	// Note 0 does NOT mean "no tree". It means a min-height tree.
	let height = grid.len();

	let mut best = 0;

	for (y, col) in grid.iter().enumerate() {
		for (x, ceiling) in col.iter().enumerate() {
			let mut score = 1;
			let ceiling = *ceiling;

			let check = |x2:usize,y2:usize,highest:&mut i8,count:&mut usize| {
				let against = grid[y2][x2];
				if against > *highest {
					*count += 1;
					//println!("\t\t{}, {}: Against {} ceiling {}", x2, y2, against, ceiling);
					if against >= ceiling { return true }
				}
				return false 
			};

			//println!("Position {}, {}", x, y);

			score *= {
				let mut highest = -1;
				let mut count:usize = 0; // Don't have to worry about edges because these are never viable
				for y2 in (y+1)..height { if check(x,y2,&mut highest,&mut count) { break } }
				//println!("\tDown: {}", count);
				count
			};
			score *= {
				let mut highest = -1;
				let mut count:usize = 0;
				for y2 in (0..y).rev() { if check(x,y2,&mut highest,&mut count) { break } }
				//println!("\tUp: {}", count);
				count
			};
			score *= {
				let mut highest = -1;
				let mut count:usize = 0;
				for x2 in (x+1)..width { if check(x2,y,&mut highest,&mut count) { break } }
				//println!("\tRight: {}", count);
				count
			};
			score *= {
				let mut highest = -1;
				let mut count:usize = 0;
				for x2 in (0..x).rev() { if check(x2,y,&mut highest,&mut count) { break } }
				//println!("\tLeft: {}", count);
				count
			};

			//println!("\tScore: {}{}", score, if score>best {" (new best)"} else {""});

			if score>best { best = score }
		}
	}


	// Final score
	println!("{}", best);

	Ok(())
}
