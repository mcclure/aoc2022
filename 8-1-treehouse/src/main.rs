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

	let mut grid: Vec<Vec<u8>> = Default::default();

	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Blank lines?")) };
	let invalide2 = || { return Error::new(ErrorKind::InvalidInput, "Invalid characters") };
	let invalid3 = || { return Err(Error::new(ErrorKind::InvalidInput, "Unequal lines")) };

	// Parse file
	{
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
			grid.push(Default::default());

			let row = line.chars().map(
					|x|x.to_digit(10).map(|x|x as u8).ok_or_else(invalide2)
				).collect::<Result<Vec<u8>, Error>>()?;

			match width { None => width = Some(row.len()),
				Some(x) => if x != row.len() { return invalid3() }}

			grid.push(row);
		}
	}

	let mut total: i64 = 0;

	// Final score
	println!("{}", total);

	Ok(())
}
