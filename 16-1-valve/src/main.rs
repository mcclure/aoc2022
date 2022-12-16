// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::HashMap;
use either::Either;
use petgraph::graph::{NodeIndex, UnGraph};

fn main() -> Result<(), Error> {
	let (start, goals, graph) = {
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

		fn positive<'a>() -> Parser<'a, u8, i32> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<i32>())
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

		fn spec<'a>() -> Parser<'a, u8, ((&'a str, i32), Vec<&'a str>)> {
			not_whitespace_single().repeat(1..) * whitespace() * word() - whitespace_single() +
			(not_numeric().repeat(1..) * positive() - not_numeric()) +
			(none_of(b"v").repeat(1..) * sym(b'v') * not_whitespace_single().repeat(1..) * whitespace() *
			 list(word(), comma_separator())) - end()
		}

		let invalid = |s:&str| { Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };

		let mut graph: UnGraph<(String, i32), ()> = Default::default();

		let mut connect: Vec<(NodeIndex, Vec<String>)> = Default::default();
		let mut names: HashMap<String, NodeIndex> = Default::default();

		let mut start: Option<NodeIndex> = Default::default();
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
					if start.is_none() { start = Some(node)}
					if weight>0 { goals.push(node) }
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
			Some(start) => (start, goals, graph)
		}
	};

	// Final score
	println!("Cyclic? {}", petgraph::algo::is_cyclic_undirected(&graph));

	Ok(())
}
