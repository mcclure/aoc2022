// Take list containing chunks of numbers separated by newlines. Return sum of top chunk.

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

	let mut lines = input.lines();

	let mut current = 0;
	let mut best = 0;

	// Scan file
	loop {
		let mut chunk_finished = || {
			// println!("Chunk finished; {} > {}", current, best);
			if current>best {
				best = current
			}
			current = 0;
		};

		let line = match lines.next() {
			None => { chunk_finished(); break },
			Some(x) => x?
		};
		if line.is_empty() {
			chunk_finished();
		} else {
			let calories = line.parse::<i64>();
			match calories {
				Ok(x) => { current += x; },
				_ => return Err(Error::new(ErrorKind::InvalidInput, "Non-numeric input"))
			}
		}
	}

	// Final score
	println!("{}", best);

	Ok(())
}
