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
	let invisible = {
		let height = grid.len();
		let mut seen:usize = 0;
		let mut check = |x:usize,y:usize,highest:&mut i8| {
			let seen_cell:&mut bool = &mut seen_grid[x][y];
			if *seen_cell { return true }
			let grid_cell = grid[x][y];
			if grid_cell > *highest {
				*highest = grid_cell;
				*seen_cell = true;
				seen += 1;
			}
			false
		};
		for x in 0..width {
			let mut highest:i8 = -1;
			for y in 0..height { if check(x,y,&mut highest) { break } }
			let mut highest:i8 = -1;
			for y in (0..height).rev() { if check(x,y,&mut highest) { break } }
		}
		for y in 0..grid.len() {
			let mut highest:i8 = -1;
			for x in 0..width { if check(x,y,&mut highest) { break } }
			let mut highest:i8 = -1;
			for x in (0..width).rev() { if check(x,y,&mut highest) { break } }
		}
		seen
	};

	for y in seen_grid { for x in y { print!("{}", if x {'█'} else {'.'}) } println!(""); }

	// Final score
	println!("{}", invisible);

	Ok(())
}
