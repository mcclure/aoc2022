// Finds "invisible" cells in a height map

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
	let mut seen_grid: Vec<Vec<bool>>;

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
			Some(width) => {
				seen_grid = vec![vec![false; width]; grid.len()];
				width
			}
		}
	};

	// Check visibility.
	// Note 0 does NOT mean "no tree". It means a min-height tree.
	let height = grid.len();
	{
		let mut check = |x:usize,y:usize,idx:usize,highest:&mut i8,pass_seen:&mut Vec<bool>| {
			let pass_seen_cell:&mut bool = &mut pass_seen[idx];
			if *pass_seen_cell { return true } // Met ourselves from other side, don't process further.

			let seen_cell:&mut bool = &mut seen_grid[x][y];
			let grid_cell = grid[x][y];
			if grid_cell > *highest {
				*highest = grid_cell;
				*pass_seen_cell = true;
				if !*seen_cell {
					*seen_cell = true;
				}
			}
			false
		};
		for x in 0..width {
			let mut pass_seen = vec![false; width];
			let mut highest:i8 = -1;
			for y in 0..height { if check(x,y,y,&mut highest,&mut pass_seen) { break } }
			let mut highest:i8 = -1;
			for y in (0..height).rev() { if check(x,y,y,&mut highest,&mut pass_seen) { break } }
		}
		for y in 0..height {
			let mut pass_seen = vec![false; height];
			let mut highest:i8 = -1;
			for x in 0..width { if check(x,y,x,&mut highest,&mut pass_seen) { break } }
			let mut highest:i8 = -1;
			for x in (0..width).rev() { if check(x,y,x,&mut highest,&mut pass_seen) { break } }
		}
	};

	// This is sideways :/
	for y in &seen_grid { for x in y { print!("{}", if *x {'█'} else {'.'}) } println!(""); }

	let mut best = 0;

	for (x, col) in seen_grid.iter().enumerate() {
		for (y, skip) in col.iter().enumerate() {
			if *skip { continue }
			let ceiling = grid[x][y];
			let mut score = 1;

			let check = |x2:usize,y2:usize,highest:&mut i8,count:&mut usize| {
				let against = grid[x2][y2];
				if against > *highest { 
					*count += 1;
					if against >= ceiling { return true }
				}
				return false 
			};

			println!("Position {}, {}", x, y);

			score *= {
				let mut highest = -1;
				let mut count:usize = 0; // Don't have to worry about edges because these are never viable
				for y2 in (y+1)..height { if check(x,y2,&mut highest,&mut count) { break } }
				println!("\tDown: {}", count);
				count
			};
			score *= {
				let mut highest = -1;
				let mut count:usize = 0;
				for y2 in (0..y).rev() { if check(x,y2,&mut highest,&mut count) { break } }
				println!("\tUp: {}", count);
				count
			};
			score *= {
				let mut highest = -1;
				let mut count:usize = 0;
				for x2 in (x+1)..width { if check(x2,y,&mut highest,&mut count) { break } }
				println!("\tRight: {}", count);
				count
			};
			score *= {
				let mut highest = -1;
				let mut count:usize = 0;
				for x2 in (0..x).rev() { if check(x2,y,&mut highest,&mut count) { break } }
				println!("\tLeft: {}", count);
				count
			};

			println!("\tScore: {}{}", score, if score>best {" (new best)"} else {""});

			if score>best { best = score }
		}
	}


	// Final score
	println!("{}", best);

	Ok(())
}
