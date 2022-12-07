// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

struct Dir {
	dir:Vec<Dir>,
	size:i64
}

impl Default for Dir {
    fn default() -> Dir {
        Dir {
            dir: Vec::new(),
            size:0
        }
    }
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut dir:Dir = Default::default();

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
