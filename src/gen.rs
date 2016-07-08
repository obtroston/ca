extern crate rand;

use self::rand::Rng;

use types::Cell;

pub fn random1d(w: usize, states: Vec<Cell>) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Cell> = vec![0; w];
    for i in 0..w {
        cells[i] = *rng.choose(&states).unwrap();
    }
    cells
}

pub fn random2d(w: usize, h: usize, states: Vec<Cell>) -> Vec<Vec<Cell>> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    for row in 0..h {
        for col in 0..w {
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
