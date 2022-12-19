// Summary

use std::io::{BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use std::cmp;
use either::Either;
use glam::IVec2;

// NOTICE YFLIP
const MINO_SPEC: &str = "\
####

.#.
###
.#.

###
..#
..#

#
#
#
#

##
##";

const MAX_FROZEN:usize = 2022;
const WIDTH:i32 = 7;

// Note: Spawn point refers to the BOTTOM LEFT coordinate,
// But "at" variable refers to TOP LEFT coordinate.
const SPAWN_X:i32 = 2; // 2 from left wall
const SPAWN_Y:i32 = 3; // 3 above highest point

const DEBUG_VERBAL:bool = false;
const DEBUG_FREEZE:bool = false;
const DEBUG_FINAL:bool = true;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
    let mut args = std::env::args().fuse();

	let ctrl = {
		let filename = args.nth(1);
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
		let mut y_root:usize = 0;
		let mut height:usize = 0;
		for (y,line) in MINO_SPEC.lines().enumerate() {
			if line.is_empty() {
				minos.push(std::mem::take(&mut mino));
				mino_heights.push(std::mem::take(&mut height) as i32);
				y_root = y+1;
			} else {
				for (x,ch) in line.chars().enumerate() {
					if ch == '#' {
						mino.insert(IVec2::new(x as i32,(y-y_root) as i32));
					}
				}
				height += 1;
			}
		}
		minos.push(mino);
		mino_heights.push(height as i32);
		(minos, mino_heights)
	};

	let report_progress = match match args.next() {
		Some(x) => Some(x.parse::<usize>().map_err(|_|Error::new(ErrorKind::InvalidInput, "Argument 2 must be positive number"))?),
		None => None
	} { Some(0) => None, x => x };

	let simulation_length = match args.next() {
		Some(x) => Some(x.parse::<usize>().map_err(|_|Error::new(ErrorKind::InvalidInput, "Argument 3 must be positive number"))?),
		None => None
	};

	let simulation_range = match simulation_length {
		Some(x) => Either::Left(0..x), None => Either::Right(0..)
	};

	let max_frozen = match match args.next() {
		Some(x) => Some(x.parse::<usize>().map_err(|_|Error::new(ErrorKind::InvalidInput, "Argument 4 must be positive number"))?),
		None => None
	} { Some(x) => x, None => MAX_FROZEN };

	let mut board: HashSet<IVec2> = Default::default();
	let mut watermark:i32 = 0;
	let mut at: Option<IVec2> = None; 
	let mut mino_at = 0;
	let mut frozen = 0; // For debugging

	for t in simulation_range {
		let mino = &minos[mino_at];
		if at.is_none() { // New piece
			at = Some(IVec2::new(SPAWN_X, watermark + SPAWN_Y));
			if DEBUG_VERBAL { println!("New piece {} at {:?}", mino_at, at) }
		}
		{
			let right = ctrl[t % ctrl.len()];
			let mut try_move = |v:IVec2, down:bool| {
				let at_unwrap = match at { Some(at) =>at, None => unreachable!() };
				let at_moved = at_unwrap + v;
				if DEBUG_VERBAL { println!("{:?}+{:?} Trying move to {}...", at, v, at_moved) }
				for &cell in mino {
					let at_cell = at_moved + cell;
					//if DEBUG_VERBAL { println!("Check x {} {} {} {}", at_cell.x, at_cell.x < 0, at_cell.x >= WIDTH, at_cell.x < 0 || at_cell.x >= WIDTH) }
					if at_cell.x < 0 || at_cell.x >= WIDTH { return }
					if at_cell.y < 0 || board.contains(&at_cell) {
						if down {
							if DEBUG_VERBAL { println!("...froze!") }
							mino_at += 1;
							mino_at %= minos.len();
							for &cell in mino { // Shadow
								match at {
									Some(at_unwrap) => {
										let at_cell = at_unwrap + cell;
										board.insert(at_cell);
										watermark = cmp::max(watermark, at_cell.y+1);
									},
									None => unreachable!() // Because None can only be set in second call
								}
							}
							frozen += 1;
							at = None;
						}
						return
					}
				}
				at = Some(at_moved);
				if DEBUG_VERBAL { println!("...success. {:?}", at) }
			};
			try_move(if right { IVec2::X } else { -IVec2::X }, false);
			try_move(-IVec2::Y, true);
		}
		if match report_progress {
			None => false,
			Some(x) => 0 == t%x
		} || (DEBUG_FREEZE && at.is_none())
		  || ((DEBUG_FINAL || !report_progress.is_none()) &&
		  	  (match simulation_length { None => false, Some(x) => t==(x-1) } 
		  	   || frozen == max_frozen)) {
			println!("Height: {}", watermark);
			for y in (0..=(match at { None => watermark, Some(at) => {println!("{}+{}",at.y,mino_heights[mino_at]);cmp::max(watermark, at.y+mino_heights[mino_at])} })).rev() {
				print!("|");
				for x in 0..WIDTH {
					let check = IVec2::new(x,y);
					print!("{}",
						if match at { None => false, Some(at) => mino.contains(&(check - at))} { '@' }
						else if board.contains(&check) { '#' }
						else { '.' }
					);
				}
				println!("|");
			}
			println!("");
		}

		if frozen >= max_frozen { break }
	}

	// Final score
	println!("{}", watermark);

	Ok(())
}
