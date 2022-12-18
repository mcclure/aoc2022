// Travelling salesman program but weird

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

const TIME_LIMIT:Time = 30;
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

	fn format_names(names: Vec<(String, Time, Weight)>) -> String {
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
		type NextPaths = Vec<(Vec<(String, Time, Weight)>, Vec<bool>, NodeIndex, Time, Weight)>;
		let mut paths: NextPaths = vec![(
				vec![(START_NAME.to_string(), 0, 0)],
				goals.iter().map(|_|false).collect(),
				start, 0, 0)];
		let (mut checked, mut timed_out, mut useless, mut best_weight) = (0,0,0,0); 
		while paths.len() > 0 {
			let mut next_paths: NextPaths = Default::default();

			for path in paths {
				for (goal_idx, &goal) in goals.iter().enumerate() {
					if path.1[goal_idx] { continue } // "visited" but don't clone
					let (mut history, mut visited, at, mut time, mut weight) = path.clone();

					time += routes[&(at, goal)];

					if time+1 >= TIME_LIMIT { timed_out += 1; continue } // > OR >= ??

					let (name, goal_weight) = &graph[goal];

					history.push((name.clone(), weight, time));
//println!("?? {}: {} {} ({})", name.clone(), time, TIME_LIMIT-time, goal_weight * (TIME_LIMIT-time));
					weight += goal_weight * (TIME_LIMIT-time-1); // I DON'T UNDERSTAND WHY -1
					time += 1;
					visited[goal_idx] = true;
					checked += 1;
					if weight > best_weight {
						best_weight = weight;
						println!("---\n{}, {}: {}", invert.paint(time.to_string()), inverty.paint(weight.to_string()), format_names(history.clone()));
					} else {
						useless += 1;
					}
					match report_progress { None => {},
						Some(report_progress) => if checked % report_progress == 0 {
							println!("checked {}, skipped {}, timeout {}", checked, useless, timed_out);
						}
					}
					next_paths.push((history, visited, goal, time, weight));
				}
			}

			paths = next_paths;
		}
	}
/*
	{
		let mut paths : Vec<(Vec<(String, Weight)>, Time, Weight)> = Default::default();


		let mut paths : Vec<(Vec<(String, Weight)>, Time, Weight)> = Default::default();

		let start_weighty = start_weight > 0;
		for n in 0..goals.len() {
			for mut path in goals.iter().permutations(n) {
				path.insert(0, &start);
				//println!("{:?}",p);
				let mut time = if start_weighty { 1 } else { 0 }; // Switch at start
				let mut total_weight = start_weight;
				let mut names: Vec<(String, Weight)> = vec![(START_NAME.to_string(), 0)];
				let mut timeout = false;
				for v in path.windows(2) {
					let (&from, &to) = (v[0], v[1]);
					let dijk = petgraph::algo::dijkstra::dijkstra(&graph, from, Some(to), |_|1);
					let time_cost = dijk[&to];

					time += time_cost; // Always 1?
					let (name, weight) = &graph[to];
					names.push((name.to_string(), time));
					if true { // "if to"
						time += 1;
						if time > TIME_LIMIT {
							timeout = true; break
						} else {
							total_weight += weight * (TIME_LIMIT-time+1);	
						}
					}
				}
				if !timeout {
					paths.push((names, time, total_weight));
				}
			}
		}
	}

	paths.sort_by(|a,b| match a.2.cmp(&b.2) {
		Ordering::Equal => a.1.cmp(&b.1), x => x
	} );

	fn format_names(names: Vec<(String, Time)>) -> String {
		let mut s:String = "[".to_string();
		for (n,(name, time)) in names.iter().enumerate() {
			if n>0 { s += ", " }
			write!(s, "{}:{}", time, name).unwrap();
		}
		s += "]";
		return s;
	}

	{
		use ansi_term::Style;
		use ansi_term::Colour::{Black, White, Yellow};
		let invert = Style::new().fg(Black).on(White);
		let inverty = Style::new().fg(Black).on(Yellow);

		for (names, time, weight) in paths {
			println!("---\n{}, {}: {}", invert.paint(time.to_string()), inverty.paint(weight.to_string()), format_names(names));
		}
	}

	// Final score
*/
	Ok(())
}
