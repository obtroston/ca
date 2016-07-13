use std::char;

extern crate rand;
use rand::distributions::{Range, IndependentSample};

pub mod gen;
pub mod nb;
pub mod types;

use types::Cell;

// (cells, width, index) -> new_state
pub type CA1Rule = Fn(&Vec<Cell>, usize, usize) -> Cell;

fn get_random_ca1_code(len: usize, base: usize) -> String {
    let base = base as u32;
    let range = Range::new(0, base);
    let mut rng = rand::thread_rng();
    let code: String = (0..len)
        .map(|_| char::from_digit(range.ind_sample(&mut rng), base).unwrap())
        .collect();
    code
}

pub fn get_ca1_rule(radius: u8, states: u8, code: Option<String>) -> Result<Box<CA1Rule>, String> {
    static ERR_ZERO_RADIUS: &'static str = "radius < 1!";
    static ERR_INVALID_STATES: &'static str = "states not in range 2-36!";
    static ERR_TOO_BIG_PARAMS: &'static str = "states.pow(radius*2+1) must fit in usize!";
    static ERR_INVALID_CODE_LEN: &'static str = "code must contain digit for every neighborhood!";

    if radius < 1 {
        return Err(String::from(ERR_ZERO_RADIUS));
    }
    if states < 2 || states > 36 {
        return Err(String::from(ERR_INVALID_STATES));
    }

    let radius = radius as usize;
    let nb_width = try!(radius.checked_mul(2)
        .ok_or(ERR_TOO_BIG_PARAMS)
        .and_then(|x| {
            x.checked_add(1)
                .ok_or(ERR_TOO_BIG_PARAMS)
        }));

    let states = states as usize;
    let mut neighborhoods = states;
    for _ in 1..nb_width {
        neighborhoods = try!(neighborhoods.checked_mul(states).ok_or(ERR_TOO_BIG_PARAMS));
    }
    let code = match code {
        Some(s) => s,
        None => get_random_ca1_code(neighborhoods, states),
    };
    if neighborhoods != code.len() {
        return Err(String::from(ERR_INVALID_CODE_LEN));
    }
    let mut rules: Vec<Cell> = vec![0; neighborhoods];
    for (i, c) in code.chars().rev().enumerate() {
        let new_state = try!(c.to_digit(states as u32)
            .ok_or(format!("{} is not a digit in base {}!", c, states)));
        rules[i] = new_state;
    }

    let radius = radius as i64;
    Ok(Box::new(move |cells, width, idx| {
        let idx = idx as i64;
        let idx_begin = idx - radius;
        let idx_end = idx + radius + 1;
        let mut nb_code: usize = 0;
        for i in idx_begin..idx_end {
            let i = nb::wrap_idx(i, width) as usize;
            let state = cells[i] as usize;
            nb_code = nb_code * states + state;
        }
        rules[nb_code]
    }))
}

pub fn get_elementary_rule(code: u8) -> Box<CA1Rule> {
    get_ca1_rule(1, 2, Some(format!("{:0>8b}", code))).unwrap()
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
        CA1 {
            w: w,
            cells: cells,
            future: future,
            rule: rule,
        }
    }

    pub fn new_ca1(cells: Vec<Cell>,
                   radius: u8,
                   states: u8,
                   code: Option<String>)
                   -> Result<CA1, String> {
        let rule = try!(get_ca1_rule(radius, states, code));
        Ok(CA1::new(cells, rule))
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
            0 => {
                if birth.contains(&live) {
                    1
                } else {
                    0
                }
            }
            _ => {
                if survive.contains(&live) {
                    1
                } else {
                    0
                }
            }
        }
    })
}

pub fn get_cyclic_rule(nbh: nb::Neighborhood, threshold: u8, states: u32) -> Box<CA2Rule> {
    Box::new(move |cells, w, h, row, col| {
        let cell = cells[row][col];
        let next = (cell + 1) % states;
        let mut cnt_next = 0;
        match nbh {
            nb::Neighborhood::Moore(range) => {
                let it = nb::MooreNeighborhoodIterator::new(cells, w, h, row, col, range);
                for nb in it {
                    if nb == next {
                        cnt_next += 1;
                    }
                }
            }
            nb::Neighborhood::VonNeumann(range) => {
                let it = nb::VonNeumannNeighborhoodIterator::new(cells, w, h, row, col, range);
                for nb in it {
                    if nb == next {
                        cnt_next += 1;
                    }
                }
            }
        };
        if cnt_next >= threshold {
            next
        } else {
            cell
        }
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
        CA2 {
            w: w,
            h: h,
            cells: cells,
            future: future,
            rule: rule,
        }
    }

    pub fn new_life(cells: Vec<Vec<Cell>>, survive: Vec<Cell>, birth: Vec<Cell>) -> CA2 {
        let rule = get_life_rule(survive, birth);
        CA2::new(cells, rule)
    }

    pub fn new_cyclic(cells: Vec<Vec<Cell>>,
                      nbh: nb::Neighborhood,
                      threshold: u8,
                      states: u32)
                      -> CA2 {
        let rule = get_cyclic_rule(nbh, threshold, states);
        CA2::new(cells, rule)
    }

    pub fn tick(&mut self) {
        for row in 0..self.h {
            for col in 0..self.w {
                self.future[row][col] = (self.rule)(&self.cells, self.w, self.h, row, col);
            }
        }
        for row in 0..self.h {
            self.cells[row].copy_from_slice(&self.future[row]);
        }
    }
}
