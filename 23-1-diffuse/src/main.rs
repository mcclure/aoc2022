// Summary

use std::io::{BufRead, BufReader, Error, ErrorKind, Stdin, stdin};
use std::fs::File;
use std::collections::{HashMap, HashSet};
use either::Either;
use glam::IVec2;

// Set 0 to disable
const DEBUG_ROUND:usize = 1;
const FINAL_ROUND:usize = 0;

fn main() -> Result<(), Error> {
    // Load file from command-line argument or (if -) stdin
	let mut args = std::env::args().fuse();

	// If each elf has their own priority order, this is needed. But they don't. So it isn't.
	//const NIB:u8 = 0x3;
	//fn from_prio(prio: u8) { (prio&NIB, (prio>>2)&NIB, (prio>>4)&NIB, (prio>>6)&NIB ) }
	//fn to_prio((a,b,c,d): (u8,u8,u8,u8)) { (a&NIB) | ((b&NIB)<<2) | ((c&NIB)<<4) | ((d&NIB)<<6) }
	//const DEFAULT_PRIO = to_prio((0,1,2,3));
	// Clockwise
	const COMPASS: [IVec2;8] = [IVec2::new( 0,-1), IVec2::new( 1,-1), IVec2::new( 0, 1), IVec2::new( 1, 1),  // 0:N 1:NE 2:E 3:SE
	                 IVec2::new( 0, 1), IVec2::new(-1, 1), IVec2::new(-1, 0), IVec2::new(-1,-1)]; // 4:S 5:SW 6:W 7:NW
	const PATTERN: [[usize;3]; 4] = [
		[0, 1, 7], // N, NE, NW
		[4, 3, 5], // S, SE, SW
		[6, 7, 5], // W, NW, SW
		[2, 1, 3], // E, NE, SE
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

	fn print_elves(round:usize, elves_map: &HashSet<IVec2>, elves_min: IVec2, elves_max: IVec2) {
		let mut empty = 0;
		for y in elves_min.y..=elves_max.y {
			for x in elves_min.x..=elves_max.x {
				print!("{}",
					if elves_map.contains(&IVec2::new(x,y)) {
						'#'
					} else {
						empty += 1;
						'.'
					}
				)
			}
			println!("");
		}
		println!("\tRound: {} Score: {}\n", round, empty);
	}

	let success = 'round: {
		let mut last_moved: isize = -1;
		for round in 0..(isize::MAX as usize) {
			let mut elves_map:HashSet<IVec2> = Default::default();
			let mut elves_min = IVec2::new(i32::MAX, i32::MAX);
			let mut elves_max = IVec2::new(i32::MIN, i32::MIN);
			let mut elves_proposed:HashSet<IVec2> = Default::default();
			let mut elves_claim:HashMap<IVec2, usize> = Default::default();
			let round_prio = round % 4;
			let mut any_crowded = false;

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
				print_elves(round, &elves_map, elves_min, elves_max);
				if FINAL_ROUND>0 && round>=FINAL_ROUND { break 'round true }
			}

			// Round 2
			'elf: for (elf_idx, &elf) in elves.iter().enumerate() {
				let mut all_unoccupied = true;
				let compass_occupied = COMPASS.map(|v| {
					let occupied = elves_map.contains(&(elf + v));
					all_unoccupied = all_unoccupied && !occupied;
					occupied
				});

				if all_unoccupied { continue 'elf }
				any_crowded = true;

				'prio: for prio_idx in 0..4 {
					let prio = (round_prio + prio_idx)%4;
					let clear = 'clear: {
						for check in PATTERN[prio] {
	//println!("{} [{}] to {}? {}", elf_idx, elf, elf+check, !elves_map.contains(&(elf + check)));
							if compass_occupied[check] { // Occupied, reject prio
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
						}
						break 'prio
					}
				}
			}

			// Round 2.5?
			for (move_to, elf_idx) in elves_claim {
	//println!("{} moved", elf_idx);
				elves[elf_idx] = move_to;
				last_moved = round as isize;
			}

			{
				let (mut done, mut success) = (false, false);
				if !any_crowded { // Nobody needed to move this round.
					success = true; done = true;
					println!("No moves needed.");
				} else if round >= 4 {
					let iround = round as isize;
					let since_moved = iround - last_moved;
					if since_moved >= 4 {
						success = false; done = true;
						println!("No remaining moves.");
					}
				}
				if done {
					break 'round success
				}
			}
		}
		println!("This is taking too long."); // Exceeded SIZE_MAX?!
		false
	};

	println!("{}", if success { "SUCCESS" } else { "FAILURE" });

	Ok(())
}
