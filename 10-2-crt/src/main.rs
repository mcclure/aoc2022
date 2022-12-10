// Timing emulator for a simple CPU

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

const DEBUG_GRID:bool = false;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let invalide = || { Error::new(ErrorKind::InvalidInput, "Invalid number argument") };
	let invalid2 = || { Err(Error::new(ErrorKind::InvalidInput, "Unrecognized command")) };

	let mut reg: i64 = 1;
	let mut cycle:i64 = 0;
	let mut advance = |x:i64, reg:&i64| {
		let mut x = x;
		while x>0 {
			{
				if cycle>1 && cycle%40==0 { println!(""); }
				print!("{}",
					if ((cycle%40)-*reg).abs() <=1 {'#'}
					else { if DEBUG_GRID {'.'} else {' '} });
			}
			x -= 1;
			cycle += 1;
		}
	};

	// Scan file
	for line in lines {
		let line = line?;
		if line.is_empty() {continue}
		let mut tokens = line.split_whitespace().fuse();
		let keyword = tokens.next().unwrap();
		match keyword {
			"noop" => { advance(1,&reg); }
			"addx" => {
				let x = tokens.next().ok_or_else(invalide)?.parse::<i64>().map_err(|_|invalide())?;
				advance(2,&reg);
				reg += x;
			}
			_ => {
				return invalid2()
			}
		}
	}

	println!("");

	Ok(())
}
