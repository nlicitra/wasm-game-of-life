#![allow(unused)]
mod canvas;
mod utils;

use js_sys::Math;
use std::sync::{Arc, Mutex};
use std::thread;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

const CELL_SIZE: u32 = 4;
const GRID_COLOR: &str = "#CCCCCC";
const DEAD_COLOR: &str = "#FFF";
const ALIVE_COLOR: &str = "cornflowerblue";

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
impl Cell {
    fn next_state(&self, live_neighbors: u32) -> Cell {
        match (*self, live_neighbors) {
            // Rule 1: Any live cell with fewer than two live neighbours
            // dies, as if caused by underpopulation.
            (Cell::Alive, x) if x < 2 => Cell::Dead,
            // Rule 2: Any live cell with two or three live neighbours
            // lives on to the next generation.
            (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
            // Rule 3: Any live cell with more than three live
            // neighbours dies, as if by overpopulation
            (Cell::Alive, x) if x > 0 => Cell::Dead,
            // Rule 4: Any dead cell with exactly three live neighbours
            // becomes a live cell, as if by reproduction.
            (Cell::Dead, 3) => Cell::Alive,
            // All other cells remain in the same state
            (otherwise, _) => otherwise,
        }
    }
}

// fn get_next_grid(board: &Universe) -> Vec<Cell> {
//     let size = board.height as usize;
//     let mut next = board.cells.clone();
//     let cells = board.cells.clone();

//     crossbeam::scope(|scope| {
//         let rows = next.chunks_mut(size);
//         for (rindex, row) in rows.enumerate() {
//             scope.spawn(move |_| {
//                 for col in 0..size {
//                     let index = col as usize;
//                     let cell = row[index];
//                     let live_neighbors =
//                         live_neighbor_count(&cells, size as u32, rindex as u32, col as u32);
//                     row[index] = next_cell;
//                 }
//             });
//         }
//     })
//     .unwrap();
//     next
// }

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Grid {
    cells: Vec<Cell>,
    size: u32,
}

#[wasm_bindgen]
impl Grid {
    pub fn new(size: u32) -> Self {
        let cells = (0..size * size)
            .map(|_| {
                if Math::random() >= 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
        Self { cells, size }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.size + column) as usize
    }

    fn get_cell(&self, row: u32, column: u32) -> Cell {
        let index = self.get_index(row, column);
        self.cells[index]
    }
    fn get_next_cell(&self, row: u32, column: u32) -> Cell {
        let index = self.get_index(row, column);
        let live_neighbors = self.live_neighbor_count(row, column);
        self.cells[index].next_state(live_neighbors)
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u32 {
        let mut count = 0;
        for delta_row in [self.size - 1, 0, 1].iter().cloned() {
            for delta_col in [self.size - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.size;
                let neighbor_col = (column + delta_col) % self.size;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u32;
            }
        }
        count
    }

    pub fn width(&self) -> u32 {
        self.size
    }

    pub fn height(&self) -> u32 {
        self.size
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn tick(&mut self) {
        let counter = Arc::new(&self);
        let mut new_cells = self.cells.clone();
        let rows = new_cells.chunks_mut(self.size as usize);
        let size = self.size;
        for (row, row_cells) in rows.enumerate() {
            for col in 0..size {
                let index = self.get_index(row as u32, col);
                let board = *counter;
                row_cells[col as usize] = board.get_next_cell(row as u32, col);
            }
        }
        self.cells = new_cells;
    }
}

#[wasm_bindgen]
pub struct Universe {
    grid: Grid,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(size: u32) -> Self {
        // let mut cells = FixedBitSet::with_capacity(size);
        // for i in 0..size {
        //     cells.set(i, Math::random() >= 0.5);
        // }

        let context = canvas::initialize_canvas_context().unwrap();
        let canvas = context.canvas().unwrap();
        canvas.set_width(size * (CELL_SIZE + 1) + 1);
        canvas.set_height(size * (CELL_SIZE + 1) + 1);

        Self {
            grid: Grid::new(size),
            context,
        }
    }

    // pub fn render(&self) -> String {
    //     self.to_string()
    // }

    pub fn tick(&mut self) {
        self.grid.tick();
    }

    pub fn render(&self) {
        self.draw_grid();
        self.draw_cells();
    }

    fn draw_grid(&self) {
        self.context.begin_path();
        let stroke_style = JsValue::from_str(GRID_COLOR);
        self.context.set_stroke_style(&stroke_style);

        for i in 0..self.grid.width() {
            self.context.move_to((i * (CELL_SIZE + 1) + 1) as f64, 0.);
            self.context.line_to(
                (i * (CELL_SIZE + 1) + 1) as f64,
                (((CELL_SIZE + 1) * self.grid.height()) + 1) as f64,
            );
        }

        for j in 0..self.grid.width() {
            self.context.move_to(0., (j * (CELL_SIZE + 1) + 1) as f64);
            self.context.line_to(
                (((CELL_SIZE + 1) * self.grid.width()) + 1) as f64,
                (j * (CELL_SIZE + 1) + 1) as f64,
            );
        }

        self.context.stroke();
    }

    fn draw_cells(&self) {
        self.context.begin_path();
        for row in 0..self.grid.width() {
            for col in 0..self.grid.height() {
                let cell = self.grid.get_cell(row, col);
                let fill_style = match cell {
                    Cell::Alive => ALIVE_COLOR,
                    Cell::Dead => DEAD_COLOR,
                };
                let fill_style = JsValue::from_str(fill_style);
                self.context.set_fill_style(&fill_style);
                self.context.fill_rect(
                    (col * (CELL_SIZE + 1) + 1) as f64,
                    (row * (CELL_SIZE + 1) + 1) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                )
            }
        }
        self.context.stroke();
    }
}
