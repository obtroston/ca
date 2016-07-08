extern crate ca;

use ca::types::Cell;

const ERR_NO_INIT_POINTS: &'static str = "INIT_POINTS is not set!";
const ERR_INVALID_INIT_POINTS: &'static str = "Invalid INIT_POINTS value!";

pub enum AutomatonType {
    Elementary(u8), // code
    Cyclic(ca::nb::Neighborhood, u8, u32), // neighborhood, threshold, states
    Life(Vec<Cell>, Vec<Cell>), // survive, birth
}

pub enum InitType {
    Random(Vec<Cell>), // states
    Points1D(Vec<usize>), // indexes
    Points2D(Vec<(usize, usize)>), // coordinates
}

pub struct Options {
    pub automaton_type: AutomatonType,
    pub init_type: InitType,
    pub size: Option<(u32, u32)>,
    pub cell_width: Option<u32>,
    pub delay: u32,
}

fn parse_u32(args: &Vec<String>, idx: usize) -> Result<(u32, usize), ()> {
    if args.len() <= idx {
        return Err(());
    }
    match args[idx].parse::<u32>() {
        Ok(val) => Ok((val, idx+1)),
        Err(_) => Err(()),
    }
}

fn parse_elementary_automaton(
    args: &Vec<String>, idx: usize
) -> Result<(AutomatonType, usize), &'static str> {
    let (code, idx) = try!(parse_u32(args, idx).map_err(|_| "Specify rule code!"));
    if code > 255 {
        return Err("Rule code must be in range 0-255!");
    }
    Ok((AutomatonType::Elementary(code as u8), idx))
}

fn parse_neighborhood(args: &Vec<String>,
                      idx: usize) -> Result<(ca::nb::Neighborhood, usize), &'static str> {
    if args.len() <= idx {
        return Err("Expected neighborhood, found end of args!");
    }
    match &args[idx][..1] {
        c @ "m" | c @ "n" => {
            match (&args[idx][1..]).parse::<u32>() {
                Ok(range) => Ok((
                    match c {
                        "m" => ca::nb::Neighborhood::Moore(range),
                        "n" => ca::nb::Neighborhood::VonNeumann(range),
                        _ => unreachable!(),
                    },
                    idx+1,
                )),
                Err(_) => Err("Neighborhood range must be unsigned 32-bit integer!")
            }
        },
        _ => Err("Neighborhood must start with 'm' or 'n'!"),
    }
}

fn parse_cyclic_automaton(args: &Vec<String>,
                          idx: usize) -> Result<(AutomatonType, usize), &'static str> {
    let (nb, idx) = try!(parse_neighborhood(args, idx));
    let (threshold, idx) = try!(
        match parse_u32(args, idx) {
            Ok((val, idx)) => {
                if val > 255 {
                    Err("Threshold must be in range 0..255!")
                } else {
                    Ok((val as u8, idx))
                }
            },
            Err(_) => Err("Invalid or no threshold value!"),
        }
    );
    let (states, idx) = try!(
        match parse_u32(args, idx) {
            Ok((args, idx)) => Ok((args, idx)),
            Err(_) => Err("Invalid or no states count!"),
        }
    );
    Ok((AutomatonType::Cyclic(nb, threshold, states), idx))
}

fn parse_u32_csv(args: &Vec<String>,
                 idx: usize) -> Result<(Vec<u32>, usize), ()> {
    if args.len() <= idx {
        return Err(());
    }
    if args[idx] == "empty" {
        return Ok((Vec::new(), idx+1));
    }
    let mut ints: Vec<u32> = Vec::new();
    for s in args[idx].split(",") {
        match s.parse::<u32>() {
            Ok(val) => ints.push(val),
            Err(_) => return Err(()),
        }
    }
    Ok((ints, idx+1))
}

fn parse_life_automaton(args: &Vec<String>,
                        idx: usize) -> Result<(AutomatonType, usize), &'static str> {
    let (survive, idx) = try!(
        match parse_u32_csv(args, idx) {
            Ok((survive, idx)) => Ok((survive, idx)),
            Err(_) => Err("Invalid SURVIVE_COUNTS value!"),
        }
    );
    let (birth, idx) = try!(
        match parse_u32_csv(args, idx) {
            Ok((birth, idx)) => Ok((birth, idx)),
            Err(_) => Err("Invalid BIRTH_COUNTS value!"),
        }
    );
    Ok((AutomatonType::Life(survive, birth), idx))
}

fn parse_automaton_type(args: &Vec<String>,
                        idx: usize) -> Result<(AutomatonType, usize), &'static str> {
    if args.len() <= idx {
        return Err("Specify automaton type!");
    }
    match &*args[idx] {
        "elementary" => parse_elementary_automaton(args, idx+1),
        "cyclic" => parse_cyclic_automaton(args, idx+1),
        "life" => parse_life_automaton(args, idx+1),
        _ => Err("Unknown automaton type!"),
    }
}

fn parse_init_state(part: &str) -> Result<(u32, u32), ()> {
    match part.find('*') {
        None => {
            match part.parse::<u32>() {
                Ok(val) => Ok((val, 1)),
                Err(_) => Err(()),
            }
        },
        Some(pos) => {
            match part[..pos].parse::<u32>() {
                Ok(val) => match part[pos+1..].parse::<u32>() {
                    Ok(count) => Ok((val, count)),
                    Err(_) => Err(()),
                },
                Err(_) => Err(()),
            }
        },
    }
}

