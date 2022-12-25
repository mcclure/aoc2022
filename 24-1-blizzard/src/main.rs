// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use glam::{IVec2, IVec3};
use int_enum::IntEnum;
use ndarray::Array2;
use pathfinding::directed::astar::astar;
use ordered_float::NotNan;
use multimap::MultiMap;

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
type BlizzardsMap = Array2<Cell>;

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();

	const DIR_CHAR: [char;4] = ['>', 'v', '<', '^'];
	fn dir_for(ch:char) -> Option<Dir> { match ch { '>' => Some(Dir::Right), 'v' => Some(Dir::Down),
	                                                  '<' => Some(Dir::Left), '^' => Some(Dir::Up), _ => None } }
	const CARDINALS:[IVec2;4] = [IVec2::new(1,0), IVec2::new(0,1), IVec2::new(-1,0), IVec2::new(0,-1)];
	const NEIGHBORHOOD: [IVec2;9] = [IVec2::new(-1,-1), IVec2::new( 0,-1), IVec2::new( 1,-1),  // NW, N, NE
	                                 IVec2::new(-1, 0), IVec2::ZERO,       IVec2::new( 1, 0),  // W,  X, E
	                                 IVec2::new(-1, 1), IVec2::new( 0, 1), IVec2::new( 1, 1)]; // SW, S, SE

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
	let mut start_blizzards_map:BlizzardsMap = blizzards_map_new();
	for &(at, dir) in start_blizzards.iter() {
		start_blizzards_map[to_index(at)] = Cell::Blizzard(dir);
	}
	let start_blizzards_map = start_blizzards_map;

	struct Moment {
		blizzards: Vec<Blizzard>,
		blizzards_map: BlizzardsMap
	}

	let is_wall = |at:IVec2| { // Is outside playing field (will return true on start and end)
		IVec2::ZERO.cmpge(at).any() || size.cmple(at).any()
	};
	let print_moment = |map: &BlizzardsMap| {
		for y in 0..=size.y {
			for x in 0..=size.x {
				let at = IVec2::new(x as i32,y as i32);
				const FLOOR:char = '.';
				print!("{}", if at == start || at == end {
					FLOOR
				} else if is_wall(at) {
					'#'
				} else {
					match map[(y as usize,x as usize)] { // NOTE REVERSE INDEX
						Cell::Blizzard(dir) => { DIR_CHAR[dir as usize] },
						Cell::Multi => { '2' }
						_ => FLOOR
					}
				})
			}
			println!("");
		}
		println!("");
	};

	fn manhattan(v:IVec2) -> i32 { v.x.abs() + v.y.abs() }
	let mut target_time = manhattan(end-start) as usize; // Look, a "heuristic"

	let mut timeline: Vec<Moment> = vec![Moment{ blizzards:start_blizzards, blizzards_map:start_blizzards_map }];
	let mut nav_tree: MultiMap<IVec3, IVec3> = Default::default();

	let start3 = start.extend(0);

	let one = NotNan::new(1.0).unwrap();

	loop { // Loop until return
		// Build timeline
		while timeline.len() < target_time {
			let prev_moment = &timeline.last().unwrap();

			// Build moment
			let mut now = Moment { blizzards:vec![], blizzards_map: blizzards_map_new() };
			for &(at, dir) in prev_moment.blizzards.iter() {
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
				//println!("{}, {:?}: {}, {}, {}", at, dir, o, o_y, o_p);

				if o { // Wrap
					let p = if o_p {-1} else {1};
					if o_y {
						next.y += (size.y - 1)*p;
					} else {
						next.x += (size.x - 1)*p;
					}
				}

				now.blizzards.push((next, dir));
				let at_index = to_index(next);
				let cell = if now.blizzards_map[at_index] == Cell::Floor {
					Cell::Blizzard(dir)
				} else { Cell::Multi };
				now.blizzards_map[at_index] = cell;
			}

			// Build out legal-steps dictionary
			let ntimeline = timeline.len();
			for y in 0..size.y {
				for x in 0..size.x {
					let at = IVec2::new(x as i32,y as i32);
					let open = |check:IVec2, blizzards_map:&BlizzardsMap| {
						(check==start || check == end)
			         	|| (!is_wall(check) && {
			        	   let cell = blizzards_map[to_index(check)];
			        	   cell == Cell::Floor
			            })
					};
					if open(at, &prev_moment.blizzards_map) {
						for offset in NEIGHBORHOOD {
							let check = at + offset;
							//println!("At {} checking {} (START:{})", at, check, start);
							if open(check, &now.blizzards_map) {
								//println!("Adding {:?}", (at.extend(ntimeline as i32), check.extend(ntimeline as i32+1)));
								nav_tree.insert(at.extend(ntimeline as i32 - 1), check.extend(ntimeline as i32))
							}
						}
					}
				}
			}

			timeline.push(now);
		}

		// Search timeline
		// <N, C, FN, IN, FH, FS> N = IVec3, C = f32, IN = vec<(IVec3,f32)>
		if let Some((path, _)) = astar(
		    &start3,
		    |at| {
		    	let mut ok:Vec<(IVec3,NotNan<f32>)> = Default::default();
		    	//println!("At {}", at);
		    	if let Some(v) = nav_tree.get_vec(at) {
		    		for &at2 in v {
		    			//println!("\tCan go from {} to {}", at, at2);
		    			ok.push((at2, one)) // Note: Diagonal steps are still cost 1 becuase they take 1 second.
		    		}
		    	}
		    	ok
		    },
		    |&at| NotNan::new((at.truncate() - end).as_vec2().length()).unwrap(),
		    |&at| at.truncate() == end
		) {
			println!("SOLUTION\n{}", path.len());
			return Ok(())
		} else {
			println!("{} steps wasn't enough...", target_time);
			target_time *= 2;
			if target_time > 40000 { panic!("DEBUG: Bailing") }
		}
	}
}
