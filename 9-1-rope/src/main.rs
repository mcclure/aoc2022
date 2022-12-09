// Intake a series of commands to move a two-cell "rope" on a grid

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use itertools::Itertools;

const DEBUG:bool = false;

// "Debug line", "debug single"
macro_rules! d { ( $( $x:expr ),* ) => { if (DEBUG) { println!($($x,)*) } }; }
macro_rules! ds { ( $( $x:expr ),* ) => { if (DEBUG) { print!($($x,)*) } }; }

#[derive(PartialEq,PartialOrd)]
enum Cell { Empty, Headed, Tailed, Start }
impl Default for Cell { fn default() -> Self { Cell::Empty } }

type At = (i32,i32);

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	let invalide =  || { Error::new(ErrorKind::InvalidInput, "Unrecognized command") };
	let invalid =   || { Err(invalide()) };
	let invalide2 = || { Error::new(ErrorKind::InvalidInput, "Non-integer argument") };

	let mut map: HashMap<At, Cell> = Default::default();
	let map_write = |at:At,v:Cell| {
		let entry = map.entry(at);
		if match entry { Entry::Vacant(_) => true, Entry::Occupied(v2) => v>*v2.get()} {
			entry.insert_entry(v);
		}
	};

	fn map_write_if_greater<K,T>(map:HashMap<K,T>, key:K, v:T) where K:Eq+std::hash::Hash, T:PartialEq+PartialOrd {
		let entry = map.entry(key);
		if match entry { Entry::Vacant(_) => true, Entry::Occupied(v2) => v>*v2.get()} {
			entry.insert_entry(v);
		}
	}

	// Scan file
	{
		let mut head_at = (0,0);
		map_write_if_greater(map, head_at, Cell::Start);
		for line in lines {
			let line = line?;
			let (dir_str, num_str) = line.split_whitespace().collect_tuple().unwrap_or_else(invalide)?;
			let dir = 
				match dir_str {
					"U" => (0,-1), "D" => (0,1), "L" => (-1,0), "R" => (1,0),
					_ => return invalid()
				};
			let count = num_str.parse::<usize>().map_err(|_|invalide2())?;
			for _ in 0..count {
				head_at += dir;
				map_write(head_at, Cell::Headed);
			}
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
