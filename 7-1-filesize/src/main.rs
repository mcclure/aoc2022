// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

#[derive(Default)]
struct Dir {
	dir:HashMap<Dir>,
	size:i64
}

fn print_tree(d:Dir, depth:usize) {
	for 0..depth { print!("\t") }
	println!()
	for d2 in d.dir {
		total += total_filesize(d2)
	}
	return total
}

fn total_filesize(d:Dir) {
	let mut total = d.size;
	for d2 in d.dir {
		total += total_filesize(d2)
	}
	return total
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut root:Dir = Default::default();

	// Scan file
	for line in lines {
		let line = line?;
	}

	let mut total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Final score
	println!("{}", total);

	Ok(())
}
