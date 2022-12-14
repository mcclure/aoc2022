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
const DEBUG_FULL:bool = true;
const DEBUG_FAILURE_ONLY:bool = false;
const DEBUG_INLINE:bool = true;

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

		use ansi_term::Style;
		use ansi_term::Colour::{Yellow, Black, Fixed};

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
							fn i(depth:u64) { for _ in 0..depth {print!("  ")}}
							fn t(v: Ordering) -> &'static str { if v==Ordering::Greater {"👎🏻"} else {"👍"} }

							fn compare(a:Node, b:Node, depth:u64) -> Ordering {
								match (a,b) {
									(Node::Num(a), Node::Num(b)) => {
										let cmp = a.cmp(&b);
										if DEBUG_INLINE { i(depth); println!("{} < {}: {:?} {}", a, b, cmp, t(cmp)); }
										cmp
									},
									(Node::List(a), Node::List(b)) => {
										let mut all_equal = true;
										if DEBUG_INLINE { i(depth); println!("["); }
										for (a,b) in std::iter::zip(a.clone(),b.clone()) {
											let cmp = compare(a,b,depth+1);
											if cmp == Ordering::Greater {
												if DEBUG_INLINE { i(depth); println!("] {}", t(Ordering::Greater)); }
												return Ordering::Greater
											}
											if cmp == Ordering::Less { all_equal = false }
										}
										if all_equal {
											let cmp = a.len().cmp(&b.len());
											if DEBUG_INLINE { i(depth); println!("] {} len {} < len {}: {:?} {}", Style::new().fg(Yellow).paint("EQ"), b.len(), a.len(), cmp, t(cmp)); }
											cmp
										} else { 
											if DEBUG_INLINE { i(depth); println!("] {}{}", t(Ordering::Less), if a.len() != b.len() { " ..." } else {""}); }
											Ordering::Less
										}
									},
									(a@Node::Num(_), b@Node::List(_)) => compare(Node::List(vec![a]), b, depth),
									(a@Node::List(_), b@Node::Num(_)) => compare(a, Node::List(vec![b]), depth),
								}
							}

							let cmp = compare((*last_node).clone(), node.clone(),1);
							let correct = cmp != Ordering::Greater;

							if correct {
								total += idx_at;
							}

							if DEBUG && !(DEBUG_FAILURE_ONLY && correct) {
								fn printable_one_line(_l:Vec<Node>) -> bool {
									false
								/*	 // No good in two column mode.
									return l.iter().all(|x| 
										match x.clone() { Node::Num(_)=>true, 
											Node::List(l)=> {
												l.len()==0 || (l.len() == 1 && printable_one_line(l))
									}})
									*/
								}
								fn debug_tree(n:Node, depth:i64) -> String {
									let mut s:String = "".to_string();
									for _ in 0..depth { s += "    " }
									match n {
										Node::Num(n) => s += &format!("{}", n),
										Node::List(l) => {
											s += "[";
											if printable_one_line(l.clone()) {
												for (idx,i) in l.into_iter().enumerate() {
													if idx>0 { s += ", " }
													s += &debug_tree(i, -1);
												}
											}  else {
												for (idx,i) in l.into_iter().enumerate() {
													s += if idx>0 { ", \n" } else { "\n" };
													s += &debug_tree(i, depth+1);
												}
											}
											s += "]";
										}
									}
									if depth==0 { s += "" }
									s
								}

								println!("COMPARE {}", idx_at);
								if DEBUG_FULL {
									let gray1 = Style::new().on(Fixed(236));
									let gray2 = Style::new().on(Fixed(237));

									let left = debug_tree((*last_node).clone(), 0);
									let right = debug_tree(node, 0);
									fn str_width(s:&str) -> usize { // Width of longest line in s
										let mut x = 0;
										for line in s.lines() {
											x = max(x, line.len());
										}
										x
									}
									let left_width = str_width(&left);
									let right_width = str_width(&right);
									for x in left.lines().zip_longest(right.lines()) {
										let (left, right) = match x {
											EitherOrBoth::Both(left, right) => (left, right),
											EitherOrBoth::Left(left) => (left, ""),
											EitherOrBoth::Right(right) => ("", right)
										};
										println!("{}{}{}{}", gray1.paint(left), gray1.paint(" ".repeat(left_width - left.len())),
											gray2.paint(right), gray2.paint(" ".repeat(right_width - right.len())));
									}
									println!("{}{}", gray1.paint(" ".repeat(left_width)), gray2.paint(" ".repeat(right_width)));
								}
								println!("{} ({:?})\n", if correct {Style::new().paint("*** YES")} else {Style::new().fg(Black).on(Fixed(9)).paint("    NO ")}, cmp);
							} else if DEBUG_INLINE { println!("") }


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
