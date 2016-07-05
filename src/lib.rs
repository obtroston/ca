extern crate rand;

use rand::Rng;

pub type Cell = u32;

// (cells, width, index) -> new_state
pub type CA1Rule = Fn(&Vec<Cell>, usize, usize) -> Cell;

pub fn get_elementary_rule(code: u8) -> Box<CA1Rule> {
    Box::new(move |cells, width, idx| {
        let left = if idx <= 0 { cells[width-1] } else { cells[idx-1] };
        let center = cells[idx];
        let right = if idx >= width { cells[0] } else { cells[idx+1] };
        let code = code as u32;
        match (left, center, right) {
            (0, 0, 0) => code & 1,
            (0, 0, 1) => code >> 1 & 1,
            (0, 1, 0) => code >> 2 & 1,
            (0, 1, 1) => code >> 3 & 1,
            (1, 0, 0) => code >> 4 & 1,
            (1, 0, 1) => code >> 5 & 1,
            (1, 1, 0) => code >> 6 & 1,
            (1, 1, 1) => code >> 7 & 1,
            _ => panic!("unexpected neighborhood: {} {} {}", left, center, right),
        }
    })
}

pub struct CA1 {
    pub w: usize,
    pub cells: Vec<Cell>,
    future: Vec<Cell>,
    rule: Box<CA1Rule>,
}

impl CA1 {
    pub fn new(cells: Vec<Cell>, rule: Box<CA1Rule>) -> CA1 {
        let w = cells.len();
        let future = cells.to_vec();
        CA1{w: w, cells: cells, future: future, rule: rule}
    }

    pub fn new_elementary(cells: Vec<Cell>, code: u8) -> CA1 {
        let rule = get_elementary_rule(code);
        CA1::new(cells, rule)
    }

    pub fn tick(&mut self) {
        for idx in 0..self.w {
            self.future[idx] = (self.rule)(&self.cells, self.w, idx);
        }
        self.cells.copy_from_slice(&self.future);
    }
}

pub enum Neighborhood {
    Moore(u32),
    VonNeumann(u32),
}

fn wrap_idx(idx: i64, limit: usize) -> i64 {
    let limit = limit as i64;
    let idx = idx % limit;
    if idx < 0 { idx + limit } else { idx }
}

#[test]
fn test_wrap_idx() {
    assert_eq!(wrap_idx(-3, 10), 7);
    assert_eq!(wrap_idx(3, 10), 3);
    assert_eq!(wrap_idx(13, 10), 3);
}

struct MooreNeighborhoodIterator<'a> {
    cells: &'a Vec<Vec<Cell>>,
    width: usize,
    height: usize,
    row: i64,
    col: i64,
    nbrow: i64,
    nbcol: i64,
    lastrow: i64,
    lastcol: i64,
    startcol: i64,
    finished: bool,
}

impl<'a> MooreNeighborhoodIterator<'a> {
    fn new(cells: &'a Vec<Vec<Cell>>, width: usize, height: usize,
           row: usize, col: usize, range: u32) -> MooreNeighborhoodIterator {
        let row_sgn = row as i64;
        let col_sgn = col as i64;
        let range_sgn = range as i64;
        let nbrow = row_sgn - range_sgn;
        let nbcol = col_sgn - range_sgn;
        let lastrow = row_sgn + range_sgn;
        let lastcol = col_sgn + range_sgn;
        MooreNeighborhoodIterator{
            cells: cells, width: width, height: height,
            row: row_sgn, col: col_sgn, nbrow: nbrow, nbcol: nbcol,
            lastrow: lastrow, lastcol: lastcol, startcol: nbcol,
            finished: false,
        }
    }

    fn advance(&mut self) {
        if self.nbcol < self.lastcol {
            self.nbcol += 1;
        } else if self.nbrow < self.lastrow {
            self.nbrow += 1;
            self.nbcol = self.startcol;
        } else {
            self.finished = true;
        }
    }
}

