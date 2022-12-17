// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt::Write;
use either::Either;
use petgraph::graph::{NodeIndex, UnGraph};
use itertools::Itertools;

type Weight = i32;

const TIME_LIMIT:Weight = 30;
const START_NAME: &str = "AA";

fn main() -> Result<(), Error> {
	let (start, start_weight, goals, graph) = {
	    // Load file from command-line argument or (if -) stdin
		let filename = std::env::args().fuse().nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		let mut total: i64 = 0;

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

	// Path, time, weight
	let mut paths : Vec<(Vec<(String, Weight)>, Weight, Weight)> = Default::default();
	{
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

	fn format_names(names: Vec<(String, Weight)>) -> String {
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

	Ok(())
}
