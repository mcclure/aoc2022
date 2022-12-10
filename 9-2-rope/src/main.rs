// Intake a series of commands to move a two-cell "rope" on a grid

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;
use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use itertools::Itertools;
use std::cmp::{min, max};
use ndarray::{Axis, Array2};

const DEBUG:bool = false;
const ROPE_LEN:usize = 10;

// "Debug line", "debug single"
macro_rules! d { ( $( $x:expr ),* ) => { if (DEBUG) { println!($($x,)*) } }; }
macro_rules! ds { ( $( $x:expr ),* ) => { if (DEBUG) { print!($($x,)*) } }; }

#[derive(PartialEq,PartialOrd,Copy,Clone)]
enum Cell { Empty, Roped, Headed, Tailed, Start }
impl Default for Cell { fn default() -> Self { Cell::Empty } }

type At = (i32,i32);

fn point_add((x,y):At, (x2,y2):At) -> At { (x+x2,y+y2) }
fn point_sub((x,y):At, (x2,y2):At) -> At { (x-x2,y-y2) }
fn point_min((x,y):At, (x2,y2):At) -> At { (min(x,x2),min(y,y2)) }
fn point_max((x,y):At, (x2,y2):At) -> At { (max(x,x2),max(y,y2)) }
fn point_usize((x,y):At) -> (usize, usize) { (x as usize, y as usize) }

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	let invalide =  || { Error::new(ErrorKind::InvalidInput, "Unrecognized command") };
	let invalid =   || { Err(invalide()) };
	let invalide2 = || { Error::new(ErrorKind::InvalidInput, "Non-integer argument") };

	let mut map: HashMap<At, Cell> = Default::default();
	let mut map_write = |at:At,v:Cell| {
		let entry = map.entry(at);
		if match &entry { Entry::Vacant(_) => true, Entry::Occupied(v2) => v>*v2.get()} {
			entry.insert(v);
		}
	};

	// Scan file
	{
		let mut rope = [(0,0); ROPE_LEN];
		map_write(rope[0], Cell::Start);

		for line in lines {
			let line = line?;
			let (dir_str, num_str) = line.split_whitespace().collect_tuple().ok_or_else(invalide)?;
			let dir = 
				match dir_str {
					"U" => (0,-1), "D" => (0,1), "L" => (-1,0), "R" => (1,0),
					_ => return invalid()
				};
			let count = num_str.parse::<usize>().map_err(|_|invalide2())?;
			for _ in 0..count {
				rope[0] = point_add(rope[0], dir);
				map_write(rope[0], Cell::Headed);
d!("\t\t---");
				for idx in 0..(ROPE_LEN-1) {
					let (rope_left, rope_right) = rope.split_at_mut(idx+1);
					let (head_at, tail_at) = (&rope_left[idx], &mut rope_right[0]);
					let (xd,yd) = point_sub(*head_at,*tail_at);
					let (xda, yda) = (xd.abs(), yd.abs());
					let offset:At;
					if xda>1 || yda>1 {
						fn dir(i:i32) -> i32 {
							if i < -1 { return 1 }
							if i > 1  { return -1 }
							return 0
						}
						offset = (dir(xd),dir(yd));
						d!("head {:?} tail {:?} diff {},{} offset {:?}", *head_at, *tail_at, xd, yd, offset);
						*tail_at = point_add(*head_at, offset);
						d!("\ttail now: {:?}", *tail_at);
						map_write(*tail_at, if idx < ROPE_LEN-2 {Cell::Roped} else {Cell::Tailed} );
					} else {
						d!("head {:?} tail {:?} diff {},{}", *head_at, *tail_at, xd, yd);
					}
				}
			}
		}
	}

	if DEBUG {
		let (mut min, mut max) = ((i32::MAX, i32::MAX), (i32::MIN, i32::MIN));
		for k in map.keys() {
			min = point_min(min, *k);
			max = point_max(max, *k);
		}
		let (xs, ys) = point_add(point_sub(max,min), (1,1));
		let mut grid:Array2<Cell> = Array2::default((xs as usize, ys as usize)); //[[Cell::Empty; ys]; xs];
		for (k,v) in &map {
			let (k,v) = (*k, *v);
			grid[point_usize(point_sub(k, min))] = v;
		}
		for col in grid.axis_iter(Axis(1)) {
			for v in col {
				ds!("{}", match v {
					Cell::Empty => '.', Cell::Headed => '█', Cell::Roped => '░', Cell::Tailed => '◊', Cell::Start => 'S'
				})
			}
			d!("");
		}
		d!("");
	}

	for v in map.values() {
		let v = *v;
		if v == Cell::Tailed || v == Cell::Start { total += 1 }
	}

	// Final score
	println!("{}", total);

	Ok(())
}
