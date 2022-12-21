// Summary

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
	Literal(i32),
	Pair(Name,Op,Name)
}

enum Value {
	Literal(i32),
	Waiting(Name)
}

type EqMonkey = [Value;2];

enum MonkeyData {
	Eq(EqMonkey,Op),
	Literal(i32)
}
struct Monkey {
	data:MonkeyData,
	next:Option<Name>
}

const king:Name = *b"root"; // King of bongo bong

fn main() -> Result<(), Error> {
	let mut args = std::env::args().fuse();

	let (monkey_hash, monkey_queue) = { 
	    // Load file from command-line argument or (if -) stdin
		let filename = std::env::args().fuse().nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};

		let lines = input.lines();

		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, i32> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(std::str::from_utf8).convert(|x|x.parse::<i32>())
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

		let mut total: i64 = 0;

		let invalid = |s:&str| { return Err(Error::new(ErrorKind::InvalidInput, format!("Unrecognized line: '{}'", s))) };
		let invalid_duplicate = |n:Name| { return Err(Error::new(ErrorKind::InvalidInput, format!("Duplicate monkey: '{}'", std::str::from_utf8(&n[..]).unwrap()))) };
		let invalid_not_found = |n:Name| { return Err(Error::new(ErrorKind::InvalidInput, format!("Monkey not found: '{}'", std::str::from_utf8(&n[..]).unwrap()))) };
		let invalid_duplicate2 = |n:Name| { return Err(Error::new(ErrorKind::InvalidInput, format!("Duplicate monkey reference: '{}'", std::str::from_utf8(&n[..]).unwrap()))) };

		let mut monkey_hash: HashMap<Name, Monkey> = Default::default();
		let mut monkey_queue: Vec<Name> = Default::default();

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
					let mut monkey = Monkey {
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
						for n in [n1,n2] {
							match monkey_hash.entry(n) {
								Entry::Vacant(_) => return invalid_not_found(n),
								Entry::Occupied(mut m) =>
									if m.get().next.is_none() {
										m.get_mut().next = Some(name);
									} else {
										return invalid_duplicate2(n);
									}
							};
						}
					} else {
						// Literal monkey
						monkey_queue.push(name);
					}
					match monkey_hash.entry(name) {
						Entry::Vacant(e) => e.insert(monkey),
						Entry::Occupied(_) => return invalid_duplicate(name)
					};
				}
			}
		}
		(monkey_hash, monkey_queue)
	};

	// Final score
	//println!("{}", total);

	Ok(())
}
