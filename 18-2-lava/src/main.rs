// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashSet;
use either::Either;
use glam::IVec3;
use ndarray::Array3;

#[derive(Debug,Copy,Clone,PartialEq)]
enum Cell {
	Unknown,
	Rock,
	Interior,
	Exterior
}
impl Default for Cell { fn default() -> Self { Cell::Unknown } }

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
	const ONE:IVec3 = IVec3::new(1,1,1);

	// Convert to ndarray
	fn to_index(v:IVec3) -> (usize, usize, usize) { (v.x as usize, v.y as usize, v.z as usize) }
	let size = max - min + ONE;
	let mut grid:Array3<Cell> = Array3::default(to_index(size));

	for v in vox {
		grid[to_index(v - min)] = Cell::Rock;
	}

	fn within (at:IVec3, size:IVec3) -> bool {
		IVec3::ZERO.cmple(at).all() && size.cmpgt(at).all()
	}

	// Flood fill
	for z in 0..size.z {
		for y in 0..size.y {
			for x in 0..size.x {
				let at = IVec3::new(x,y,z);
				if grid[to_index(at)] == Cell::Unknown {
					// Found one
					let mut fill: HashSet<IVec3> = Default::default();
					fill.insert(at);
					let cell = 'cell: {
						let mut next_pass: Vec<IVec3> = vec![at];
						while !next_pass.is_empty() {
							let pass = std::mem::take(&mut next_pass);
							for at in pass {
								for card in cardinals {
									let at = at + card;
									//println!("Flood fill maybe {} ({}? {})", at, size, within(at, size));
									if fill.contains(&at) { continue }
									if !within(at, size) {
										break 'cell Cell::Exterior;
									}
									//println!("Flood fill test {} = {:?}", at, grid[to_index(at)]);
									
									match grid[to_index(at)] {
										Cell::Unknown => next_pass.push(at),
										Cell::Interior => break 'cell Cell::Interior,
										Cell::Exterior => break 'cell Cell::Exterior,
										_ => continue // Don't insert
									}
									fill.insert(at);
								}
							}
						}
						Cell::Interior // The only way to get here is if we searched and found only walls.
					};
					for v in fill {
						//if cell == Cell::Interior { println!("WRITE INTERIOR") }
						grid[to_index(v)] = cell;
					}
				}
			}
		}
	}

	// Run
	for z in 0..size.z {
		for y in 0..size.y {
			for x in 0..size.x {
				let at = IVec3::new(x,y,z);
				if grid[to_index(at)] == Cell::Rock {
					for card in cardinals {
						let at = at + card;
						if !within(at, size) || grid[to_index(at)] == Cell::Exterior {
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
