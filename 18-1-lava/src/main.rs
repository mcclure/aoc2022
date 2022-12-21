// Calculate the surface area of a voxel object

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashSet;
use either::Either;
use glam::IVec3;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
		None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
		Some("-") => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	let invalid = |s| { return Error::new(ErrorKind::InvalidInput, format!("Line must be list of 3 comma-separated numbers, but got '{}'", s)) };

	let mut vox:HashSet<IVec3> = Default::default();
	let (mut min, mut max) = (IVec3::new(i32::MAX, i32::MAX, i32::MAX), IVec3::new(i32::MIN, i32::MIN, i32::MIN));

	// Scan file
	for line in lines {
		let line = line?;
		let i3 = line.split(",").map(|x|x.parse::<i32>().map_err(|_|invalid(line.clone()))).collect::<Result<Vec<i32>,Error>>()?;
		if i3.len() != 3 { return Err(invalid(line)) }
		let v = IVec3::from_slice(&i3[0..3]);
		vox.insert(v);
		min = min.min(v);
		max = max.max(v);
	}

	let cardinals = [IVec3::new(-1,0,0), IVec3::new(1,0,0),
				     IVec3::new(0,-1,0), IVec3::new(0,1,0),
				     IVec3::new(0,0,-1), IVec3::new(0,0,1)];

	for z in min.z..=max.z {
		for y in min.y..=max.y {
			for x in min.x..=max.x {
				let at = IVec3::new(x,y,z);
				if vox.contains(&at) {
					for card in cardinals {
						let at = at + card;
						if !vox.contains(&at) {
							total += 1;
						}
					}
				}
			}
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
