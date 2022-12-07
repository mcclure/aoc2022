// Find character where 4 unalike characters appear in a row
// Does not support stdin

use std::io::{Error, ErrorKind};
use char_reader::CharReader;

// Looking for 4 character sequence, so remember 3.
const BACK:usize = 14;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if none) stdin
	let filename = std::env::args().fuse().nth(1);
	let mut chars = match &filename {
		None => return Err(Error::new(ErrorKind::InvalidInput, "File argument expected")),
		Some(x) => CharReader::new(std::fs::File::open(x)?)
	};

	let mut seen: usize = 0;
	let mut back: [char; BACK] = [' '; BACK];

	let invalid = || { return Err(Error::new(ErrorKind::InvalidInput, "No repeating characters")) };

	// Scan file
	while let Ok(Some(ch)) = chars.next_char() {
		if ch.is_whitespace() { continue }

		back[seen % BACK] = ch;
		seen += 1;

		//println!("{} in {:?}", ch, back);

		let repeated = seen <= BACK
			|| 'repeated: {
				for left_idx in 0..(BACK-1) {
					for right_idx in 0..BACK {
						if left_idx != right_idx && back[left_idx] == back[right_idx] {
							break 'repeated true
						}
					}
				}
				false
			};

		if !repeated {
			println!("{}", seen); // Success
			return Ok(())
		}
	}

	// Should not escape loop
	invalid()
}
