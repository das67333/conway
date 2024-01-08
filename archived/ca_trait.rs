use std::ops::{Bound, RangeBounds};

pub(crate) fn convert_bounds<S: RangeBounds<usize>>(range: S, size: usize) -> (usize, usize) {
    (
        match range.start_bound() {
            Bound::Included(&t) => t,
            Bound::Excluded(&t) => t + 1,
            Bound::Unbounded => 0,
        },
        match range.end_bound() {
            Bound::Included(&t) => t,
            Bound::Excluded(&t) => t - 1,
            Bound::Unbounded => size - 1,
        },
    )
}

pub trait CellularAutomaton: Sized {
    /// Name that is used in benchmarks
    fn id<'a>() -> &'a str;

    /// Creates a field filled with dead cells
    fn blank(width: usize, height: usize) -> Self;

    /// [`(width, height)`] of the field
    fn size(&self) -> (usize, usize);

    fn get_cell(&self, x: usize, y: usize) -> bool;

    fn set_cell(&mut self, x: usize, y: usize, state: bool);

    /// Updates the field `iters_cnt` times
    fn update(&mut self, iters_cnt: usize);

    /// Gets cells in rectangle
    fn get_cells(
        &self,
        x_range: impl RangeBounds<usize>,
        y_range: impl RangeBounds<usize>,
    ) -> Vec<bool> {
        let (x_min, x_max) = convert_bounds(x_range, self.size().0);
        let (y_min, y_max) = convert_bounds(y_range, self.size().1);
        (y_min..=y_max)
            .flat_map(|y| (x_min..=x_max).map(move |x| self.get_cell(x, y)))
            .collect()
    }

    /// Sets cells in rectangle
    fn set_cells(
        &mut self,
        x_range: impl RangeBounds<usize>,
        y_range: impl RangeBounds<usize>,
        mut data: impl Iterator<Item = bool>,
    ) {
        let (x_min, x_max) = convert_bounds(x_range, self.size().0);
        let (y_min, y_max) = convert_bounds(y_range, self.size().1);
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                self.set_cell(x, y, data.next().expect("Iterator concluded prematurely"));
            }
        }
    }

    /// Fills the field with random cells
    fn randomize(&mut self, seed: Option<u64>, fill_rate: f64) {
        use rand::{Rng, SeedableRng};

        let mut rng = if let Some(x) = seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(x)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        let (w, h) = self.size();
        self.set_cells(.., .., (0..w * h).map(|_| rng.gen_bool(fill_rate)));
    }

    /// Prints the field to the stdout
    #[cfg(debug_assertions)]
    fn println(&self) {
        const LIMIT: usize = 128;
        let (w, h) = self.size();
        assert!(w <= LIMIT && h <= LIMIT);
        for y in 0..h {
            for x in 0..w {
                print!("{}", self.get_cell(x, y) as u8);
                if x + 1 == w {
                    println!();
                }
            }
        }
        println!();
    }

    fn paste_rle(&mut self, x: usize, y: usize, data: &[u8]) {
        let parse_next_number = |i: &mut usize| {
            while !data[*i].is_ascii_digit() {
                *i += 1;
            }
            let j = {
                let mut j = *i;
                while j < data.len() && data[j].is_ascii_digit() {
                    j += 1;
                }
                j
            };
            let ans = String::from_utf8(data[*i..j].to_vec())
                .unwrap()
                .parse::<usize>()
                .unwrap();
            *i = j;
            ans
        };

        let mut i = 0;
        // skipping comment lines
        while data[i] == b'#' {
            while data[i] != b'\n' {
                i += 1;
            }
            i += 1;
        }
        // next line must start with 'x'; parsing sizes
        let width = parse_next_number(&mut i);
        let height = parse_next_number(&mut i);
        while data[i] != b'\n' {
            i += 1;
        }
        i += 1;
        // run-length encoded pattern data
        let (mut dx, mut dy, mut cnt) = (0, 0, 1);
        while i < data.len() {
            let c = data[i];
            match c {
                b'\n' => i += 1,
                b'0'..=b'9' => cnt = parse_next_number(&mut i),
                b'o' => {
                    for _ in 0..cnt {
                        self.set_cell(x + dx, y + dy, true);
                        dx += 1
                    }
                    (i, cnt) = (i + 1, 1);
                    assert!(
                        dx <= width,
                        "i={} {:?}",
                        i,
                        String::from_utf8(data[i - 36..=i].to_vec())
                    );
                }
                b'b' => {
                    (dx, i, cnt) = (dx + cnt, i + 1, 1);
                    assert!(dx <= width);
                }
                b'$' => {
                    (dx, dy, i, cnt) = (0, dy + cnt, i + 1, 1);
                    assert!(dy <= height);
                }
                b'!' => {
                    dy += 1;
                    assert!(dy <= height);
                    break;
                }
                _ => panic!("Unexpected symbol"),
            };
        }
        assert!(dy <= height);
    }

    
}
