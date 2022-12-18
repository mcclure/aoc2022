// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use std::cmp;
use either::Either;
use glam::IVec2;

const MINO_SPEC: &str = "\
####

.#.
###
.#.

..#
..#
###

#
#
#
#

##
##";

const SIMULATION:usize = 2022;
const WIDTH:i32 = 7;
const SPAWN_X:i32 = 2; // 2 from left wall
const SPAWN_Y:i32 = 3; // 3 above highest point

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let filename = std::env::args().fuse().nth(1);

	let ctrl = {
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let invalid = |c| { return Err(Error::new(ErrorKind::InvalidInput, format!("Expecting < and >, saw '{}'", c))) };

		let mut ctrl: Vec<bool> = Default::default();

		// Scan file
		for byte in input.bytes() {
			ctrl.push(match byte? {
				b'>' => true,
				b'<' => false,
				c => if (c as char).is_whitespace() { 
					break
				} else {
					return invalid(c)
				}
			});
		}
		ctrl
	};

	let (minos, mino_heights) = {
		let mut minos: Vec<HashSet<IVec2>> = Default::default();
		let mut mino_heights: Vec<i32> = Default::default();
		let mut mino: HashSet<IVec2> = Default::default();
		let mut height = 0;
		for (y,line) in MINO_SPEC.lines().enumerate() {
			if line.is_empty() {
				minos.push(std::mem::take(&mut mino));
				mino_heights.push(std::mem::take(&mut height));
			} else {
				for (x,ch) in line.chars().enumerate() {
					mino.insert(IVec2::new(x as i32,y as i32));
				}
				height += 1;
			}
		}
		minos.push(mino);
		mino_heights.push(height);
		(minos, mino_heights)
	};

	let mut board: HashSet<IVec2> = Default::default();
	let mut max:i32 = 0;
	let mut at: Option<IVec2> = None; 
	let mut mino_at = 0;

	for t in 0..SIMULATION {
		let mino = &minos[mino_at];
		if at.is_none() { // New piece
			at = Some(IVec2::new(SPAWN_X, max + SPAWN_Y + mino_heights[mino_at]));
		}
		match at {
			Some(at_unwrap) => {
				let right = ctrl[t % ctrl.len()];
				let mut try_move = |v:IVec2, down:bool| {
					let at_moved = at_unwrap + v;
					for &cell in mino {
						let at_cell = at_moved + cell;
						if at_cell.x < 0 || at_cell.x >= WIDTH { return }
						if board.contains(&at_cell) {
							if down {
								mino_at += 1;
								mino_at %= minos.len();
								for &cell in mino { // Shadow
									let at_cell = at_unwrap + cell;
									board.insert(at_cell);
								}
								max = cmp::max(max, at_unwrap.y);
								at = None;
							}
							return
						}
					}
					at = Some(at_moved);
				};
				try_move(if right { IVec2::X } else { -IVec2::X }, false);
				try_move(-IVec2::Y, true);
			}
			_ => unreachable!()
		}
	}

	// Final score
	println!("{}", max);

	Ok(())
}
