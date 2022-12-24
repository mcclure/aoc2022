// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;
use glam::{IVec2, IVec3};
use int_enum::IntEnum;
use ndarray::{Array2, Axis};
use pathfinding::directed::astar::astar;

#[repr(i8)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,IntEnum)]
enum Dir {
	Right = 0,
	Down = 1,
	Left = 2,
	Up = 3
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
enum Cell {
	Floor,
	Blizzard(Dir),
	Multi
}
impl Default for Cell { fn default() -> Self { Cell::Floor } }

type Blizzard = (IVec2, Dir);

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();

	const DIR_CHAR: [char;4] = ['>', 'v', '<', '^'];
	fn dir_for(ch:char) -> Option<Dir> { match ch { '>' => Some(Dir::Right), 'v' => Some(Dir::Down),
	                                                  '<' => Some(Dir::Left), '^' => Some(Dir::Up), _ => None } }
	let CARDINALS:[IVec2;4] = [IVec2::new(1,0), IVec2::new(0,1), IVec2::new(-1,0), IVec2::new(0,-1)];

	let (size, start, end, start_blizzards) = {
	    // Load file from command-line argument or (if -) stdin
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};
		let lines = input.lines();
		
		let (mut start, mut end, mut max) : (Option<IVec2>, IVec2, IVec2) = (None, IVec2::ZERO, IVec2::ZERO);
		let mut blizzards: Vec<Blizzard> = Default::default();

		let invalid = |ch:char| { return Err(Error::new(ErrorKind::InvalidInput, format!("Char not understood: '{}'", ch))) };

		for (y,line) in lines.enumerate() {
			let line = line?;
			let line = line.trim_end();
			if line.is_empty() { continue } // This better be at the fricking start

			for (x,ch) in line.chars().enumerate() {
				let at = IVec2::new(x as i32,y as i32);
				max = max.max(at);
				if ch == '.' {
					if start.is_none() { start = Some(at) }
					end = at;
				}
				else if let Some(dir) = dir_for(ch) {
					blizzards.push((at, dir))
				}
				else if ch != '#' {
					return invalid(ch);
				}
			}
		}

		if start.is_none() {
			return Err(Error::new(ErrorKind::InvalidInput, "No floors?"))
		}
		if max.x <= 1 || max.y <= 1 {
			return Err(Error::new(ErrorKind::InvalidInput, "Map too small?"))
		}

		(max, start.unwrap(), end, blizzards)
	};

	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	let blizzards_map_new = ||Array2::default(to_index(size));
	let mut start_blizzards_map:Array2<Cell> = blizzards_map_new();
	for &(at, dir) in start_blizzards.iter() {
		start_blizzards_map[to_index(at)] = Cell::Blizzard(dir);
	}
	let start_blizzards_map = start_blizzards_map;

	struct Moment {
		blizzards: Vec<Blizzard>,
		blizzards_map: Array2<Cell>
	}

	let is_wall = |at:IVec2| {
		IVec2::ZERO.cmpge(at).all() || size.cmple(at).all()
	};
	let print_moment = |map: &Array2<Cell>| {
		for (y,col) in map.axis_iter(Axis(1)).enumerate() {
			for (x,cell) in col.iter().enumerate() {
				let at = IVec2::new(x as i32,y as i32);
				const FLOOR:char = '.';
				print!("{}", if at == start || at == end {
					FLOOR
				} else if is_wall(at) {
					'#'
				} else if let &Cell::Blizzard(dir) = cell {
					DIR_CHAR[dir as usize]
				} else {
					FLOOR
				})
			}
			println!("");
		}
		println!("");
	};

	fn manhattan(v:IVec2) -> i32 { v.x.abs() + v.y.abs() }
	let mut target_time = manhattan(end-start) as usize; // Look, a "heuristic"

	let timeline: Vec<Moment> = vec![Moment{ blizzards:start_blizzards, blizzards_map:start_blizzards_map }];
	let mut wraps: HashMap<IVec2, IVec2> = Default::default();

	'solved: loop {
		while timeline.len() < target_time {
			let prev_moment = &timeline.last().unwrap().blizzards;
			let mut now = Moment { blizzards:vec![], blizzards_map: blizzards_map_new() };
			for &(at, dir) in prev_moment {
				let mut next = at + CARDINALS[dir as usize];

				let (mut o, mut o_y, mut o_p) = (false,false,false); // overflow, overflow y, overflow positive 
				if next.y <= 0 {
					o = true; o_y = true; o_p = false;
				} else if next.y >= size.y {
					o = true; o_y = true; o_p = true;
				} else if next.x <= 0 {
					o = true; o_y = false; o_p = false;
				} else if next.x >= size.x {
					o = true; o_y = false; o_p = true;
				}

				if o { // Wrap
					let p = if o_p {-1} else {1};
					if o_y {
						next.y += (size.y - 1)*p;
					} else {
						next.x += (size.x - 1)*p;
					}
				}

				now.blizzards.push((next, dir));
				let at_index = to_index(at);
				let cell = if now.blizzards_map[at_index] == Cell::Floor {
					Cell::Blizzard(dir)
				} else { Cell::Multi };

				print_moment(&now.blizzards_map);
			}
		}
	}

	let total: i64 = 0;

	// Final score
	println!("{}", total);

	Ok(())
}
