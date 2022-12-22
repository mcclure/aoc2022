// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::cmp;
use either::Either;

type Num = i32;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
		None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
		Some("-") => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let mut numbers = {
		let lines = input.lines();
		// Original-index, value
		let mut numbers: Vec<(Num,Num)> = Default::default();
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }
			let x = line.parse::<Num>().map_err(|_|Error::new(ErrorKind::InvalidInput, "Argument 3 must be positive number"))?;
			numbers.push((numbers.len() as i32, x));
		}
		numbers
	};

	let nlen = numbers.len() as i32;

	#[cfg(debug_assertions)]
	let print = |n:&Vec<(Num,Num)>| { for (_,v) in n.iter() { print!("{}, ", v) } println!("") };

	#[cfg(debug_assertions)] { print(&numbers); }

	for target_idx in 0..numbers.len() {
		let (current_index, value, pair) = 'index: {
			for (idx,pair@(original_index, value)) in numbers.iter().enumerate() {
				if target_idx == (*original_index as usize) {
					break 'index (idx, value, pair)
				}
			}
			panic!("Number went missing");
		};
		let pair = *pair;
		let current_index = current_index as i32;
		let new_index = (current_index + value).rem_euclid(nlen);

		// Reseat
		if current_index != new_index {
			if current_index < new_index {
				for idx in current_index..new_index {
					numbers[idx as usize] = numbers[(idx + 1) as usize];
				}
			} else {
				for idx in ((new_index+1)..=current_index).rev() {
					numbers[idx as usize] = numbers[(idx - 1) as usize];
				}
			}
			numbers[new_index as usize] = pair;
		}

		#[cfg(debug_assertions)] { print(&numbers); }
	}

	let mut total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Final score
	println!("{}", total);

	Ok(())
}
