// Calculate the result of a system of equations

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::{HashMap, hash_map::Entry};
use either::Either;

#[derive(Debug,Copy,Clone)]
enum Op {
	Plus,
	Minus,
	Times,
	Divide
}

type Name = [u8;4];

#[derive(Debug,Copy,Clone)]
enum Chant {
	Literal(i64),
	Pair(Name,Op,Name)
}

#[derive(Debug)]
enum Value {
	Literal(i64),
	Waiting(Name)
}

type EqMonkey = [Value;2];

#[derive(Debug)]
enum MonkeyData {
	Eq(EqMonkey,Op),
	Literal(i64)
}
struct Monkey {
	data:MonkeyData,
	next:Option<Name>
}

const KING:Name = *b"root"; // King of bongo bong

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();

	fn monkey_name(n:&Name) -> &str { std::str::from_utf8(&n[..]).unwrap() }

	let (mut monkey_hash, mut monkey_queue) = { 
	    // Load file from command-line argument or (if -) stdin
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, i64> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<i64>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn letter<'a>() -> Parser<'a, u8, u8> {
			one_of(b"abcdefghijklmnopqrstuvwxyz") // I really wish I could make this library work with Unicode
		}

		fn word<'a>() -> Parser<'a, u8, Name> {
			letter().repeat(4).map(|x|x.try_into().unwrap()) // repeat ensures exactly 4
		}

		fn op<'a>() -> Parser<'a, u8, Op> {
			sym(b'+').map(|_|Op::Plus) |
			sym(b'-').map(|_|Op::Minus) |
			sym(b'*').map(|_|Op::Times) |
			sym(b'/').map(|_|Op::Divide)
		}

		fn statement<'a>() -> Parser<'a, u8, (Name, Chant)> {
			(word() - sym(b':') - whitespace()) +
			(
				(positive().map(|x|Chant::Literal(x)))
			  | ((word() - whitespace()) + (op() - whitespace()) + word() - whitespace())
			  		.map(|((a, b), c)|{Chant::Pair(a,b,c)})
			)- end()
		}

		let invalid = |s:&str| { return Err(Error::new(ErrorKind::InvalidInput, format!("Unrecognized line: '{}'", s))) };
		let invalid_duplicate = |n:Name| { return Err(Error::new(ErrorKind::InvalidInput, format!("Duplicate monkey: '{}'", monkey_name(&n)))) };
		let invalid_not_found = |n:Name| { return Err(Error::new(ErrorKind::InvalidInput, format!("Monkey not found: '{}'", monkey_name(&n)))) };
		let invalid_duplicate2 = |n:Name| { return Err(Error::new(ErrorKind::InvalidInput, format!("Duplicate monkey reference: '{}'", monkey_name(&n)))) };

		let mut monkey_hash: HashMap<Name, Monkey> = Default::default();
		let mut monkey_queue: Vec<Name> = Default::default();
		let mut monkey_next: Vec<(Name,Name)> = Default::default();

		// Scan file
		for line in lines {
			let line = line?;
			let line = line.trim();
			if line.is_empty() { continue }

			let parsed = statement().parse(line.as_bytes());
			match parsed {
				Err(_) => return invalid(line),
				Ok((name, chant)) => {
					let mut notify: Option<(Name,Name)> = Default::default();
					let monkey = Monkey {
						data:match chant {
							Chant::Literal(i)=>MonkeyData::Literal(i),
							Chant::Pair(n1,op,n2)=> {
								notify = Some((n1,n2));
								MonkeyData::Eq([Value::Waiting(n1),Value::Waiting(n2)],op)
							}
						},
						next:None
					};
					if let Some((n1,n2)) = notify {
						monkey_next.push((n1,name));
						monkey_next.push((n2,name));
					} else {
						// Literal monkey must yell
						monkey_queue.push(name);
					}
					match monkey_hash.entry(name) {
						Entry::Vacant(e) => e.insert(monkey),
						Entry::Occupied(_) => return invalid_duplicate(name)
					};
				}
			}
		}

		for (from,to) in monkey_next {
			//println!("From {} to {}", monkey_name(&from), monkey_name(&to));
			match monkey_hash.entry(from) {
				Entry::Vacant(_) => return invalid_not_found(from),
				Entry::Occupied(mut m) =>
					if m.get().next.is_none() {
						m.get_mut().next = Some(to);
					} else {
						return invalid_duplicate2(from);
					}
			};
		}
		(monkey_hash, monkey_queue)
	};

	while !monkey_queue.is_empty() {
		for name in std::mem::take(&mut monkey_queue) {
			let value = match monkey_hash[&name].data {
				MonkeyData::Literal(i) => { i },
				MonkeyData::Eq([Value::Literal(i1),Value::Literal(i2)], op) => {
					/*{
						let ch = match op { Op::Plus => '+', Op::Minus => '-', Op::Times => '*', Op::Divide => '/' };
						println!("{} = {} {} {}", monkey_name(&name), i1, ch, i2);
					}*/
					match op {
						Op::Plus  => { i1 + i2 },
						Op::Minus => { i1 - i2 }
						Op::Times => { i1 * i2 }
						Op::Divide => {
							if i1 == 0 || i2 == 0 { return Err(Error::new(ErrorKind::InvalidInput, "Divide by zero??")) }
							i1 / i2
						}
					}
				},
				_ => panic!("Bad queue")
			};
			//println!("\t = {}", value);
			if name == KING { // DONE
				println!("{}", value);
				return Ok(())
			}
			//println!("Check {}, {:?}", monkey_name(&name), monkey_hash[&name].next);
			if let Some(next) = monkey_hash[&name].next {
				let next_monkey = monkey_hash.get_mut(&next).unwrap();
				match &mut next_monkey.data {
					MonkeyData::Eq(values,_) => {
						let mut found = false;
						for v in values.iter_mut() {
							if let Value::Waiting(name2) = &v {
								if name == *name2 {
									*v = Value::Literal(value);
									found = true;
								}
							}
						}
						if !found { panic!("Next monkey wasn't waiting for us") }
						let mut ready = true;
						for v in values.iter() {
							if let Value::Waiting(_) = v {
								ready = false;
							}
						}
						if ready {
							monkey_queue.push(next)
						}
					}
					_ => panic!("Literal monkey can't be next")
				}
			} else {
				return Err(Error::new(ErrorKind::InvalidInput, format!("No next monkey for {}", monkey_name(&name))));
			}
		}
	}

	Err(Error::new(ErrorKind::InvalidInput, "Monkeys didn't form a pyramid"))
}
