// Travelling salesman program but weird alo there's an elephant
// Second argument is printout debug

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::fmt::Write;
use either::Either;
use petgraph::graph::{NodeIndex, UnGraph};
use itertools::Itertools;

type Weight = i32;
type Time = Weight;

const TIME_LIMIT:Time = 26;
const START_NAME: &str = "AA";

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();
	let filename = args.nth(1);

	let (start, start_weight, goals, graph) = {
	    // Load file from command-line argument or (if -) stdin
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, Weight> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<Weight>())
		}

		fn not_numeric<'a>() -> Parser<'a, u8, ()> {
			none_of(b"0123456789").discard()
		}

		fn not_whitespace_single<'a>() -> Parser<'a, u8, ()> {
			none_of(b" \t").discard()
		}

		fn whitespace_single<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").discard()
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			whitespace_single().repeat(0..).discard()
		}

		fn word<'a>() -> Parser<'a, u8, &'a str> {
			none_of(b" \t,").repeat(1..).collect().map(std::str::from_utf8).map(|x|x.unwrap())
		}
 
		fn comma_separator<'a>() -> Parser<'a, u8, ()> {
			(whitespace() * sym(b',') * whitespace()).discard() 
		}

		fn spec<'a>() -> Parser<'a, u8, ((&'a str, Weight), Vec<&'a str>)> {
			not_whitespace_single().repeat(1..) * whitespace() * word() - whitespace_single() +
			(not_numeric().repeat(1..) * positive() - not_numeric()) +
			(none_of(b"v").repeat(1..) * sym(b'v') * not_whitespace_single().repeat(1..) * whitespace() *
			 list(word(), comma_separator())) - end()
		}

		let invalid = |s:&str| { Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };

		let mut graph: UnGraph<(String, Weight), ()> = Default::default();

		let mut connect: Vec<(NodeIndex, Vec<String>)> = Default::default();
		let mut names: HashMap<String, NodeIndex> = Default::default();

		let mut start: Option<(NodeIndex, Weight)> = Default::default();
		let mut goals: Vec<NodeIndex> = Default::default();

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let parsed = spec().parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok(((name, weight), connections)) => {
					let name = name.to_string();
					let node = graph.add_node((name.clone(), weight));
					if start.is_none() && name == START_NAME {
						start = Some((node, weight))
					}
					else if weight>0 { goals.push(node) }
					connect.push((node, connections.into_iter().map(|x|x.to_string()).collect()));
					names.insert(name, node);
				}
			}
		}

		for (node, to) in connect {
			graph.extend_with_edges(to.into_iter().map(|n2|(node, names[&n2])).collect::<Vec<(NodeIndex, NodeIndex)>>());
		}

		match start {
			None => return Err(Error::new(ErrorKind::InvalidInput, "No nodes?")),
			Some((start, start_weight)) => (start, start_weight, goals, graph)
		}
	};

	let routes = {
		let mut routes: HashMap<(NodeIndex, NodeIndex), Time> = Default::default();
		let mut origins = goals.clone();
		origins.insert(0, start);
		let mut is_goal: HashSet<NodeIndex> = Default::default();
		for &goal in goals.iter() { is_goal.insert(goal); };
		for origin in origins {
			let dijk = petgraph::algo::dijkstra::dijkstra(&graph, origin, None, |_|1);
			for (destination, time) in dijk {
				if is_goal.contains(&destination) {
					routes.insert((origin, destination), time);
				}
			}
		}
		routes
	};

	let report_progress = match args.next() {
		Some(x) => Some(x.parse::<u64>().map_err(|_|Error::new(ErrorKind::InvalidInput, "Argument 2 must be number"))?),
		None => None
	};

	type Player = usize;

	fn format_names(names: Vec<(String, Time, Player)>) -> String {
		let mut s:String = "[".to_string();
		for (n,(name, time, weight)) in names.iter().enumerate() {
			if n>0 { s += ", " }
			write!(s, "{}[{},{}]", name, time, weight).unwrap();
		}
		s += "]";
		return s;
	}

	// Path, time, weight
	{
		use ansi_term::Style;
		use ansi_term::Colour::{Black, White, Yellow};
		let invert = Style::new().fg(Black).on(White);
		let inverty = Style::new().fg(Black).on(Yellow);

		// Note first Vec can lose time and weight in non-debug scenario
		type PathHistory = Vec<(String, Time, Weight)>;
		const PLAYERS:usize = 2;
		type NextPaths = Vec<(Vec<(String, Time, Player)>, Vec<bool>, [NodeIndex; PLAYERS], [Time; PLAYERS], Weight)>;
		let mut paths: [NextPaths; TIME_LIMIT as usize] = Default::default();
		paths[0].push((
				vec![(START_NAME.to_string(), 0, 0)],
				goals.iter().map(|_|false).collect(),
				[start, start], [0,0], 0
		));
		let (mut checked, mut timed_out, mut useless, mut best_weight) = (0,0,0,0);
		for t in 0..(TIME_LIMIT as usize) {
			while paths[t].len() > 0 {
				let mut pass_paths = std::mem::take(&mut paths[t]);

				for path in pass_paths {
					for (goal_idx, &goal) in goals.iter().enumerate() {
						if path.1[goal_idx] { continue } // "visited" but don't clone
						let (mut history, mut visited, mut at, mut time, mut weight) = path.clone();

						let player:Player = if time[1] < time[0] { 1 } else { 0 }; 

						time[player] += routes[&(at[player], goal)];

						if time[player]+1 >= TIME_LIMIT { timed_out += 1; continue } // > OR >= ??

						let next_time = time[if time[1] < time[0] { 1 } else { 0 }] as usize;

						let (name, goal_weight) = &graph[goal];

						history.push((name.clone(), time[player], player));
	//println!("?? {}: {} {} ({})", name.clone(), time, TIME_LIMIT-time, goal_weight * (TIME_LIMIT-time));
						weight += goal_weight * (TIME_LIMIT-time[player]-1); // I DON'T UNDERSTAND WHY -1
						time[player] += 1;
						visited[goal_idx] = true;
						checked += 1;
						if weight > best_weight {
							best_weight = weight;
							println!("---\n{}. {}, {}: {}", player, invert.paint(time[player].to_string()), inverty.paint(weight.to_string()), format_names(history.clone()));
						} else {
							useless += 1;
						}
						match report_progress { None => {},
							Some(report_progress) => if checked % report_progress == 0 {
								println!("checked {}, skipped {}, timeout {}", checked, useless, timed_out);
							}
						}
						at[player] = goal;
						paths[next_time].push((history, visited, at, time, weight));
					}
				}
			}
		}
	}

	Ok(())
}
