// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::{HashMap, HashSet};
use either::Either;
use glam::IVec2;

// Set 0 to disable
const DEBUG_ROUND:usize = 1;
const FINAL_ROUND:usize = 10;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();

	// If each elf has their own priority order, this is needed. But they don't. So it isn't.
	//const NIB:u8 = 0x3;
	//fn from_prio(prio: u8) { (prio&NIB, (prio>>2)&NIB, (prio>>4)&NIB, (prio>>6)&NIB ) }
	//fn to_prio((a,b,c,d): (u8,u8,u8,u8)) { (a&NIB) | ((b&NIB)<<2) | ((c&NIB)<<4) | ((d&NIB)<<6) }
	//const DEFAULT_PRIO = to_prio((0,1,2,3));
	const PATTERN: [[IVec2;3]; 4] = [
		[IVec2::new( 0,-1), IVec2::new( 1,-1), IVec2::new(-1,-1)], // N, NE, NW
		[IVec2::new( 0, 1), IVec2::new( 1, 1), IVec2::new(-1, 1)], // S, SE, SW
		[IVec2::new(-1, 0), IVec2::new(-1,-1), IVec2::new(-1, 1)], // W, NW, SW
		[IVec2::new( 0, 1), IVec2::new( 1,-1), IVec2::new( 1, 1)], // E, NE, SE
	];
	//fn check_pattern(prio: u8) -> [IVec2; 3] {
	//	return PATTERN[prio as usize];
	//}
	const RESULT: [IVec2; 4] = [ IVec2::NEG_Y, IVec2::Y, IVec2::NEG_X, IVec2::X ];
	//const check_result(prio: u8) -> IVec2 {
	//	return RESULT[prio as usize];
	//}

	let mut elves = {
		let filename = args.nth(1);
		let input: Either<BufReader<Stdin>, BufReader<File>> = match filename.as_deref() {
			None => return Err(Error::new(ErrorKind::InvalidInput, "Argument 1 must be filename or -")),
			Some("-") => either::Left(BufReader::new(stdin())),
			Some(x) => either::Right(BufReader::new(std::fs::File::open(x)?))
		};
		let lines = input.lines();
		
		let mut elves: Vec<IVec2> = Default::default();

		for (y,line) in lines.enumerate() {
			let line = line?;
			let line = line.trim_end();
			if line.is_empty() { break } // NOT DONE

			for (x,ch) in line.chars().enumerate() {
				if ch == '#' {
					elves.push(IVec2::new(x as i32,y as i32));
				}
			}
		}

		elves
	};

	if elves.is_empty() { return Err(Error::new(ErrorKind::InvalidInput, "No elves?")) }

	let success = 'round: { for round in 0.. {
		let mut elves_map:HashSet<IVec2> = Default::default();
		let mut elves_min = IVec2::new(i32::MAX, i32::MAX);
		let mut elves_max = IVec2::new(i32::MIN, i32::MIN);
		let mut elves_proposed:HashSet<IVec2> = Default::default();
		let mut elves_claim:HashMap<IVec2, usize> = Default::default();
		let round_prio = round % 4;
		let mut moved = false;
		let mut collided = false;

		// Round 1
		for &elf in elves.iter() {
			elves_min = elves_min.min(elf);
			elves_max = elves_max.max(elf);
			elves_map.insert(elf);
		}

		// Non-round: Debug printouts
		// So the math works, do this after building elves_map but before any mutation
		// (IE print on round 10 means "print after 10 rounds...")
		if DEBUG_ROUND > 0 && round % DEBUG_ROUND == 0 {
			let mut empty = 0;
			for y in elves_min.y..=elves_max.y {
				for x in elves_min.x..=elves_max.x {
					print!("{}",
						if elves.contains(&IVec2::new(x,y)) {
							'#'
						} else {
							empty += 1;
							'.'
						}
					)
				}
				println!("");
			}
			println!("Round: {} Score: {}\n", round, empty);
			if round>=FINAL_ROUND { break 'round true }
		}

		// Round 2
		for (elf_idx, &elf) in elves.iter().enumerate() {
			'prio: for prio_idx in 0..4 {
				let prio = (round_prio + prio_idx)%4;
				let clear = 'clear: {
					for check in PATTERN[prio] {
//println!("{} [{}] to {}? {}", elf_idx, elf, elf+check, !elves_map.contains(&(elf + check)));
						if elves_map.contains(&(elf + check)) { // Occupied, reject prio
							break 'clear false
						}
					}
					true
				};
				if clear {
					let move_to = elf + RESULT[prio];
					let unoccupied = elves_proposed.insert(move_to);
//println!("{} attempted {}. {}", elf_idx, move_to, if !unoccupied { "Occupied" } else { "SUCCESS" });
					if unoccupied {
						elves_claim.insert(move_to, elf_idx);
					} else {
						elves_claim.remove(&move_to);
						collided = true;
					}
					break 'prio
				}
			}
		}

		// Round 2.5?
		for (move_to, elf_idx) in elves_claim {
//println!("{} moved", elf_idx);
			elves[elf_idx] = move_to;
			moved = true;
		}
	} panic!("Unreachable"); };

	println!("{}", if success { "SUCCESS" } else { "FAILURE" });

	Ok(())
}
