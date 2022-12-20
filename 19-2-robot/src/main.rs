// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fmt;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
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
type Decision = i32;

// Creates, (costs1, costs2)
type Cost = (Count,Cell);
type Robot = (Cost,Option<Cost>);
type RobotSpec = (Cell, Robot);

const TIME_LIMIT:Time = 32;
const MAXIMUM_DECISION:Decision = 26;

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
					if blueprints.len() >= 3 { break }
				}
			}
		}
		blueprints
	};

	#[cfg(debug_assertions)]
	#[derive(Debug)]
	struct HistoryNode {
		next: HistoryNodeRef,
		cell:Cell,
		at:Time
	}
	#[cfg(debug_assertions)]
	type HistoryNodeRef = Option<Rc<RefCell<HistoryNode>>>;

	// History, time, robot count, resource count, next
	#[derive(Clone)]
	struct Consider {
		#[cfg(debug_assertions)]
		history:HistoryNodeRef,
		time:Time,
		robots:[Count;4],
		cells:[Count;4],
		want:Option<Cell>
	}

	impl fmt::Debug for Consider {
	    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	        let mut f = f.debug_struct("Point");
	        let mut f = f
	         .field("time", &self.time)
	         .field("robots", &self.robots)
	         .field("cells", &self.cells)
	         .field("want", &self.want);
	        #[cfg(debug_assertions)] {
	        	f.field("history", &{
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
	         });
	        }
	        f.finish()
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
	fn best_consider(a:Option<Consider>, b:&Consider) -> (Option<Consider>, bool) {
		match a {
			None => (Some(b.clone()), true),
			Some(a) => if score_consider(b) > score_consider(&a) { (Some(b.clone()), true) } else { (Some(a), false) }
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

	let mut total:i64 = 1;

	// A "code" is a bitmasked sequence of branches. Each 2 bits represent a branch, lowest bits being least important
	for (idx,blueprint) in blueprints.iter().enumerate() {
		let mut winning_consider: Option<Consider> = Default::default();
		let mut code_at = 0;
		let mut decision_ceiling: Decision = 1;
		let mut raise_ceiling:bool = false;

		'construct: loop {
			let mut consider = Consider{
				#[cfg(debug_assertions)] history:None, 
				time:0, robots:[1,0,0,0], cells:[0;4], want:None
			};
			let mut decision: Decision = 0;
			'clock: loop {
				match consider.want {
					None => {
						let resource_idx = 3 - ((code_at >> (decision*2)) & 0x3);

						let ((_,cell1), cost2) = blueprint[resource_idx];
//println!("d {} i {} ({:?}), cell1 {:?}, robot {}, cost2 {:?} robot2 {:?}", decision, resource_idx, Cell::from_int(resource_idx as u8).unwrap(), cell1, consider.robots[cell1 as usize], cost2, cost2.map(|(_,cell)|consider.robots[cell as usize]));
						if consider.robots[cell1 as usize] > 0 && match cost2 { None => true, Some((_, cell)) => consider.robots[cell as usize] > 0 } {
							consider.want = Some(Cell::from_int(resource_idx as u8).unwrap());
							//#[cfg(debug_assertions)] { cell.history = history:consider.history.clone(); }
						} else {
							break 'clock;
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
							#[cfg(debug_assertions)] {
								consider.history = Some(Rc::new(RefCell::new(HistoryNode 
									{ cell:want, at:consider.time, 
										#[cfg(debug_assertions)]
										next: consider.history.clone()
									})));
							}
							consider.want = None;
							decision += 1;
let old_ceiling = decision_ceiling;
							decision_ceiling = cmp::max(decision_ceiling, decision+1);
if old_ceiling != decision_ceiling { println!("DECISION CEILING {} code {}", decision_ceiling, code_at) }
						}

						// Pass time
						consider.time += 1;
						for robot_idx in 0..4 {
							consider.cells[robot_idx] += original_robots[robot_idx];
						}
						if consider.time >= TIME_LIMIT { // TERMINATE
							let won:bool;
							(winning_consider, won) = best_consider(winning_consider, &consider);
							if won { println!("Better {:?}", consider) }
							break 'clock;
						}
						/*
						// Override?
						if let Some(best_consider) = best_consider {
							let best_score = score_consider(&best_consider);
							let score = score_consider(&consider);
						}
						*/
					}
				}
			}
//println!("CODE {}", code_at);
			code_at += 1;
			if code_at > (1 << (decision_ceiling*2)) || (decision+1)>=MAXIMUM_DECISION {
				break 'construct;
			}
		}

		println!("Blueprint {}, best: {:?}", idx+1, winning_consider);

		let score = score_consider(&winning_consider.unwrap());
		winning_blueprint = best_blueprint(winning_blueprint, (idx, score));
		total *= (score as i64);
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
