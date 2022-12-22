// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;
use int_enum::IntEnum;
use ndarray::{Array2, ArrayView, Axis};
use glam::IVec2;

#[repr(u8)]
#[derive(Debug,Copy,Clone,PartialEq,IntEnum)]
enum Dir {
	Right = 0,
	Down = 1,
	Left = 2,
	Up = 3
}

enum Cell {
	Blank,
	Floor,
	Wall,
	#[cfg(debug_assertions)] FloorRecord(Dir)
}

type Steps = u32;

enum Instr {
	Turn(bool), // true for R, false for L
	Forward(Steps)
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();

	let (map, instructions) = {
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};
		let mut lines = input.lines().enumerate();

		let invalid_blank = ||Err(Error::new(ErrorKind::InvalidInput, "No instructions at end of file"));

		let map: Array2<Cell>;
		{
			let sparse_map: HashMap<IVec2, Cell> = Default::default();

			loop {
				if let Some((y,line)) = lines.next() {
					let line = line?;
					let line = line.trim_end();
					if line.is_empty() { break } // NOT DONE

					for (x,ch) in line.chars().enumerate() {

					}

					// TODO initialize map here
				} else {
					return invalid_blank();
				}
			}
		}

		let instructions:Vec<Instr>;
		{
			if let Some((_,line)) = lines.next() {
				let line = line?;
				let line = line.trim();
				if line.is_empty() { return invalid_blank() }

				use pom::parser::*;

				fn positive<'a>() -> Parser<'a, u8, Steps> {
					let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
					integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<Steps>())
				}

				fn token<'a>() -> Parser<'a, u8, Instr> {
					sym(b'L').map(|_|Instr::Turn(false)) |
					sym(b'R').map(|_|Instr::Turn(true)) |
					positive().map(|x|Instr::Forward(x))
				} 

				fn statement<'a>() -> Parser<'a, u8, Vec<Instr>> {
					token().repeat(1..)
				}

				let invalid = |s:&str| { Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };

				let parsed = statement().parse(line.as_bytes());
				match parsed {
					Err(_) => return invalid(line),
					Ok(x) => {
						instructions = x;
					}
				}
			} else {
				return invalid_blank()
			}
		}

		(map, instructions)
	};

	let mut total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Final score
	println!("{}", total);

	Ok(())
}
