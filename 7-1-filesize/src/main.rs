// Snoop a command line history and print the sum of all directories larger than 100kb
// Has various problems:
// - Will crash on too-deep stack depth.
// - Can't handle Unicode input (or at least not Unicode whitespace).
// - Memory inefficient.
// - Assumes no spaces in filenames, which is probably fine?

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
		println!("{}: {}", k, v.size);
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

	// Line parser
	{
		use pom::parser::*;

		let mut pwd:Vec<&mut Dir> = Vec::new();
		pwd.push(&mut root);

		let invalid = |s:String| { return Err(Error::new(ErrorKind::InvalidInput, format!("Unrecognized line: '{}'", s))) };

		enum Parsed {
			Ls,         // Reset
			Dir,        // Ignore
	        Cd(String), // Change directory
	        Size(u64)   // Listed
	    }

//	    const ls_seq: [char; 2] = ['l', 's'];

	    fn splode(s:&str) -> Vec<char> {
	    	s.chars().collect()
	    }

		fn positive<'a>() -> Parser<'a, char, u64> {
			let integer = one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
			integer.collect().convert(|s|String::from_iter(s.iter()).parse::<u64>())
		}

		fn whitespace<'a>() -> Parser<'a, char, ()>
			{ one_of(" \t").repeat(1..).discard() }

		fn cli_prefix<'a>() -> Parser<'a, char, ()>
			{ empty() - sym('$') - whitespace() }

		fn cli_ls<'a>() -> Parser<'a, char, Parsed> {
			let pattern = cli_prefix() * seq(&['l', 's']);
			pattern.map(|_| Parsed::Ls)
		}
		const DIR_SLICE:[char;3] = ['d', 'i', 'r'];
		fn cli_dir<'a>() -> Parser<'a, char, Parsed> {
			let pattern = empty() - seq(&DIR_SLICE) - whitespace() - none_of(" \t").repeat(1..);
			pattern.map(|_| Parsed::Dir)
		}
		fn cli_size<'a>() -> Parser<'a, char, Parsed> {
			let pattern = positive() - whitespace() - none_of(" \t").repeat(1..);
			pattern.map(|x| Parsed::Size(x))
		}
		fn cli_cd<'a>() -> Parser<'a, char, Parsed> {
			let pattern = cli_prefix() - seq(&['c', 'd']) - whitespace() * none_of(" \t").repeat(1..);
			pattern.collect().map(|x| Parsed::Cd(x.iter().collect()))
		}
		fn cli_line<'a>() -> Parser<'a, char, Parsed>
			{ cli_ls() | cli_dir() | cli_size() | cli_cd() }

		// Scan file
		for line in lines {
			let line = line?;
			let line_array:Vec<char> = splode(&line);
			let content = cli_line().parse(&line_array);
			match content {
				Ok(Parsed::Ls) => pwd.last_mut().unwrap().size = 0,
				Ok(Parsed::Dir) => (),
				Ok(Parsed::Cd(s)) => {
					match s.as_str() {
						"/" => pwd.truncate(1),
						".." => if pwd.len() > 1 { pwd.pop(); },
						_ => pwd.push( &mut pwd.last_mut().unwrap().dir[&s] )
					}
				},
				Ok(Parsed::Size(s)) => pwd.last_mut().unwrap().size += s,
				_ => return invalid(line)
			}
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
