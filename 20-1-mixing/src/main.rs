// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
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
		let new_index = current_index + value;
		// This isn't a normal modulo, it bumps on wrap
		let new_index = 
			if new_index < 0 {
				let offset = (-new_index) / nlen + 1;
				(new_index - offset).rem_euclid(nlen)
			} else if new_index >= nlen {
				let offset = (new_index) / nlen;
				(new_index + offset).rem_euclid(nlen)
			} else {
				new_index
			};

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

	let zero_at = numbers.iter().position(|(_,x)|*x==0).unwrap(); // Succeeds or corrupt array
	let probes:Vec<i32> = vec![1000,2000,3000];

	let mut total: i64 = 0;

	for probe in probes {
		let (_, v) = numbers[(zero_at + (probe as usize))%numbers.len()];
		print!("{}, ", v);
		total += v as i64;
	}
	println!("=");

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Final score
	println!("{}", total);

	Ok(())
}
