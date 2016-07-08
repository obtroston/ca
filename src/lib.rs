pub mod gen;
pub mod nb;
pub mod types;

use types::Cell;

// (cells, width, index) -> new_state
pub type CA1Rule = Fn(&Vec<Cell>, usize, usize) -> Cell;

pub fn get_elementary_rule(code: u8) -> Box<CA1Rule> {
    Box::new(move |cells, width, idx| {
        let left = if idx <= 0 { cells[width-1] } else { cells[idx-1] };
        let center = cells[idx];
        let right = if idx >= width-1 { cells[0] } else { cells[idx+1] };
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

// (cells, width, height, row, col) -> new_state
pub type CA2Rule = Fn(&Vec<Vec<Cell>>, usize, usize, usize, usize) -> Cell;

pub fn get_life_rule(survive: Vec<Cell>, birth: Vec<Cell>) -> Box<CA2Rule> {
    Box::new(move |cells, w, h, row, col| {
        let mut live = 0;
        for nb in nb::MooreNeighborhoodIterator::new(cells, w, h, row, col, 1) {
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

pub fn get_cyclic_rule(nbh: nb::Neighborhood, threshold: u8,
                       states: u32) -> Box<CA2Rule> {
    Box::new(move |cells, w, h, row, col| {
        let cell = cells[row][col];
        let next = (cell+1) % states;
        let mut cnt_next = 0;
        match nbh {
            nb::Neighborhood::Moore(range) => {
                let it = nb::MooreNeighborhoodIterator::new(cells, w, h,
                                                            row, col, range);
                for nb in it {
                    if nb == next {
                        cnt_next += 1;
                    }
                }
            },
            nb::Neighborhood::VonNeumann(range) => {
                let it = nb::VonNeumannNeighborhoodIterator::new(cells, w, h,
                                                                 row, col, range);
                for nb in it {
                    if nb == next {
                        cnt_next += 1;
                    }
                }
            },
        };
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

    pub fn new_cyclic(cells: Vec<Vec<Cell>>, nbh: nb::Neighborhood,
                      threshold: u8, states: u32) -> CA2 {
        let rule = get_cyclic_rule(nbh, threshold, states);
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
