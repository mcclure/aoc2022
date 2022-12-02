use std::io::{stdin, Error, ErrorKind, BufRead, BufReader};

fn main() -> Result<(), Error> {
	let filename = std::env::args().fuse().nth(1);
	let input: BufReader = match &filename {
		None => BufReader::new(stdin()),
		Some(x) => BufReader::new(std::fs::File::open(x)?)
	};

	for line in input.lines() {
		let line = line?;
		if line.is_empty() {
			println!("EMPTY")
		} else {
			let calories = line.parse::<i64>();
			match calories {
				Ok(_) => println!("INT"),
				_ => return Err(Error::new(ErrorKind::InvalidInput, "Non-numeric input"))
			}
		}
	}

	Ok(())
}
