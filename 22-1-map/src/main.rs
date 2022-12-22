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

#[derive(Debug,Copy,Clone,PartialEq)]
enum Cell {
	Blank,
	Floor,
	Wall,
	#[cfg(debug_assertions)] FloorRecord(Dir)
}
impl Default for Cell { fn default() -> Self { Cell::Blank } }

type Steps = u32;

enum Instr {
	Turn(bool), // true for R, false for L
	Forward(Steps)
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();

	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	fn within (at:IVec2, size:IVec2) -> bool {
  		IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
  	}

	let (map, map_max, instructions) = {
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};
		let mut lines = input.lines().enumerate();

		let invalid_blank = ||Err(Error::new(ErrorKind::InvalidInput, "No instructions at end of file"));

		let mut map: Array2<Cell>;
		let mut max:IVec2 = IVec2::ZERO;
		{
			let mut sparse_map: HashMap<IVec2, Cell> = Default::default();

			loop {
				if let Some((y,line)) = lines.next() {
					let line = line?;
					let line = line.trim_end();
					if line.is_empty() { break } // NOT DONE

					'ch: for (x,ch) in line.chars().enumerate() {
						let cell = match ch {
							'.' => Cell::Floor,
							'#' => Cell::Wall,
							' ' => continue 'ch,
							_ => return Err(Error::new(ErrorKind::InvalidInput, format!("Unrecognized character '{}'", ch)))
						};
						let at = IVec2::new(x as i32,y as i32);
						sparse_map.insert(at, cell);
						max = max.max(at);
					}

					// TODO initialize map here
				} else {
					return invalid_blank();
				}
			}

			map = Array2::default(to_index(max + IVec2::ONE));
			for (at,cell) in sparse_map {
				map[to_index(at)] = cell;
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

		(map, max, instructions)
	};

	fn print_map(map: Array2<Cell>) {
		for col in map.axis_iter(Axis(0)) {
			for cell in col.iter() {
				print!("{}", match cell {
					Cell::Blank => ' ',
					Cell::Floor => '.',
					Cell::Wall =>  '#',
					#[cfg(debug_assertions)]
					Cell::FloorRecord(dir) => match dir {
						Dir::Right => '>',
						Dir::Down => 'v',
						Dir::Left => '<',
						Dir::Up => '^',
					}
				})
			}
			println!("");
		}
		println!("");
	}

	print_map(map);

	let mut total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Final score
	println!("{}", total);

	Ok(())
}
