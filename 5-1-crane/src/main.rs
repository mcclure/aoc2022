// Simulate a crane robot based on a drawing and a series of instructions.

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use either::Either;

use regex::Regex;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let input: Either<BufReader<Stdin>, BufReader<File>> = match &filename {
		None => either::Left(BufReader::new(stdin())),
		Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
	};

	let mut lines = input.lines();

	let invalid =   || { Err(Error::new(ErrorKind::InvalidInput, "Did not find expected ascii art diagram")) };
	let invalide2 = || { Error::new(ErrorKind::InvalidInput, "Expected sentence like 'move x from y to z'") };
	let invalid2 =  || { Err(invalide2()) };
	let invalide3 = || { Error::new(ErrorKind::InvalidInput, "Out of crates") };

	// Series of either three spaces or [W], separated by spaces. Will capture W or S (for Word or Space)
	let separator_re = Regex::new(r"^\p{gc:Zs}").unwrap();
	let blank_re = Regex::new(r"^\p{gc:Zs}{3}").unwrap();
	let crate_re = Regex::new(r"^\[(\w)\]").unwrap();
	let numbers_re = Regex::new(r"^[\s\d]+$").unwrap();
	let move_re = Regex::new(r"^move (\d+) from (\d+) to (\d+)$").unwrap();

	// Returns rest of string after match
	fn match_next<'a>(m:regex::Captures, s:&'a str) -> &'a str {
		return &s[m.get(0).unwrap().end()..]
	}

	// Returns first match group, rest of string after match
	fn match_next_get<'a, 'b>(m:regex::Captures<'a>, s:&'b str) -> (&'a str, &'b str) {
		return (m.get(1).unwrap().as_str(), match_next(m, s))
	}

	fn index_two<T>(a:& mut[T], b:usize, c:usize) -> (&mut T, &mut T) {
		let ordered = b < c;
		let (low_idx, high_idx) = if ordered { (b,c) } else { (c,b) };
		let (low_slice, high_slice) = a.split_at_mut(high_idx);
		let (low, high) = (&mut low_slice[low_idx], &mut high_slice[0]);
		return if ordered { (low, high) } else { (high, low) }
	}

	let mut data:Vec<Vec<char>> = Vec::new();

	// Scan file
	for line in lines.by_ref() {
		let line = line?;
		let mut rest = line.as_str();
		println!("Line");

		// Note: Moves to phase 2 on first empty line, ignores number "comment"
		// Does NOT check accuracy of number "comment"
		if rest.is_empty() { break }
		if let Some(_) = numbers_re.captures(rest) { continue }

		let mut column = 0;
		loop {
			// Blank space before crate
			if column > 0 {
				if let Some(capture) = separator_re.captures(rest) {
					rest = match_next(capture, rest)
				} else {
					break // End of string
				}
			}
			// No crate
			if let Some(capture) = blank_re.captures(rest) {
				rest = match_next(capture, rest);
			// Crate
			} else if let Some(capture) = crate_re.captures(rest) {
				let tag:&str;
				(tag, rest) = match_next_get(capture, rest);
				while data.len() <= column
					{ data.push(Vec::new()) }
				let tag_ch = tag.chars().next().unwrap();
				data[column].push(tag_ch);
				println!("Column {} tag {}", column, tag_ch);
			} else {
				return invalid();
			}
			column += 1
		}
	}

	if data.len() == 0 { return invalid() }

	// Reverse all columns of data
	// Note column not of same type as before
	for column in data.iter_mut() {
		column.reverse()
	}

	for line in lines {
		let line = line?;
		println!("Command: {} On: {:?}", line, data);

		if let Some(capture) = move_re.captures(&line) {
			let v = capture.iter().skip(1)
				.map(|x| match x {
					None => Err(invalide2()),
					Some(x) => x.as_str().parse::<usize>().map_err(|_|invalide2())
				}).collect::<Result<Vec<usize>, Error>>()?;

			let [a,b,c] = <[usize; 3]>::try_from(v).ok().unwrap();

			if b == 0 || c == 0 { return invalid2() }
			if b != c {
				let (column_from, column_to) = index_two(&mut data, b-1, c-1);

				// This is wrong, but isn't it nice?! // Update: This turns out to be the 5-2 puzzle actually
				//let column_from_n = column_from.len();
				//let column_from_post_n = column_from_n-a;
				// column_to.extend_from_slice(&column_from[column_from_post_n..column_from_n]);
				// column_from.truncate(column_from_post_n);
				for _ in 0..a {
					column_to.push(column_from.pop().ok_or_else(invalide3)?)
				}
			}
		} else {
			return invalid2()
		}
	}

	// Debug, print entire tree
	println!("Final: {:?}", data);

	// Result code
	for column in data {
		print!("{}", column.last().unwrap_or(&' '));
	}
	println!("");

	Ok(())
}
