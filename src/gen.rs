extern crate rand;

use self::rand::Rng;

use types::Cell;

pub fn random_area(w: usize, h: usize, states: Vec<Cell>) -> Vec<Vec<Cell>> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    for row in 0..h {
        for col in 0..w {
            cells[row][col] = *rng.choose(&states).unwrap();
        }
    }
    cells
}

pub fn area_with_points(w: usize, h: usize,
                        dots: Vec<(usize, usize)>) -> Vec<Vec<Cell>> {
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    for point in dots {
        let (x, y) = point;
        cells[y][x] = 1;
    }
    cells
}
