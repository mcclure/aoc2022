// Take list containing chunks of numbers separated by newlines. Return sum of sums of top 3 chunks.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use std::collections::BinaryHeap;

const SUM_OF:usize = 3;

fn main() -> Result<(), Error> {
	// Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let mut lines = input.lines();

	let mut current: i64 = 0;
	let mut best = BinaryHeap::<i64>::with_capacity(SUM_OF);

	// Scan file
	loop {
		let mut chunk_finished = || {
			// println!("Chunk finished; {} > {}", current, best.peek());
			best.push(current);
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

	// Calculate final score
	{
		let mut count = 0;
		let mut total:i64 = 0;
		loop {
			total += match best.pop() {
				None => break,
				Some(x) => {
					//println!("Summing {}", x);
					x
				} 
			};
			// Unfortunately, if you go over the capacity of BinaryHeap it just keeps growing!
			// So this solution works, but is much less efficient than it could be.
			count = count + 1;
			if count >= SUM_OF { break }
		}
		println!("{}", total);
	}

	Ok(())
}
