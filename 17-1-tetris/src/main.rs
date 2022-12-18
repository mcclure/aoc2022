// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
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

const SIMULATION:i32 = 2022;
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
				c => return invalid(c)
			});
		}
	};

	let minos = {
		let mut minos: Vec<HashSet<IVec2>> = Default::default();
		let mut mino: HashSet<IVec2> = Default::default();
		for (y,line) in MINO_SPEC.lines().enumerate() {
			if line.is_empty() {
				minos.push(std::mem::take(&mut mino));
			} else {
				for (x,ch) in line.chars().enumerate() {
					mino.insert(IVec2::new(x as i32,y as i32));
				}
			}
		}
		minos.push(mino);
		minos
	};

	let mut board: HashSet<IVec2> = Default::default();
	let mut max:i32 = 0;

	for t in 0..SIMULATION {

	}

	// Final score
	println!("{}", max);

	Ok(())
}
