use types::Cell;

pub enum Neighborhood {
    Moore(u32),
    VonNeumann(u32),
}

pub fn wrap_idx(idx: i64, limit: usize) -> i64 {
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

struct NeighborhoodCoordinatesIterator {
    row: i64,
    col: i64,
    nbrow: i64,
    nbcol: i64,
    lastrow: i64,
    lastcol: i64,
    startcol: i64,
    finished: bool,
}

impl NeighborhoodCoordinatesIterator {
    fn new(row: usize, col: usize, range: u32) -> NeighborhoodCoordinatesIterator {
        let row_sgn = row as i64;
        let col_sgn = col as i64;
        let range_sgn = range as i64;
        let nbrow = row_sgn - range_sgn;
        let nbcol = col_sgn - range_sgn;
        let lastrow = row_sgn + range_sgn;
        let lastcol = col_sgn + range_sgn;
        NeighborhoodCoordinatesIterator{
            row: row_sgn, col: col_sgn, nbrow: nbrow, nbcol: nbcol,
            lastrow: lastrow, lastcol: lastcol, startcol: nbcol,
            finished: false
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

impl Iterator for NeighborhoodCoordinatesIterator {
    type Item = (i64, i64);

    fn next(&mut self) -> Option<(i64, i64)> {
        if self.finished { return None };
        let result = Some((self.nbrow, self.nbcol));
        self.advance();
        result
    }
}

pub struct MooreNeighborhoodIterator<'a> {
    cells: &'a Vec<Vec<Cell>>,
    w: usize,
    h: usize,
    nci: NeighborhoodCoordinatesIterator,
}

impl<'a> MooreNeighborhoodIterator<'a> {
    pub fn new(cells: &'a Vec<Vec<Cell>>, width: usize, height: usize,
           row: usize, col: usize, range: u32) -> MooreNeighborhoodIterator {
        let nci = NeighborhoodCoordinatesIterator::new(row, col, range);
        MooreNeighborhoodIterator{cells: cells, w: width, h: height, nci: nci}
    }
}

impl<'a> Iterator for MooreNeighborhoodIterator<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Cell> {
        match self.nci.next() {
            Some((row, col)) => {
                if self.nci.row == row && self.nci.col == col {
                    self.next()
                } else {
                    let row = wrap_idx(row, self.h) as usize;
                    let col = wrap_idx(col, self.w) as usize;
                    Some(self.cells[row][col])
                }
            },
            None => None,
        }
    }
}

pub struct VonNeumannNeighborhoodIterator<'a> {
    cells: &'a Vec<Vec<Cell>>,
    w: usize,
    h: usize,
    range: i64,
    nci: NeighborhoodCoordinatesIterator,
}

impl<'a> VonNeumannNeighborhoodIterator<'a> {
    pub fn new(cells: &'a Vec<Vec<Cell>>, width: usize, height: usize,
           row: usize, col: usize, range: u32) -> VonNeumannNeighborhoodIterator {
        let nci = NeighborhoodCoordinatesIterator::new(row, col, range);
        VonNeumannNeighborhoodIterator{cells: cells, w: width, h: height,
                                       range: range as i64, nci: nci}
    }
}

impl<'a> Iterator for VonNeumannNeighborhoodIterator<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Cell> {
        match self.nci.next() {
            Some((row, col)) => {
                let dist = (self.nci.row-row).abs() + (self.nci.col-col).abs();
                if dist > self.range {
                    self.next()
                } else if self.nci.row == row && self.nci.col == col {
                    self.next()
                } else {
                    let row = wrap_idx(row, self.h) as usize;
                    let col = wrap_idx(col, self.w) as usize;
                    Some(self.cells[row][col])
                }
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gen;
    use types::Cell;

    #[test]
    fn test_moore_neighborhood_iterator() {
        let cells = gen::area_with_points(3, 3, vec![(0,0), (1,1), (2,2)]);
        let mut it = MooreNeighborhoodIterator::new(&cells, 3, 3, 1, 1, 1);
        let neighbors: Vec<Cell> = it.collect();
        assert_eq!(neighbors, vec![1, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_von_neumann_neighborhood_iterator() {
        let cells = gen::area_with_points(3, 3, vec![(0,0), (0,1), (1,0), (1,1), (2,2)]);
        let mut it = VonNeumannNeighborhoodIterator::new(&cells, 3, 3, 1, 1, 1);
        let neighbors: Vec<Cell> = it.collect();
        assert_eq!(neighbors, vec![1, 1, 0, 0]);
    }
}
