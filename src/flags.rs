use nb;
use types::Cell;

enum AutomatonType {
    Life(Vec<Cell>, Vec<Cell>), // survive, birth
    Cyclic(nb::Neighborhood, u8, u32), // neighborhood, threshold, states
}

enum InitType {
    Random(Vec<Cell>), // states
    Points(Vec<(usize, usize)>), // coordinates
}

struct Options {
    automaton_type: AutomatonType,
    init_type: InitType,
}
