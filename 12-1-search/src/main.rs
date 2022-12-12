// Perform a shortest-path search.

use std::io::{Error, ErrorKind};
use char_reader::CharReader;
use ndarray::{Array2, ArrayView};
use glam::UVec2;

fn main() -> Result<(), Error> {
	let (grid, start, end) = { // Populate from file
		// Load file from command-line argument
		let filename = std::env::args().fuse().nth(1);
		let mut chars = match &filename {
			None => return Err(Error::new(ErrorKind::InvalidInput, "File argument expected")),
			Some(x) => CharReader::new(std::fs::File::open(x)?)
		};

		let invalid = || { Err(Error::new(ErrorKind::InvalidInput, "Inconsistent sized lines")) };
		let invalid2 = || { Err(Error::new(ErrorKind::InvalidInput, "Unrecognized characters")) };
		let invalideg = |s| { Error::new(ErrorKind::InvalidInput, s) };

		let mut grid : Option<Array2<u8>> = Default::default();
		let mut trimming = false;
		let mut width:usize = 0;
		let mut line:Vec<u8> = Default::default();
		let (mut start, mut end):(Option<UVec2>,Option<UVec2>) = Default::default(); 
		while let Some(ch) = chars.next_char()? {
			if ch == '\n' || ch == '\r' { // End of line
				trimming = false;
				if line.len()>0 {
					if width==0 { width = line.len() }
					else if line.len() != width { return invalid() }
					//println!("?? {} {} {}", line.len(), grid.nrows(), grid.ncols());
					if grid == None {
						grid = Some(Array2::zeros((0,width)));
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

			line.push(match ch {
				'a'..='z' => { (ch as u8) - ('a' as u8) }
				'S' => { start = Some(UVec2::new(line.len() as u32, match grid { None => 0, Some(ref grid) => grid.nrows() as u32 })); 0  }
				'E' => { end   = Some(UVec2::new(line.len() as u32, match grid { None => 0, Some(ref grid) => grid.nrows() as u32 })); 25 }
				_ => return invalid2()
			})
		}

		(grid .ok_or_else(||invalideg("File is empty"))?,
		 start.ok_or_else(||invalideg("No start point"))?,
		 end  .ok_or_else(||invalideg("No start point"))?)
	};

	println!("{:?}, {}, {}", grid, start, end);

	// Final score
	//println!("{}", total);

	Ok(())
}
