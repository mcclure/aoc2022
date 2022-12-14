// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::cmp::{Ordering, max};
use either::Either;
use itertools::{EitherOrBoth, Itertools};

#[derive(Debug, Clone)]
enum Node {
	Num(u64),
	List(Vec<Node>)
}

const DEBUG:bool = true;

fn main() -> Result<(), Error> {
	let mut packets: Vec<Node> = Default::default();

	// Scan file
	{
	    // Load file from command-line argument or (if none) stdin
		let filename = std::env::args().fuse().nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
			None => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		let invalid = |s:&str| { return Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, u64> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(|s|String::from_iter(s.iter().map(|s|*s as char)).parse::<u64>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn comma_separator<'a>() -> Parser<'a, u8, ()> {
			(whitespace() * sym(b',') * whitespace()).discard() 
		}

		fn comma_separated_list<'a>() -> Parser<'a, u8, Node> {
			sym(b'[') * whitespace() * (
				list(
					call(comma_separated_list) |
					positive().map(|s|Node::Num(s))
				, comma_separator()).map(|s|Node::List(s))
			) - whitespace() - sym(b']')
		}

		let mut last: Option<Node> = Default::default();
		let mut idx_at = 1; // 1-index

		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let parsed = (comma_separated_list() - end()).parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok(node) => packets.push(node)
			}
		}
	}

	// Sort packets
	{
		packets.push(Node::List(vec![Node::List(vec![Node::Num(2)])]));
		packets.push(Node::List(vec![Node::List(vec![Node::Num(6)])]));

		fn compare(a:&Node, b:&Node) -> Ordering {
			match (a,b) {
				(&Node::Num(a), &Node::Num(b)) => {
					let cmp = a.cmp(&b);
					cmp
				},
				(&Node::List(ref a), &Node::List(ref b)) => {
					for (a,b) in std::iter::zip(a.clone(),b.clone()) {
						let cmp = compare(&a,&b);
						if cmp != Ordering::Equal {
							return cmp
						}
					}

					let cmp = a.len().cmp(&b.len());
					cmp
				},
				(a@&Node::Num(_), b@&Node::List(_)) => compare(&Node::List(vec![a.clone()]), b),
				(a@&Node::List(_), b@&Node::Num(_)) => compare(a, &Node::List(vec![b.clone()])),
			}
		}

		packets.sort_by(compare);

		for p in packets { println!("{:?}", p); }
	}

	// Final score
	//println!("{}", total);

	Ok(())
}
