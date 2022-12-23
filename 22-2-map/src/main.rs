// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;
use int_enum::IntEnum;
use ndarray::{Array2, Axis};
use glam::IVec2;

#[cfg(debug_assertions)]
const DEBUG_STEP:bool = false;

#[repr(i8)]
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

#[derive(Debug)]
enum Instr {
	Turn(bool), // true for R, false for L
	Forward(Steps)
}

#[derive(Debug)]
struct Player {
	at:IVec2,
	dir:Dir
}
impl Player {
	fn new(at:IVec2) -> Self { Player {at, dir:Dir::Right} }
}

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();

	#[cfg(not(any(topology="0", topology="1")))]
	compile_error!("Pass --cfg topology=0 or --cfg topology=1 when building.");

	// Faces are 1-indexed, is that a problem?
	#[cfg(topology="0")]
	const CUBE_FACE_BYTES: [[u8;4];3] = [
		*b"  1 ",
		*b"234 ",
		*b"  56"];
	#[cfg(topology="0")]
	const CUBE_FACE_BYTES_SIZE: IVec2 = IVec2::new(4,3);
	#[cfg(topology="0")]
	const FACE_AT:[IVec2;6] = [
		                                  IVec2::new(2,0),
		IVec2::new(0,1), IVec2::new(1,1), IVec2::new(2,1),
		                                  IVec2::new(2,2), IVec2::new(3,2)
	];
	#[cfg(topology="0")]
	fn cube_face_exit(x: (u8, Dir)) -> (u8, Dir) { match x {
			(1, Dir::Left)  => (3, Dir::Down),
			(1, Dir::Right) => (6, Dir::Left),
			(1, Dir::Up)    => (2, Dir::Down),
			(2, Dir::Left)  => (6, Dir::Up),
			(2, Dir::Down)  => (5, Dir::Up),
			(3, Dir::Down)  => (5, Dir::Right),
			(4, Dir::Right) => (6, Dir::Down),
			// REVERSE
			(3, Dir::Up)    => (1, Dir::Right),
			(6, Dir::Right) => (1, Dir::Left),
			(2, Dir::Up)    => (1, Dir::Down),
			(6, Dir::Down)  => (2, Dir::Right),
			(5, Dir::Down)  => (2, Dir::Up),
			(5, Dir::Left)  => (3, Dir::Up),
			(6, Dir::Up)   => (4, Dir::Left),

			_ => panic!("Impossible cube ?! {:?}", x) 
	} }

	// Faces are 1-indexed, is that a problem?
	#[cfg(topology="1")]
	const CUBE_FACE_BYTES: [[u8;3];4] = [
		*b" 12",
		*b" 3 ",
		*b"45 ",
		*b"6  "];
	#[cfg(topology="1")]
	const CUBE_FACE_BYTES_SIZE: IVec2 = IVec2::new(3,4);
	#[cfg(topology="1")]
	const FACE_AT:[IVec2;6] = [
		                 IVec2::new(1,0), IVec2::new(2,0),
		                 IVec2::new(1,1),
		IVec2::new(0,2), IVec2::new(1,2),
		IVec2::new(0,3)
	];
	#[cfg(topology="1")]
	fn cube_face_exit(x: (u8, Dir)) -> (u8, Dir) { match x {
			(1, Dir::Left)  => (4, Dir::Right),
			(1, Dir::Up)    => (6, Dir::Right),
			(2, Dir::Up)    => (6, Dir::Up),
			(2, Dir::Right) => (5, Dir::Left),
			(3, Dir::Right) => (2, Dir::Up),
			(3, Dir::Left)  => (4, Dir::Down),
			(5, Dir::Down)  => (6, Dir::Left),
			// REVERSE
			(4, Dir::Left)  => (1, Dir::Right),
			(6, Dir::Left)  => (1, Dir::Down),
			(6, Dir::Down)  => (2, Dir::Down),
			(5, Dir::Right) => (2, Dir::Left),
			(2, Dir::Down)  => (3, Dir::Left),
			(4, Dir::Up)    => (3, Dir::Right),
			(6, Dir::Right) => (5, Dir::Up),

			_ => panic!("Impossible cube ?! {:?}", x) 
	} }

	fn cube_face_at(face:u8) -> IVec2 {
		return FACE_AT[face as usize - 1]
	}
	#[cfg(unused)]
	fn dir_reverse(d:Dir) -> Dir {
		match d {
			Dir::Left => Dir::Right,
			Dir::Down => Dir::Up,
			Dir::Right => Dir::Left,
			Dir::Up => Dir::Down
		}
	}
	fn turn_square(v:IVec2, size:IVec2, turns:i8) -> IVec2 {
		match turns {
			0 => v,
			2 => size - v - IVec2::ONE,
			1 => IVec2::new(size.y - v.y - 1, v.x),
			3 => IVec2::new(v.y, size.x - v.x - 1),
			_ => panic!("Turn should have been mod 4")
		}
	}

	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	fn within (at:IVec2, size:IVec2) -> bool {
  		IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
  	}

	fn cube_face(v:IVec2) -> u8 {
		if !within(v, CUBE_FACE_BYTES_SIZE) { panic!("cube_face bad query") }
		return CUBE_FACE_BYTES[v.y as usize][v.x as usize] - ('0' as u8)
	}

	let (map, map_size, mut player, instructions) = {
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};
		let mut lines = input.lines().enumerate();

		let invalid_blank = ||Err(Error::new(ErrorKind::InvalidInput, "No instructions at end of file"));

		let mut map: Array2<Cell>;
		let size:IVec2;
		let player: Player;
		{
			let mut sparse_map: HashMap<IVec2, Cell> = Default::default();
			let mut player_at: Option<IVec2> = None;
			let mut max:IVec2 = IVec2::ZERO;

			loop {
				if let Some((y,line)) = lines.next() {
					let line = line?;
					let line = line.trim_end();
					if line.is_empty() { break } // NOT DONE

					'ch: for (x,ch) in line.chars().enumerate() {
						let cell = match ch {
							'.' => {
								if player_at.is_none() { player_at = Some(IVec2::new(x as i32, y as i32))}
								Cell::Floor
							},
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

			if player_at.is_none() { return Err(Error::new(ErrorKind::InvalidInput, "No floors in grid")) }
			player = Player::new(player_at.unwrap());

			size = max + IVec2::ONE;
//println!("MAX {} SIZE {}", max, size);
			map = Array2::default(to_index(size));
			for (at,cell) in sparse_map {
				map[to_index(at)] = cell;
			}
			#[cfg(debug_assertions)] {
				map[to_index(player.at)] = Cell::FloorRecord(player.dir);
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

		(map, size, player, instructions)
	};

	#[cfg(debug_assertions)]
	let mut map = map;

	let face_size = map_size.y / CUBE_FACE_BYTES_SIZE.y;
	let face_size_vec = IVec2::new(face_size, face_size);

	{
		let expected_size = face_size * CUBE_FACE_BYTES_SIZE;
		if expected_size != map_size {
			return Err(Error::new(ErrorKind::InvalidInput, format!("Unusual size, expected like {} but got {}", expected_size, map_size)))
		}
	}

	fn dir_char(dir:Dir) -> char {
		match dir {
			Dir::Right => '>',
			Dir::Down => 'v',
			Dir::Left => '<',
			Dir::Up => '^',
		}
	}

	fn print_map(map: &Array2<Cell>, player:Option<&Player>) {
		use ansi_term::Style;
		use ansi_term::Colour::{Black, White};

		let invert = Style::new().fg(Black).on(White);

		for (y,col) in map.axis_iter(Axis(0)).enumerate() {
			for (x,cell) in col.iter().enumerate() {
				if let Some(player) = &player {
					if player.at.x == x as i32 && player.at.y == y as i32 {
						print!("{}", invert.paint(dir_char(player.dir).to_string()));
						continue;
					}
				}
				print!("{}", match cell {
					Cell::Blank => ' ',
					Cell::Floor => '.',
					Cell::Wall =>  '#',
					#[cfg(debug_assertions)]
					Cell::FloorRecord(dir) => dir_char(*dir)
				})
			}
			println!("");
		}
		println!("");
	}

	//print_map(&map, Some(&player));

	let cardinals = [IVec2::new(1,0), IVec2::new(0,1), IVec2::new(-1,0), IVec2::new(0,-1)];

	for instr in instructions {
#[cfg(debug_assertions)] if DEBUG_STEP { print_map(&map, Some(&player));println!("\n---------\nSTEP: {:?}\n", instr); }
		match instr {
			Instr::Turn(dir) => {
				player.dir = Dir::from_int(((player.dir as i8) + if dir { 1 } else { -1 }).rem_euclid(4)).unwrap()
			},
			Instr::Forward(mut steps) => {
				let mut next = player.at;
				let mut next_dir = player.dir;
				loop {
					if steps <= 0 { break }

					let last = next;
					next += cardinals[next_dir as usize];

					if !within(next, map_size) || map[to_index(next)] == Cell::Blank {
//println!("Will wrap at {}", next);
						let face = cube_face(last/face_size);
						let (new_face, new_dir) = cube_face_exit((face, next_dir));
						let turn = ((new_dir as i8) - (next_dir as i8)).rem_euclid(4);
//println!("Leaving face {}, dir {:?} into face {}, dir {:?}, turn {}", face, next_dir, new_face, new_dir, turn);
//println!("Pre wrap:"); print_map(&map, Some(&Player{at:next, dir:next_dir}));
						let adjusted = next - cube_face_at(face)*face_size;
//println!("Defaced:"); print_map(&map, Some(&Player{at:adjusted, dir:next_dir}));
						let adjusted = IVec2::new(adjusted.x.rem_euclid(face_size), adjusted.y.rem_euclid(face_size));
//println!("Wrapped:"); print_map(&map, Some(&Player{at:adjusted, dir:next_dir}));
						let adjusted = turn_square(adjusted, face_size_vec, turn);
						next_dir = new_dir;
//println!("Turned:"); print_map(&map, Some(&Player{at:adjusted, dir:next_dir}));
						next = adjusted + cube_face_at(new_face)*face_size;
//println!("Refaced:"); print_map(&map, Some(&Player{at:next, dir:next_dir}));
//println!("Standing on: {:?}", map[to_index(next)]);
					}

					if next == player.at { panic!("NO FLOORS?!") }
					match map[to_index(next)] {
						#[cfg(debug_assertions)]
						Cell::FloorRecord(_) |
						Cell::Floor => {
							player.at = next;
							player.dir = next_dir;

							steps -= 1;
							#[cfg(debug_assertions)] {
								map[to_index(next)] = Cell::FloorRecord(player.dir);	
							}

							continue
						}
						Cell::Wall => {
							break
						}
						_ => panic!("Fell into void, problem with within() check") // check should have wrapped out of Blank zone
					}
				}
			}
		}
	}

	print_map(&map, Some(&player));

	let total: i64 = (player.at.y+1) as i64*1000 + (player.at.x+1) as i64*4 + player.dir as i64;

	// Final score
	println!("{}", total);

	Ok(())
}
