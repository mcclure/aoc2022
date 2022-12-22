// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;
use int_enum::IntEnum;
use ndarray::{Array2, Axis};
use glam::IVec2;

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

	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	//fn within (at:IVec2, size:IVec2) -> bool {
  	//	IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
  	//}

	let (map, map_max, mut player, instructions) = {
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
		let player: Player;
		{
			let mut sparse_map: HashMap<IVec2, Cell> = Default::default();
			let mut player_at: Option<IVec2> = None;

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

			map = Array2::default(to_index(max + IVec2::ONE));
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

		(map, max, player, instructions)
	};

	#[cfg(debug_assertions)]
	let mut map = map;

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
		match instr {
			Instr::Turn(dir) => {
				player.dir = Dir::from_int(((player.dir as i8) + if dir { 1 } else { -1 }).rem_euclid(4)).unwrap()
			},
			Instr::Forward(mut steps) => {
				let mut next = player.at;
				let step = cardinals[player.dir as usize];
				loop {
					if steps <= 0 { break }
					//print_map(&map, Some(&player));println!("---------");

					next += step;
					next = IVec2::new(next.x.rem_euclid(map_max.x), next.y.rem_euclid(map_max.y));
					if next == player.at { panic!("NO FLOORS?!") }
					match map[to_index(next)] {
						#[cfg(debug_assertions)]
						Cell::FloorRecord(_) |
						Cell::Floor => {
							player.at = next;

							steps -= 1;
							#[cfg(debug_assertions)] {
								map[to_index(next)] = Cell::FloorRecord(player.dir);	
							}

							continue
						}
						Cell::Wall => {
							break
						}
						_ => ()
					}
				}
			}
		}
	}

	print_map(&map, Some(&player));

	let total: i64 = 0;

//	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "Expecting other")) };

	// Final score
	println!("{}", total);

	Ok(())
}
