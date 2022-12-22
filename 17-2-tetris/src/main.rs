// Executes the program 17-2-tetris

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::io::Write;
use either::Either;

const TARGET_TIME:u64 = 1000000000000;
const DEFAULT_CHECKS:u64 = 100;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();
	let exe_path = match args.nth(1) {
		None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename")),
		Some(x) => x
	};
	let filename = match args.next() {
		None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 2 must be filename")),
		Some(x) => x
	};

	// Non-whitespace characters
	let file_len = std::fs::read_to_string(&filename);
	let file_len = file_len?.trim().len();
	if file_len == 0 { return Err(Error::new(ErrorKind::InvalidInput, "File empty?")) }

	let check_count = match match args.next() {
		Some(x) => Some(x.parse::<u64>().map_err(|_|Error::new(ErrorKind::InvalidInput, "Argument 3 must be positive number"))?),
		None => None
	} { Some(x) => x, None => DEFAULT_CHECKS };

	let checks = {
		use std::process::Command;
		let mut checks: Vec<u64> = Default::default();
		let mut last:Option<u64> = None;

		for idx in 0..check_count {
			let output = Command::new(&exe_path)
	            .args([&filename, "0", "0", &(idx*(file_len as u64)).to_string()])
	            .output()
	            .expect("failed to execute arg1 process");
	        let output = std::str::from_utf8(&output.stdout).map_err(|_|Error::new(ErrorKind::InvalidInput, "Run {}, process gave binary output?"))?;
	        let output = output.trim().parse::<u64>().map_err(|_|Error::new(ErrorKind::InvalidInput, format!("Run {}, process gave bad output", idx)))?;
	        //print!("[[[ {},{:?} ]]]", output, checks.last());
	        let output2 = output - last.unwrap_or(0);
	        checks.push(output2);
	        last = Some(output);
	        print!("{},", output2);
	        std::io::stdout().flush(); 
	    }
	    println!("");
		checks
	};

	// Initial offset
	let (offset, modulus) = 'cycle: {
		for idx_outer in 0..(check_count/2) {
			// Repeat size
			'offset: for idx_inner in 1..(check_count/2) {
				// Testing
				for idx_inner2 in (idx_outer + idx_inner)..check_count {
					if checks[idx_inner2 as usize] != checks[(idx_inner2 - idx_inner) as usize] {
						continue 'offset
					}
				}
				break 'cycle (idx_outer, idx_inner)
			}
		}
		return Err(Error::new(ErrorKind::InvalidInput, "No reasonable loops found"));
	};
	println!("Offset {}, cycle-size {}", offset, modulus);
	print!("Prefix: ");
	let mut prefix_total: u64 = 0;
	for idx in 0..offset {
		prefix_total += checks[idx as usize];
		print!("{},", checks[idx as usize]);
	}
	println!("");
	print!("Loop: ");
	let mut loop_total: u64 = 0;
	for idx in offset..(offset+modulus) {
		loop_total += checks[idx as usize];
		print!("{},", checks[idx as usize]);
	}
	println!("");

	let mut total: u64 = 0;

	let time = TARGET_TIME - offset;
	total += prefix_total;
	total += (TARGET_TIME / modulus) * loop_total;
	let leftover = TARGET_TIME % modulus;
	for idx in 0..leftover {
		total += checks[(offset + idx) as usize];
	}

	// Final score
	println!("{}", total);

	Ok(())
}
