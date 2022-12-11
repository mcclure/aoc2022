// Parses a series of monkey descriptions.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

#[derive(Debug)]
enum Op {
	Plus, Times
}

#[derive(Debug)]
enum Operand {
	Old,
	Literal(u64)
}

struct Monkey {
	holding:Vec<u64>,
	operation:(Op, Operand),
	divisible:u64,
	if_true:usize,
	if_false:usize,
	inspections:u64
}

const MONKEY_ROUNDS:u64 = 20;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	// Filter input to remove blank lines.
	let mut lines = input.lines().filter(|x|match x { Ok(x) => !x.is_empty(), _ => true }).peekable();

	let mut monkeys:Vec<Monkey> = Default::default();

	{
		use pom::parser::*;

		fn positive<'a>() -> Parser<'a, u8, u64> {
			let integer = (one_of(b"123456789") - one_of(b"0123456789").repeat(0..)) | sym(b'0');
			integer.collect().convert(|s|String::from_iter(s.iter().map(|s|*s as char)).parse::<u64>())
		}

		fn whitespace<'a>() -> Parser<'a, u8, ()> {
			one_of(b" \t").repeat(0..).discard()
		}

		fn not_number<'a>() -> Parser<'a, u8, ()> {
			none_of(b"0123456789").repeat(0..).discard()
		}

		fn comma_separator<'a>() -> Parser<'a, u8, ()> {
			(whitespace() * sym(b',') * whitespace()).discard() 
		}

		fn comma_separated_positive<'a>() -> Parser<'a, u8, Vec<u64>> {
			list(positive(), comma_separator())
		}

		fn ends_with_positive<'a>() -> Parser<'a, u8, u64> { // Matches any line ending with a integer
			not_number() * positive() - whitespace()
		}

		fn ends_with_positive_list<'a>() -> Parser<'a, u8, Vec<u64>> { // Matches any line ending with
			not_number() * comma_separated_positive() - whitespace()   // a comma-separated list of ints
		}

		fn ends_with_operation<'a>() -> Parser<'a, u8, (Op, Operand)> {
			none_of(b"*+").repeat(0..) * (
				( (sym(b'+').map(|_|Op::Plus) | sym(b'*').map(|_|Op::Times)) - whitespace() ) + 
				( seq(b"old").map(|_|Operand::Old) | positive().map(Operand::Literal))
			)
		}

		let invalide = |s| { Error::new(ErrorKind::InvalidInput, format!("Unrecognized line '{}'", s)) };
		fn next<I, T:Iterator<Item = Result<I, Error>>>(l:&mut T) -> Result<I, Error> { match (*l).next() { Some(x) => x, None => Err(Error::new(ErrorKind::InvalidInput, "Incomplete monkey")) } }

		#[inline] fn as_usize(u:u64) -> Result<usize, Error> {
			TryInto::<usize>::try_into(u).map_err(|_|Error::new(ErrorKind::InvalidInput, "Too many monkeys"))
		}

		// Scan file
		loop {
			let _ = next(&mut lines)?; // Discard monkey number
			let monkey = Monkey {
				holding: {
					let temp = next(&mut lines)?;
					let temp2 = temp.clone();
					let temp = ends_with_positive_list().parse(temp.as_bytes()).map_err(|_|invalide(temp2))?;
					temp
				},
				operation: {
					let temp = next(&mut lines)?;
					let temp2 = temp.clone();
					let temp = ends_with_operation().parse(temp.as_bytes()).map_err(|_|invalide(temp2))?;
					temp
				},
				divisible: {
					let temp = next(&mut lines)?;
					let temp2 = temp.clone();
					let temp = ends_with_positive().parse(temp.as_bytes()).map_err(|_|invalide(temp2))?;
					temp
				},
				if_true: as_usize({
					let temp = next(&mut lines)?;
					let temp2 = temp.clone();
					let temp = ends_with_positive().parse(temp.as_bytes()).map_err(|_|invalide(temp2))?;
					temp
				})?,
				if_false: as_usize({
					let temp = next(&mut lines)?;
					let temp2 = temp.clone();
					let temp = ends_with_positive().parse(temp.as_bytes()).map_err(|_|invalide(temp2))?;
					temp
				})?,
				inspections: 0
			};

			monkeys.push(monkey);

			// If EOF occurs at this known place, break cleanly.
			if let None = lines.peek() { break }
		}
	}

	for _ in 0..MONKEY_ROUNDS {
		for monkey_idx in 0..monkeys.len() {
			let (mut under, mut monkey) = monkeys.split_at_mut(monkey_idx);
			let (mut monkey, mut over) = monkey.split_at_mut(1); // Notice monkey not monkeys
			let mut monkey = &mut monkey[0];

			/*
			let mut other_monkey = |other_idx:usize|->&mut Monkey {
				if other_idx<monkey_idx { &mut under[other_idx] }
				else if other_idx>monkey_idx { &mut over[other_idx-monkey_idx-1 ] }
				else { panic!("Impossible error")}
			};
			*/

			let mut inspect_idx = 0;
			while inspect_idx < monkey.holding.len() {
				// FIRST increment worry
				{
					let (op, operand) = &monkey.operation;
					let operand = match operand {
						Operand::Old => monkey.holding[inspect_idx],
						Operand::Literal(n) => *n
					};
					match op {
						Op::Plus  => { monkey.holding[inspect_idx] += operand },
						Op::Times => { monkey.holding[inspect_idx] *= operand }
					};
				}
				// THEN calm down
				monkey.holding[inspect_idx] /= 3;
				// THEN throw
				let other_monkey_idx = if monkey.holding[inspect_idx] % monkey.divisible == 0 {
					monkey.if_true
				} else {
					monkey.if_false
				};
				if other_monkey_idx != monkey_idx {
					let throw = monkey.holding.remove(inspect_idx);
					let other_monkey = 
						if other_monkey_idx<monkey_idx { &mut under[other_monkey_idx] }
						else if other_monkey_idx>monkey_idx { &mut over[other_monkey_idx-monkey_idx-1 ] }
						else { panic!("Impossible error")};
					other_monkey.holding.push(throw); // WAIT THIS IS WRONG
				} else {
					// There's nothing semantically wrong with this (you could just move it to the end of self)
					// But it could too easily lead to infinite loops
					return Err(Error::new(ErrorKind::InvalidInput, "Assuming a monkey cannot throw to itself"))
					//inspect_idx += 1;
				}
				monkey.inspections += 1;
			}
		}
	}

	monkeys.sort_unstable_by_key(|x|std::cmp::Reverse(x.inspections)); // i64::MAX-

	{
		if monkeys.len() < 2 { return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two monkeys")) }
		let total = monkeys[0].inspections * monkeys[1].inspections;

		// Final score
		println!("{}", total);
	}

	Ok(())
}
