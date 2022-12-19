// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fmt;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use either::Either;
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Debug,Copy,Clone,PartialEq,IntEnum)]
enum Cell {
	Ore = 0,
	Clay = 1,
	Obsidian = 2,
	Geode = 3
}
impl Default for Cell { fn default() -> Self { Cell::Ore } }

type Count = i32;
type Time = i32;

// Creates, (costs1, costs2)
type Cost = (Count,Cell);
type Robot = (Cost,Option<Cost>);
type RobotSpec = (Cell, Robot);

const TIME_LIMIT:Time = 24;

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();

	let blueprints = {
	    // Load file from command-line argument or (if -) stdin
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, Count> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<Count>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn prelude<'a>() -> Parser<'a, u8, ()> {
			none_of(b":").repeat(0..) * sym(b':').discard()
		}

		fn mineral<'a>() -> Parser<'a, u8, Cell> {
			seq(b"ore").map(|_|Cell::Ore) |
			seq(b"clay").map(|_|Cell::Clay) |
			seq(b"obsidian").map(|_|Cell::Obsidian) |
			seq(b"geode").map(|_|Cell::Geode)
		}

		fn minerals<'a>() -> Parser<'a, u8, (Count, Cell)> {
			(positive() - whitespace()) +
			mineral()
		}

		fn robot<'a>() -> Parser<'a, u8, RobotSpec> {
			(seq(b"Each ") * mineral() - seq (b" robot costs")) +
			(whitespace() * minerals()
			 + (whitespace() * seq(b"and") * whitespace() * minerals()).opt()
			 - sym(b'.'))
		}

		fn statement<'a>() -> Parser<'a, u8, Vec<RobotSpec>> {
			prelude() * whitespace() * list(robot(), whitespace()) - end()
		}

		let invalid = |s:&str| { return Err(Error::new(ErrorKind::InvalidInput, format!("Unrecognized line: '{}'", s))) };

		let mut blueprints: Vec<[Robot;4]> = Default::default();

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let parsed = (statement()).parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok(robot_specs) => {
					let mut robots: [Robot;4] = Default::default();
					let mut robots_seen: [bool;4] = [false;4];
					for (cell, robot) in robot_specs {
						robots[cell as usize] = robot;
						if robots_seen[cell as usize] {
							return Err(Error::new(ErrorKind::InvalidInput, format!("Line has duplicate robots: '{}'", line)))
						}
						if let ((_, cell1),Some((_,cell2))) = robot { if cell1 == cell2 {
							return Err(Error::new(ErrorKind::InvalidInput, format!("Robot has duplicate resource: '{}'", line)))
						} }
						robots_seen[cell as usize] = true;
					}
					if !robots_seen.iter().all(|x|*x) {
						return Err(Error::new(ErrorKind::InvalidInput, format!("Line is missing robots: '{}'", line)))
					}
					blueprints.push(robots);
				}
			}
		}
		blueprints
	};

	#[derive(Debug)]
	struct HistoryNode {
		next: HistoryNodeRef,
		cell:Cell,
		at:Time
	}
	type HistoryNodeRef = Option<Rc<RefCell<HistoryNode>>>;

	// History, time, robot count, resource count, next
	struct Consider {
		history:HistoryNodeRef,
		time:Time,
		robots:[Count;4],
		cells:[Count;4],
		want:Option<Cell>
	}

	impl fmt::Debug for Consider {
	    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	        f.debug_struct("Point")
	         .field("time", &self.time)
	         .field("robots", &self.robots)
	         .field("cells", &self.cells)
	         .field("want", &self.want)
	         .field("history", &{
	         	let mut s:String = "]".to_string();
	         	let mut node = self.history.clone();
	         	while !node.is_none() {
	         		let n2 = node.unwrap();
	         		let n2 = n2.borrow_mut();
	         		s = format!("[{:?}@{}],{}", n2.cell, n2.at+1, s); // NOTICE TIME INCREMENT
	         		node = n2.next.clone();
	         	}
	         	s = "[".to_string() + &s;
	         	s
	         })
	         .finish()
	    }
	}

	// id, geode count
	type BlueprintRecord = (usize,Count);

	let mut winning_blueprint: Option<BlueprintRecord> = None;
	fn best_blueprint(a:Option<BlueprintRecord>, b:BlueprintRecord) -> Option<BlueprintRecord> {
		match a {
			None => Some(b),
			Some(a) => Some(if b.1 > a.1 { b } else { a })
		}
	}
	fn score_consider(a:&Consider) -> Count {
		a.cells[3]
	}
	fn best_consider(a:Option<Consider>, b:Consider) -> Option<Consider> {
		match a {
			None => Some(b),
			Some(a) => Some(if score_consider(&b) > score_consider(&a) { b } else { a })
		}
	}

	fn robot_can(cost:Cost, count:[Count;4]) -> bool {
		let (cost, idx) = cost;
		count[idx as usize] >= cost as Count
	}

	fn robot_deplete(cost:Cost, count:&mut [Count;4]) {
		let (cost, idx) = cost;
		count[idx as usize] -= cost as Count;
	}

	let mut total:i64 = 0;

	for (idx,blueprint) in blueprints.iter().enumerate() {
		let mut winning_consider: Option<Consider> = Default::default();

		let mut next_considers = vec![Consider{history:None, time:0, robots:[1,0,0,0], cells:[0;4], want:None}];
		while !next_considers.is_empty() {
			let considers = std::mem::take(&mut next_considers);
			for mut consider in considers {
				'considering: loop {
					match consider.want {
						None => {
							// Branch
							let mut need:Vec<Consider> = Default::default();
							for resource_idx in (0..4).rev() { // Check which robots can form currently
								let ((_,cell1), cost2) = blueprint[resource_idx];
								if consider.robots[cell1 as usize] > 0 && match cost2 { None => true, Some((_, cell)) => consider.robots[cell as usize] > 0 } {
									need.push(Consider{want:Some(Cell::from_int(resource_idx as u8).unwrap()), history:consider.history.clone(), ..consider})
								}
							}

							let mut need_iter = need.into_iter();
							match need_iter.next() {
								None => panic!("Blueprint {}: Should always have 1 ore robot", idx+1),
								Some(first) => {
									consider = first;
									next_considers.extend(need_iter);
								}
							}
						},
						Some(want) => {
							// Core logic here
							let original_robots = consider.robots;

							// Spend money
							let (cost1, cost2) = blueprint[want as usize];
							if consider.time+1 < TIME_LIMIT // EG, don't bother buying robots in the last second
							&& robot_can(cost1, consider.cells)
							&& match cost2 { None=>true, Some(cost)=>robot_can(cost, consider.cells) } {
								robot_deplete(cost1, &mut consider.cells);
								if let Some(cost) = cost2 { robot_deplete(cost, &mut consider.cells) }
								consider.robots[want as usize] += 1;
								consider.history = Some(Rc::new(RefCell::new(HistoryNode { cell:want, at:consider.time, next: consider.history.clone() })));
								consider.want = None;
							}

							// Pass time
							consider.time += 1;
							for robot_idx in 0..4 {
								consider.cells[robot_idx] += original_robots[robot_idx];
							}
							if consider.time >= TIME_LIMIT { // TERMINATE
								winning_consider = best_consider(winning_consider, consider);
								break 'considering;
							}
						}
					}
				}
			}
		}

		println!("Blueprint {}, best: {:?}", idx+1, winning_consider);

		let score = score_consider(&winning_consider.unwrap());
		winning_blueprint = best_blueprint(winning_blueprint, (idx, score));
		total += ((idx as i64)+1)*(score as i64);
	}

	// Final score
	match winning_blueprint {
		None => return Err(Error::new(ErrorKind::InvalidInput, "No blueprints?")),
		Some((id, count)) =>
			println!("Best {} with {}", id+1, count)
	}

	println!("{}", total);

	Ok(())
}
