extern crate rand;

use self::rand::Rng;

use types::Cell;

pub fn random1d(w: usize, states: Vec<Cell>,
                i1: Option<usize>, i2: Option<usize>) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Cell> = vec![0; w];
    let min_idx = match i1 { None => 0, Some(i) => i };
    let max_idx = match i2 {
        None => w,
        Some(i) => if i < w { i } else { w },
    };
    for i in min_idx..max_idx {
        cells[i] = *rng.choose(&states).unwrap();
    }
    cells
}

pub fn random2d(w: usize, h: usize, states: Vec<Cell>,
                x1: Option<usize>, x2: Option<usize>,
                y1: Option<usize>, y2: Option<usize>) -> Vec<Vec<Cell>> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    let min_x = match x1 { None => 0, Some(x) => x };
    let max_x = match x2 {
        None => w,
        Some(x) => if x < w { x } else { w },
    };
    let min_y = match y1 { None => 0, Some(y) => y };
    let max_y = match y2 {
        None => h,
        Some(y) => if y < h { y } else { h },
    };
    for row in min_y..max_y {
        for col in min_x..max_x {
            cells[row][col] = *rng.choose(&states).unwrap();
        }
    }
    cells
}

pub fn points1d(w: usize, indexes: Vec<usize>) -> Vec<Cell> {
    let mut cells: Vec<Cell> = vec![0; w];
    for i in indexes {
        cells[i] = 1;
    }
    cells
}

pub fn points2d(w: usize, h: usize,
                coords: Vec<(usize, usize)>) -> Vec<Vec<Cell>> {
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    for coord in coords {
        let (x, y) = coord;
        cells[y][x] = 1;
    }
    cells
}
