mod utils;

use wasm_bindgen::prelude::*;
use std::fmt;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type Value = usize;  // univeral value type

extern crate js_sys;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Live = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Live,
            Cell::Live => Cell::Dead,
        };
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: Value,
    height: Value,
    cells: Vec<Cell>,
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Universe {
    fn get_index(&self, row: Value, column: Value) -> Value {
        row * self.width + column
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe
    /// by passing the row and column of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(Value, Value)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Live;
        }
    }

    fn live_neighbor_count(&self, row: Value, column: Value) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new(width: Value, height: Value) -> Universe {
        utils::set_panic_hook();  // log panic messages to browser console

        let cells = (0..width * height).map(|_i| Cell::Dead).collect();

        log!("creating new {}x{} Universe", width, height);

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> Value {
        self.width
    }

    pub fn height(&self) -> Value {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn toggle_cell(&mut self, row: Value, column: Value) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn clear_grid(&mut self) {
        log!("clear_grid {}x{}", self.width, self.height);
        self.cells = (0..self.width * self.height)
            .map(|_i| Cell::Dead)
            .collect();
    }

    pub fn launch_ship(&mut self) {
        let ship = [(1,2), (2,3), (3,1), (3,2), (3,3)];
        log!("launch_ship {:?}", &ship);
        self.set_cells(&ship);
    }

    pub fn pattern_fill(&mut self) {
        log!("pattern_fill {}x{}", self.width, self.height);
        self.cells = (0..self.width * self.height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Live
                } else {
                    Cell::Dead
                }
            })
            .collect();
    }

    pub fn random_fill(&mut self) {
        log!("random_fill {}x{}", self.width, self.height);
        self.cells = (0..self.width * self.height)
            .map(|_i| {
                if js_sys::Math::random() < 0.375 {
                    Cell::Live
                } else {
                    Cell::Dead
                }
            })
            .collect();
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1:  Any live cell with fewer than two live neighbours
                    //          dies, as if caused by underpopulation.
                    (Cell::Live, x) if x < 2 => Cell::Dead,
                    // Rule 2:  Any live cell with two or three live neighbours
                    //          lives on to the next generation.
                    (Cell::Live, 2) | (Cell::Live, 3) => Cell::Live,
                    // Rule 3:  Any live cell with more than three live
                    //          neighbours dies, as if by overpopulation.
                    (Cell::Live, x) if x > 3 => Cell::Dead,
                    // Rule 4:  Any dead cell with exactly three live neighbours
                    //          becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Live,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }
}