fn parse_init_random(
    args: &Vec<String>, idx: usize, automaton_type: &AutomatonType
) -> Result<(InitType, usize), &'static str> {
    if args.len() <= idx ||
       args[idx].find(|c: char| !c.is_digit(10) && c != ',' && c != '*').is_some() ||
       args[idx] == "default" {
        return Ok((
            InitType::Random(
                match *automaton_type {
                   AutomatonType::Cyclic(_, _, states) => { (0..states).collect() },
                   _ => { vec![0, 1] },
                }
            ),
            if args.len() > idx && args[idx] == "default" { idx+1 } else { idx },
        ))
    }
    let mut states = Vec::new();
    for part in args[idx].split(',') {
        let (state, count) = try!(parse_init_state(part)
                                  .map_err(|_| "Invalid INIT_STATES value!"));
        for _ in 0..count {
            states.push(state);
        }
    }
    Ok((InitType::Random(states), idx+1))
}

fn parse_points1d(args: &Vec<String>, idx: usize) -> Result<(InitType, usize), ()> {
    if args.len() <= idx {
        return Err(());
    }
    let (indexes, idx) = try!(parse_u32_csv(args, idx).map_err(|_| ()));
    let indexes = indexes.iter().map(|x| *x as usize).collect();
    Ok((InitType::Points1D(indexes), idx))
}

fn parse_points2d(args: &Vec<String>, idx: usize) -> Result<(InitType, usize), ()> {
    if args.len() <= idx {
        return Err(())
    }
    let mut points: Vec<(usize, usize)> = Vec::new();
    for part in args[idx].split(';') {
        let point_str: Vec<&str> = part.split(',').collect();
        if point_str.len() != 2 {
            return Err(());
        }
        let x = try!(point_str[0].parse::<usize>().map_err(|_| ()));
        let y = try!(point_str[1].parse::<usize>().map_err(|_| ()));
        points.push((x, y));
    }
    Ok((InitType::Points2D(points), idx+1))
}

fn parse_init_points(
    args: &Vec<String>, idx: usize, automaton_type: &AutomatonType
) -> Result<(InitType, usize), &'static str> {
    if args.len() <= idx {
        return Err(ERR_NO_INIT_POINTS);
    }
    let (init_type, idx) = try!((match *automaton_type {
        AutomatonType::Elementary(..) => parse_points1d(args, idx),
        _ => parse_points2d(args, idx),
    }).map_err(|_| ERR_INVALID_INIT_POINTS));
    Ok((init_type, idx))
}

fn parse_init_type(
    args: &Vec<String>, idx: usize, automaton_type: &AutomatonType
) -> Result<(InitType, usize), &'static str> {
    if args.len() <= idx {
        return Err("Specify initialization type!");
    }
    match &*args[idx] {
        "random" => parse_init_random(args, idx+1, automaton_type),
        "points" => parse_init_points(args, idx+1, automaton_type),
        _ => Err("Unknown initialization type!"),
    }
}

fn parse_size(args: &Vec<String>,
              idx: usize) -> Result<(Option<(u32, u32)>, usize), &'static str> {
    if args.len() <= idx {
        return Ok((None, idx))
    }
    let xpos = try!(args[idx].find('x')
                    .ok_or("Size must me specified as WIDTHxHEIGHT!"));
    let w = try!(args[idx][..xpos].parse::<u32>().map_err(|_| "Invalid width!"));
    let h = try!(args[idx][xpos+1..].parse::<u32>().map_err(|_| "Invalid height!"));
    Ok((Some((w, h)), idx+1))
}

fn parse_sdl_params(
    args: &Vec<String>, mut idx: usize
) -> Result<(Option<(u32, u32)>, Option<u32>, u32, usize), &'static str> {
    let mut cell_width: Option<u32> = None;
    let mut delay = 5;
    let (size, _idx) = try!(parse_size(args, idx));
    idx = _idx;
    if args.len() > idx {
        let (_cell_width, _idx) = try!(parse_u32(args, idx)
                                       .map_err(|_| "Invalid CELL_WIDTH value!"));
        cell_width = Some(_cell_width);
        idx = _idx;
    }
    if args.len() > idx {
        let (_delay, _idx) = try!(parse_u32(args, idx)
                                  .map_err(|_| "Invalid DELAY value!"));
        delay = _delay;
        idx = _idx;
    }
    Ok((size, cell_width, delay, idx))
}

pub fn parse_args(args: Vec<String>) -> Result<Options, &'static str> {
    let idx: usize = 1;
    let (automaton_type, idx) = try!(parse_automaton_type(&args, idx));
    let (init_type, idx) = try!(parse_init_type(&args, idx, &automaton_type));
    let (size, cell_width, delay, idx) = try!(parse_sdl_params(&args, idx));
    if idx < args.len() {
        return Err("Trailing args!");
    }
    Ok(Options{automaton_type: automaton_type, init_type: init_type,
               size: size, cell_width: cell_width, delay: delay})
}
