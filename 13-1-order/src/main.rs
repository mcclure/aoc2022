// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::cmp::Ordering;
use either::Either;

#[derive(Debug, Clone)]
enum Node {
	Num(u64),
	List(Vec<Node>)
}

const DEBUG:bool = true;
const DEBUG_FAILURE_ONLY:bool = true;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let lines = input.lines();

	let mut total: i64 = 0;

	let invalid = |s:&str| { return Err(Error::new(ErrorKind::InvalidInput, format!("Line not understood: '{}'", s))) };
	let invalid2 = || { return Err(Error::new(ErrorKind::InvalidInput, "Odd number of lines")) };

	// Scan file
	{
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
				Ok(node) => {
					match last {
						None => { last = Some(node); }
						Some(ref last_node) => {
							// Actual program lives here

							fn compare(a:Node, b:Node) -> Ordering {
								match (a,b) {
									(Node::Num(a), Node::Num(b)) => a.cmp(&b),
									(Node::List(a), Node::List(b)) => {
										let mut all_equal = true;
										for (a,b) in std::iter::zip(a.clone(),b.clone()) {
											let cmp = compare(a,b);
											if cmp == Ordering::Greater { return Ordering::Greater }
											if cmp == Ordering::Less { all_equal = false }
										}
										if all_equal { a.len().cmp(&b.len()) } else { Ordering::Less }
									},
									(a@Node::Num(_), b@Node::List(_)) => compare(Node::List(vec![a]), b),
									(a@Node::List(_), b@Node::Num(_)) => compare(a, Node::List(vec![b])),
								}
							}

							let cmp = compare((*last_node).clone(), node.clone());
							let correct = cmp != Ordering::Greater;

							if correct {
								total += idx_at;
							}

							if DEBUG && !(DEBUG_FAILURE_ONLY && correct) {
								fn printable_one_line(l:Vec<Node>) -> bool {
									return l.iter().all(|x| 
										match x.clone() { Node::Num(_)=>true, 
											Node::List(l)=> {
												l.len()==0 || (l.len() == 1 && printable_one_line(l))
									}})
								}
								fn debug_tree(n:Node, depth:i64) {
									for _ in 0..depth { print!("\t") }
									match n {
										Node::Num(n) => print!("{}", n),
										Node::List(l) => {
											print!("[");
											if printable_one_line(l.clone()) {
												for (idx,i) in l.into_iter().enumerate() {
													if idx>0 { print!(", ") }
													debug_tree(i, -1);
												}
											}  else {
												for (idx,i) in l.into_iter().enumerate() {
													if idx>0 { println!(", ") } else { println!("") }
													debug_tree(i, depth+1);
												}
											}
											print!("]");
										}
									}
									if depth==0 { println!("") }
								}

								println!("COMPARE {}", idx_at);
								debug_tree((*last_node).clone(), 0);
								debug_tree(node, 0);
								println!("{} ({:?})\n", if correct {"*** YES"} else {"    NO "}, cmp);
							}

							idx_at += 1;
							last = None;
						}
					}
				}
			}
		}

		if !last.is_none() {
			return invalid2();
		}
	}

	// Final score
	println!("{}", total);

	Ok(())
}
