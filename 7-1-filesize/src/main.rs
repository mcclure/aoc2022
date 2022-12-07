// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;

const BADSIZE:u64 = 100_000;

#[derive(Default)]
struct Dir {
	dir:HashMap<String, Dir>,
	size:u64
}

fn print_tree(d:&Dir, depth:usize) {
	for (k,v) in &d.dir {
		for _ in 0..depth { print!("\t") }
		println!("{}: {}", k, d.size);
		print_tree(d, depth+1);
	}
}

fn total_filesize(d:&Dir) -> u64 {
	let mut total = d.size;
	for d2 in d.dir.values() {
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

	{
		let mut pwd:Vec<&mut Dir> = Vec::new();
		pwd.push(&mut root);

		//let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

		// Scan file
		for line in lines {
			let line = line?;
		}
	}

	print_tree(&root, 0);

	let mut total: u64 = 0;

	for v in root.dir.values() {
		let size = total_filesize(v);
		if size > BADSIZE { total += size }
	}

	// Final score
	println!("{}", total);

	Ok(())
}