impl<'a> Iterator for MooreNeighborhoodIterator<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Cell> {
        if self.finished { return None };
        let row = wrap_idx(self.nbrow, self.height) as usize;
        let col = wrap_idx(self.nbcol, self.width) as usize;
        let result = self.cells[row][col];
        self.advance();
        if self.nbrow == (self.row as i64) && self.nbcol == (self.col as i64) {
            self.advance();
        }
        Some(result)
    }
}

#[test]
fn test_moore_neighborhood_iterator() {
    let cells = get_area_with_points(3, 3, vec![(0,0), (1,1), (2,2)]);
    let mut it = MooreNeighborhoodIterator::new(&cells, 3, 3, 1, 1, 1);
    let neighbors: Vec<Cell> = it.collect();
    assert_eq!(neighbors, vec![1, 0, 0, 0, 0, 0, 0, 1]);
}

// (cells, width, height, row, col) -> new_state
pub type CA2Rule = Fn(&Vec<Vec<Cell>>, usize, usize, usize, usize) -> Cell;

pub fn get_life_rule(survive: Vec<Cell>, birth: Vec<Cell>) -> Box<CA2Rule> {
    Box::new(move |cells, w, h, row, col| {
        let mut live = 0;
        for nb in MooreNeighborhoodIterator::new(cells, w, h, row, col, 1) {
            if nb == 1 {
                live += 1;
            }
        }
        match cells[row][col] {
            0 => if birth.contains(&live) { 1 } else { 0 },
            _ => if survive.contains(&live) { 1 } else { 0 },
        }
    })
}

pub fn get_cyclic_rule(range: u32, threshold: u8, states: u32) -> Box<CA2Rule> {
    Box::new(move |cells, w, h, row, col| {
        let cell = cells[row][col];
        let next = (cell+1) % states;
        let mut cnt_next = 0;
        for nb in MooreNeighborhoodIterator::new(cells, w, h, row, col, range) {
            if nb == next {
                cnt_next += 1;
            }
        }
        if cnt_next >= threshold { next } else { cell }
    })
}

pub struct CA2 {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Vec<Cell>>,
    future: Vec<Vec<Cell>>,
    rule: Box<CA2Rule>,
}

impl CA2 {
    pub fn new(cells: Vec<Vec<Cell>>, rule: Box<CA2Rule>) -> CA2 {
        let h = cells.len();
        let w = cells[0].len();
        let future = cells.to_vec();
        CA2{w: w, h: h, cells: cells, future: future, rule: rule}
    }

    pub fn new_life(cells: Vec<Vec<Cell>>,
                    survive: Vec<Cell>, birth: Vec<Cell>) -> CA2 {
        let rule = get_life_rule(survive, birth);
        CA2::new(cells, rule)
    }

    pub fn new_cyclic(cells: Vec<Vec<Cell>>, range: u32, threshold: u8,
                      states: u32) -> CA2 {
        let rule = get_cyclic_rule(range, threshold, states);
        CA2::new(cells, rule)
    }

    pub fn tick(&mut self) {
        for row in 0..self.h {
            for col in 0..self.w {
                self.future[row][col] = (self.rule)(
                    &self.cells, self.w, self.h, row, col
                );
            }
        }
        for row in 0..self.h {
            self.cells[row].copy_from_slice(&self.future[row]);
        }
    }
}

pub fn get_random_area(w: usize, h: usize, states: Vec<Cell>) -> Vec<Vec<Cell>> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    for row in 0..h {
        for col in 0..w {
            cells[row][col] = *rng.choose(&states).unwrap();
        }
    }
    cells
}

pub fn get_area_with_points(w: usize, h: usize,
                            dots: Vec<(usize, usize)>) -> Vec<Vec<Cell>> {
    let mut cells: Vec<Vec<Cell>> = vec![vec![0; w]; h];
    for point in dots {
        let (x, y) = point;
        cells[y][x] = 1;
    }
    cells
}
