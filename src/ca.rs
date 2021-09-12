//! Module for cellular automata

use std::mem;

/// C = Cell
///     - data type of the cell
/// T = Transition
///     - for a given cell and neighborhood, define how the new cell state is computed
///     - either clone the cell or create a diff
/// N = Neighborhood
///     - for a given cell (position) get all neighboring cells (positions)
///     - Fn(i32, i32) -> [(i32, i32)]
pub struct Simulation<C: Send> {
    width: i32,
    height: i32,
    transition: Box<dyn FnMut(&mut C, &[&C])>,
    neighborhood: Box<dyn Fn(i32, i32, i32, i32) -> Vec<(i32, i32)>>,
    state: Vec<C>,
    buffer: Vec<C>,
}

/// T applies a function to Cell of buffer 1 and neighborhood and then puts a clone of the cell with the new state in buffer 2
impl<C: Send> Simulation<C>
where
    C: Clone + Default + std::fmt::Debug,
{
    pub fn new(
        width: i32,
        height: i32,
        trans_fn: impl FnMut(&mut C, &[&C]) + 'static,
        neighbor_fn: impl Fn(i32, i32, i32, i32) -> Vec<(i32, i32)> + 'static,
    ) -> Self {
        let capacity: usize = (width * height) as usize;
        let state = vec![C::default(); capacity];
        let buffer = vec![C::default(); capacity];
        debug!("creating simulation");
        Simulation {
            width,
            height,
            transition: Box::new(trans_fn),
            neighborhood: Box::new(neighbor_fn),
            state,
            buffer,
        }
    }

    // TODO: Return error if `cells` doesn't match the width and height parameter.
    pub fn from_cells(
        width: i32,
        height: i32,
        trans_fn: impl FnMut(&mut C, &[&C]) + 'static,
        neighbor_fn: impl Fn(i32, i32, i32, i32) -> Vec<(i32, i32)> + 'static,
        cells: Vec<C>,
    ) -> Self {
        Simulation {
            width,
            height,
            transition: Box::new(trans_fn),
            neighborhood: Box::new(neighbor_fn),
            state: cells.to_vec(),
            buffer: cells,
        }
    }

    /// Perform one simulation step.
    pub fn step(&mut self) {
        // Manipulate the internal state of a cell the `buffer` grid by iterating over the cells at
        // the neighborhood coordinates in the `state` grid.
        let w = self.width;
        let buf_ref = &mut self.buffer;
        let state_ref = &self.state;
        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors: Vec<&C> = (self.neighborhood)(x, y, self.width, self.height)
                    .iter()
                    .map(|(i, j)| &state_ref[coord_to_idx(w, *i, *j)])
                    .collect();
                (self.transition)(&mut buf_ref[coord_to_idx(w, x, y)], &neighbors)
            }
        }

        // Swap the assignments of `state` and `buffer` to "update the grid", so to speak.
        mem::swap(&mut self.state, &mut self.buffer)
    }

    pub fn step_until(&mut self, step_count: i32) {
        for _ in 0..step_count {
            self.step();
        }
    }

    pub fn cells(&self) -> &[C] {
        &self.state
    }
}

pub fn coord_to_idx(width: i32, x: i32, y: i32) -> usize {
    (y * width + x) as usize
}

pub fn idx_to_coord(width: usize, idx: usize) -> (i32, i32) {
    let x = idx % width;
    let y = idx / width;
    (x as i32, y as i32)
}

static VON_NEUMAN_NEIGHBORHOOD: &'static [(i32, i32); 4] = &[(-1, 0), (0, -1), (1, 0), (0, 1)];

struct Neighborhood {
    count: usize,
    bounds: &'static [(i32, i32)],
    ca_bounds: (i32, i32),
    cell_coords: Option<(i32, i32)>,
}

impl Neighborhood {
    fn new(bounds: &'static [(i32, i32)], ca_bounds: (i32, i32)) -> Self {
        Neighborhood {
            count: 0,
            bounds,
            ca_bounds,
            cell_coords: None,
        }
    }

    fn init_with_cell(&mut self, cell: (i32, i32)) {
        self.cell_coords = Some(cell);
    }

    fn reset(&mut self) {
        self.count = 0;
    }
}

// Implement `Iterator` for `Fibonacci`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl Iterator for Neighborhood {
    // We can refer to this type using Self::Item
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.bounds.len() {
            None
        } else {
            let cell = self.cell_coords.unwrap();
            let neigh = (
                self.bounds[self.count].0 + cell.0,
                self.bounds[self.count].1 + cell.1,
            );
            let idx = coord_to_idx(self.ca_bounds.0, cell.0, cell.1);
            if idx < 0 || idx > self.ca_bounds.0 {
                None
            } else {
                Some(idx)
            }
        }
    }
}

pub fn von_neuman(x: i32, y: i32, width: i32, height: i32) -> Vec<(i32, i32)> {
    VON_NEUMAN_NEIGHBORHOOD
        .iter()
        .map(|(a, b)| (x + a, y + b))
        .filter(|(a, b)| *a >= 0 && *a < width && *b >= 0 && *b < height)
        .collect::<Vec<(i32, i32)>>()
}

#[test]
fn test_roundtrip_idx_coords() {
    for idx in 0..9_999 {
        for width in 1..10_000 {
            let coord = idx_to_coord(width, idx);
            let new_idx = coord_to_idx(width as i32, coord.0, coord.1);
            assert!(idx == new_idx);
        }
    }
}

#[test]
fn test_roundtrip_coords_idx() {
    for width in 1..49 {
        for y in 0..49 {
            for x in 0..(width - 1) {
                let idx = coord_to_idx(width, x, y);
                let new_coord = idx_to_coord(width as usize, idx);
                assert!(new_coord.0 == x && new_coord.1 == y);
            }
        }
    }
}
