// Perform a shortest-path search.

use std::io::{Error, ErrorKind};
use char_reader::CharReader;
use ndarray::{Array2, ArrayView, Axis};
use glam::IVec2;
use pathfinding::directed::astar::astar;
use ordered_float::NotNan;

const DEBUG:bool = false;
const DEBUG_ANIMATE:bool = false;

fn main() -> Result<(), Error> {
	let invalideg = |s| { Error::new(ErrorKind::InvalidInput, s) };

	let (grid, start, end) = { // Populate from file
		// Load file from command-line argument
		let filename = std::env::args().fuse().nth(1);
		let mut chars = match &filename {
			None => return Err(Error::new(ErrorKind::InvalidInput, "File argument expected")),
			Some(x) => CharReader::new(std::fs::File::open(x)?)
		};

		let invalid = || { Err(Error::new(ErrorKind::InvalidInput, "Inconsistent sized lines")) };
		let invalid2 = || { Err(Error::new(ErrorKind::InvalidInput, "Unrecognized characters")) };

		let mut grid : Option<Array2<u8>> = Default::default();
		let mut trimming = false;
		let mut width:usize = 0;
		let mut line:Vec<u8> = Default::default();
		let (mut start, mut end):(Option<IVec2>,Option<IVec2>) = Default::default(); 
		while let Some(ch) = chars.next_char()? {
			if ch == '\n' || ch == '\r' { // End of line
				trimming = false;
				if line.len()>0 {
					if width==0 { width = line.len() }
					else if line.len() != width { return invalid() }
					//println!("?? {} {} {}", line.len(), grid.nrows(), grid.ncols());
					if grid == None {
						grid = Some(Array2::zeros((0, width))); // ROW MAJOR
					} 
					grid.as_mut().unwrap().push_row(ArrayView::from(&line)).unwrap(); // Unwrap to panic on impossible error
					line = Default::default()
				}
				continue
			}

			// Allow whitespace at end of line but NOT before
			if ch.is_whitespace() {
				trimming = true;
				continue
			}
			if trimming { return invalid2() }

			line.push({ // Will panic on arrays longer than 2^31 I guess
				let at = || Some(IVec2::new(line.len() as i32, match grid { None => 0, Some(ref grid) => grid.nrows() as i32 }));
				match ch {
					'a'..='z' => { (ch as u8) - ('a' as u8) }
					'S' => { start = at(); 0  }
					'E' => { end   = at(); 25 }
					_ => return invalid2()
				}
			})
		}

		(grid .ok_or_else(||invalideg("File is empty"))?,
		 start.ok_or_else(||invalideg("No start point"))?,
		 end  .ok_or_else(||invalideg("No start point"))?)
	};

	//println!("{:?}, {}, {}", grid, start, end);

	{
		let cardinals = [IVec2::new(1,0), IVec2::new(0,-1), IVec2::new(-1,0), IVec2::new(0,1)];
		fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
		fn to_letter(u:u8) -> char { (u + ('a' as u8)) as char }
		let one = NotNan::new(1.0).unwrap();

		// <N, C, FN, IN, FH, FS> N = Vec2, C = f32, IN = vec<(Vec2,f32)>
		if let Some((path, _)) = astar(
		    &start,
		    |&at| {
		    	let mut ok:Vec<(IVec2,NotNan<f32>)> = Default::default();
		    	let at_val = grid[to_index(at)];
		    	for card in cardinals {
		    		let cand = at + card;
					if cand.x >= 0 && cand.y >= 0 {
						if let Some(&cand_val) = grid.get(to_index(cand)) {
							if !(cand_val > at_val + 1) {
								ok.push((cand, one))
							}
						}
					}
		    	}
		    	ok
		    },
		    |&at| NotNan::new((at - end).as_vec2().length()).unwrap(),
		    |&at| at == end
		) {
			if DEBUG {
				use ansi_term::Style;
				use ansi_term::Colour::{Black, White};

				let text = Style::new();
				let invert = Style::new().fg(Black).on(White);

				for &at in &path {
					if DEBUG_ANIMATE { print!("\x1B[2J\x1B[1;1H"); }
					println!("\tAt: {} Thinks: {}", at, to_letter(grid[to_index(at)]));
					for (y, col) in grid.axis_iter(Axis(0)).enumerate() {
						for (x, v) in col.iter().enumerate() {
							let print_at = IVec2::new(x as i32,y as i32);
							print!("{}", {
								let ch = (
									if print_at == start { 'S' }
									else if print_at == end { 'E' }
									else { to_letter(*v) }
								).to_string();
								if print_at == at { invert.paint(ch) } else { text.paint(ch) }
							})
						}
						println!("");
					}
					println!("");
				} 
			}

			println!("{}", path.len() - 1);

			Ok(())
		} else {
			Err(invalideg("No path from start to end"))
		}
	}
}
